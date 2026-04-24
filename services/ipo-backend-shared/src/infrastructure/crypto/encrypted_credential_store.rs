use std::sync::Arc;

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Key, Nonce,
};
use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::{
    acl::{crypto::KeyManagementPort, secrets::CredentialStorePort},
    errors::DomainError,
};

const CURRENT_ENVELOPE_VERSION: u8 = 1;

/// Decorator that applies envelope encryption on top of a backing
/// [`CredentialStorePort`]. Plaintext values are encrypted with a fresh
/// AES-256-GCM data key per `save`, the data key is wrapped by the KMS, and
/// the resulting bundle is persisted as UTF-8 JSON through `inner`.
///
/// The secret key passed to `save`/`get` doubles as the AEAD associated data,
/// so a ciphertext produced for one key fails to decrypt when fetched under
/// another key — catching swap attacks at the storage layer.
pub struct EncryptedCredentialStore {
    inner: Arc<dyn CredentialStorePort>,
    kms: Arc<dyn KeyManagementPort>,
}

impl EncryptedCredentialStore {
    pub fn new(inner: Arc<dyn CredentialStorePort>, kms: Arc<dyn KeyManagementPort>) -> Self {
        Self { inner, kms }
    }
}

impl core::fmt::Debug for EncryptedCredentialStore {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.debug_struct("EncryptedCredentialStore").finish()
    }
}

#[derive(Serialize, Deserialize)]
struct EnvelopeBundle {
    version: u8,
    wrapped_dek: String,
    nonce: String,
    ciphertext: String,
}

#[async_trait]
impl CredentialStorePort for EncryptedCredentialStore {
    async fn save(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let data_key = self.kms.generate_data_key().await?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(data_key.plaintext.as_slice()));
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce_bytes),
                Payload {
                    msg: value.as_bytes(),
                    aad: key.as_bytes(),
                },
            )
            .map_err(|error| DomainError::EnvelopeDecryptionError {
                reason: format!("failed to encrypt payload: {error}"),
            })?;

        let bundle = EnvelopeBundle {
            version: CURRENT_ENVELOPE_VERSION,
            wrapped_dek: URL_SAFE_NO_PAD.encode(&data_key.wrapped),
            nonce: URL_SAFE_NO_PAD.encode(nonce_bytes),
            ciphertext: URL_SAFE_NO_PAD.encode(&ciphertext),
        };

        let serialized =
            serde_json::to_string(&bundle).map_err(|error| DomainError::SecretPayloadError {
                reason: format!("failed to serialize envelope bundle: {error}"),
            })?;

        self.inner.save(key, &serialized).await
    }

    async fn get(&self, key: &str) -> Result<String, DomainError> {
        let stored = self.inner.get(key).await?;
        let bundle: EnvelopeBundle = serde_json::from_str(&stored).map_err(|error| {
            DomainError::EnvelopeDecryptionError {
                reason: format!("failed to parse envelope bundle for {key}: {error}"),
            }
        })?;

        if bundle.version != CURRENT_ENVELOPE_VERSION {
            return Err(DomainError::EnvelopeDecryptionError {
                reason: format!("unsupported envelope version {} for {key}", bundle.version),
            });
        }

        let wrapped_dek = URL_SAFE_NO_PAD
            .decode(bundle.wrapped_dek.as_bytes())
            .map_err(|error| DomainError::EnvelopeDecryptionError {
                reason: format!("wrapped_dek is not valid base64url: {error}"),
            })?;
        let nonce_bytes = URL_SAFE_NO_PAD
            .decode(bundle.nonce.as_bytes())
            .map_err(|error| DomainError::EnvelopeDecryptionError {
                reason: format!("nonce is not valid base64url: {error}"),
            })?;
        let ciphertext = URL_SAFE_NO_PAD
            .decode(bundle.ciphertext.as_bytes())
            .map_err(|error| DomainError::EnvelopeDecryptionError {
                reason: format!("ciphertext is not valid base64url: {error}"),
            })?;

        if nonce_bytes.len() != 12 {
            return Err(DomainError::EnvelopeDecryptionError {
                reason: format!("nonce must be 12 bytes, got {}", nonce_bytes.len()),
            });
        }

        let dek = self.kms.unwrap_data_key(&wrapped_dek).await?;
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(dek.as_slice()));
        let plaintext = cipher
            .decrypt(
                Nonce::from_slice(&nonce_bytes),
                Payload {
                    msg: &ciphertext,
                    aad: key.as_bytes(),
                },
            )
            .map_err(|error| DomainError::EnvelopeDecryptionError {
                reason: format!("failed to decrypt payload for {key}: {error}"),
            })?;

        String::from_utf8(plaintext).map_err(|error| DomainError::EnvelopeDecryptionError {
            reason: format!("decrypted payload is not valid UTF-8: {error}"),
        })
    }

    async fn delete(&self, key: &str) -> Result<(), DomainError> {
        self.inner.delete(key).await
    }

    async fn exists(&self, key: &str) -> Result<bool, DomainError> {
        self.inner.exists(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::{
        crypto::in_memory_key_management::InMemoryKeyManagement, secrets::InMemoryCredentialStore,
    };

    fn build_store() -> (Arc<InMemoryCredentialStore>, EncryptedCredentialStore) {
        let inner = Arc::new(InMemoryCredentialStore::new());
        let kms = Arc::new(InMemoryKeyManagement::random_for_test());
        let store = EncryptedCredentialStore::new(inner.clone(), kms);
        (inner, store)
    }

    #[tokio::test]
    async fn save_then_get_roundtrips_plaintext() {
        let (_inner, store) = build_store();
        let key = "ipo-account-01HZXY";
        let value = r#"{"login_id":"user","login_password":"p@ss"}"#;

        store.save(key, value).await.expect("save succeeds");
        let retrieved = store.get(key).await.expect("get succeeds");

        assert_eq!(retrieved, value);
    }

    #[tokio::test]
    async fn save_is_non_deterministic() {
        let (inner, store) = build_store();
        let key = "ipo-account-01HZXY";
        let value = "same-secret";

        store.save(key, value).await.expect("save succeeds");
        let first = inner.get(key).await.expect("inner get succeeds");

        store.save(key, value).await.expect("save succeeds");
        let second = inner.get(key).await.expect("inner get succeeds");

        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn get_rejects_tampered_ciphertext() {
        let (inner, store) = build_store();
        let key = "ipo-account-01HZXY";
        store.save(key, "top secret").await.expect("save succeeds");

        let stored = inner.get(key).await.expect("inner get succeeds");
        let mut bundle: EnvelopeBundle = serde_json::from_str(&stored).expect("bundle parses");
        let mut ciphertext = URL_SAFE_NO_PAD
            .decode(bundle.ciphertext.as_bytes())
            .expect("ciphertext decodes");
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0x01;
        bundle.ciphertext = URL_SAFE_NO_PAD.encode(&ciphertext);
        let reserialized = serde_json::to_string(&bundle).expect("bundle serializes");
        inner
            .save(key, &reserialized)
            .await
            .expect("inner save succeeds");

        let result = store.get(key).await;
        assert!(matches!(
            result,
            Err(DomainError::EnvelopeDecryptionError { .. })
        ));
    }

    #[tokio::test]
    async fn get_rejects_aad_mismatch() {
        let (inner, store) = build_store();
        let original_key = "ipo-account-01HZXY";
        let other_key = "ipo-account-99ZZZZ";

        store
            .save(original_key, "top secret")
            .await
            .expect("save succeeds");
        let stored = inner.get(original_key).await.expect("inner get succeeds");
        inner
            .save(other_key, &stored)
            .await
            .expect("inner save succeeds");

        let result = store.get(other_key).await;
        assert!(matches!(
            result,
            Err(DomainError::EnvelopeDecryptionError { .. })
        ));
    }

    #[tokio::test]
    async fn get_rejects_unknown_version() {
        let (inner, store) = build_store();
        let key = "ipo-account-01HZXY";
        store.save(key, "top secret").await.expect("save succeeds");

        let stored = inner.get(key).await.expect("inner get succeeds");
        let mut bundle: EnvelopeBundle = serde_json::from_str(&stored).expect("bundle parses");
        bundle.version = 99;
        let reserialized = serde_json::to_string(&bundle).expect("bundle serializes");
        inner
            .save(key, &reserialized)
            .await
            .expect("inner save succeeds");

        let result = store.get(key).await;
        assert!(matches!(
            result,
            Err(DomainError::EnvelopeDecryptionError { .. })
        ));
    }

    #[tokio::test]
    async fn get_rejects_tampered_wrapped_dek() {
        let (inner, store) = build_store();
        let key = "ipo-account-01HZXY";
        store.save(key, "top secret").await.expect("save succeeds");

        let stored = inner.get(key).await.expect("inner get succeeds");
        let mut bundle: EnvelopeBundle = serde_json::from_str(&stored).expect("bundle parses");
        let mut wrapped = URL_SAFE_NO_PAD
            .decode(bundle.wrapped_dek.as_bytes())
            .expect("wrapped decodes");
        let last = wrapped.len() - 1;
        wrapped[last] ^= 0x01;
        bundle.wrapped_dek = URL_SAFE_NO_PAD.encode(&wrapped);
        let reserialized = serde_json::to_string(&bundle).expect("bundle serializes");
        inner
            .save(key, &reserialized)
            .await
            .expect("inner save succeeds");

        let result = store.get(key).await;
        assert!(matches!(
            result,
            Err(DomainError::KeyManagementError { .. })
        ));
    }

    #[tokio::test]
    async fn delete_and_exists_passthrough() {
        let (inner, store) = build_store();
        let key = "ipo-account-01HZXY";

        assert!(!store.exists(key).await.expect("exists works"));
        store.save(key, "value").await.expect("save succeeds");
        assert!(store.exists(key).await.expect("exists works"));
        assert!(inner.exists(key).await.expect("inner exists works"));

        store.delete(key).await.expect("delete succeeds");
        assert!(!store.exists(key).await.expect("exists works"));
        assert!(!inner.exists(key).await.expect("inner exists works"));
    }
}
