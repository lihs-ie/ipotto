use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Key, Nonce,
};
use async_trait::async_trait;
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroizing;

use crate::{
    acl::crypto::{GeneratedDataKey, KeyManagementPort},
    errors::DomainError,
};

const WRAP_AAD: &[u8] = b"ipo-backend-shared/in-memory-kek/v1";

/// Test double for [`KeyManagementPort`] that wraps DEKs with a fixed,
/// in-process KEK using AES-256-GCM. Never use in production.
#[derive(Clone)]
pub struct InMemoryKeyManagement {
    kek: Zeroizing<[u8; 32]>,
}

impl InMemoryKeyManagement {
    pub fn from_bytes(kek: [u8; 32]) -> Self {
        Self {
            kek: Zeroizing::new(kek),
        }
    }

    pub fn random_for_test() -> Self {
        let mut kek = [0u8; 32];
        OsRng.fill_bytes(&mut kek);
        Self::from_bytes(kek)
    }

    fn cipher(&self) -> Aes256Gcm {
        Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(self.kek.as_ref()))
    }
}

impl core::fmt::Debug for InMemoryKeyManagement {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.debug_struct("InMemoryKeyManagement").finish()
    }
}

#[async_trait]
impl KeyManagementPort for InMemoryKeyManagement {
    async fn generate_data_key(&self) -> Result<GeneratedDataKey, DomainError> {
        let mut dek = Zeroizing::new(vec![0u8; 32]);
        OsRng.fill_bytes(dek.as_mut_slice());

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);

        let ciphertext = self
            .cipher()
            .encrypt(
                Nonce::from_slice(&nonce_bytes),
                Payload {
                    msg: dek.as_slice(),
                    aad: WRAP_AAD,
                },
            )
            .map_err(|error| DomainError::KeyManagementError {
                reason: format!("failed to wrap data key: {error}"),
            })?;

        let mut wrapped = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        wrapped.extend_from_slice(&nonce_bytes);
        wrapped.extend_from_slice(&ciphertext);

        Ok(GeneratedDataKey {
            plaintext: dek,
            wrapped,
        })
    }

    async fn unwrap_data_key(&self, wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>, DomainError> {
        if wrapped.len() < 12 {
            return Err(DomainError::KeyManagementError {
                reason: "wrapped data key is too short".to_string(),
            });
        }
        let (nonce_bytes, ciphertext) = wrapped.split_at(12);
        let plaintext = self
            .cipher()
            .decrypt(
                Nonce::from_slice(nonce_bytes),
                Payload {
                    msg: ciphertext,
                    aad: WRAP_AAD,
                },
            )
            .map_err(|error| DomainError::KeyManagementError {
                reason: format!("failed to unwrap data key: {error}"),
            })?;
        Ok(Zeroizing::new(plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn generate_then_unwrap_roundtrips_dek() {
        let kms = InMemoryKeyManagement::random_for_test();
        let generated = kms.generate_data_key().await.expect("generate succeeds");
        let unwrapped = kms
            .unwrap_data_key(&generated.wrapped)
            .await
            .expect("unwrap succeeds");
        assert_eq!(unwrapped.as_slice(), generated.plaintext.as_slice());
        assert_eq!(generated.plaintext.len(), 32);
    }

    #[tokio::test]
    async fn unwrap_rejects_tampered_wrapped_dek() {
        let kms = InMemoryKeyManagement::random_for_test();
        let generated = kms.generate_data_key().await.expect("generate succeeds");
        let mut tampered = generated.wrapped.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 0x01;
        let result = kms.unwrap_data_key(&tampered).await;
        assert!(matches!(
            result,
            Err(DomainError::KeyManagementError { .. })
        ));
    }

    #[tokio::test]
    async fn unwrap_rejects_short_blob() {
        let kms = InMemoryKeyManagement::random_for_test();
        let result = kms.unwrap_data_key(&[0u8; 4]).await;
        assert!(matches!(
            result,
            Err(DomainError::KeyManagementError { .. })
        ));
    }

    #[tokio::test]
    async fn two_generations_produce_distinct_deks() {
        let kms = InMemoryKeyManagement::random_for_test();
        let first = kms.generate_data_key().await.expect("generate succeeds");
        let second = kms.generate_data_key().await.expect("generate succeeds");
        assert_ne!(first.plaintext.as_slice(), second.plaintext.as_slice());
        assert_ne!(first.wrapped, second.wrapped);
    }
}
