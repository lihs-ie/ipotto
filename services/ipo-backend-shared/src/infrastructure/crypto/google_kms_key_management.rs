use std::sync::Arc;

use async_trait::async_trait;
use gcloud_sdk::{
    google::cloud::kms::v1::{
        key_management_service_client::KeyManagementServiceClient, DecryptRequest,
    },
    proto_ext::kms::EncryptRequest,
    tonic, GoogleApi, GoogleAuthMiddleware, SecretValue,
};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroizing;

use crate::{
    acl::crypto::{GeneratedDataKey, KeyManagementPort},
    errors::DomainError,
};

const KMS_URL: &str = "https://cloudkms.googleapis.com";
const WRAP_AAD: &[u8] = b"ipo-backend-shared/google-kms/v1";
const DEK_LEN: usize = 32;

/// [`KeyManagementPort`] implementation backed by Google Cloud KMS.
///
/// Data encryption keys (DEKs) are generated locally with `OsRng`, wrapped by
/// the configured KEK through `KeyManagementService.Encrypt`, and unwrapped
/// through `KeyManagementService.Decrypt`. Using a SOFTWARE-protected KEK is
/// sufficient: `GenerateRandomBytes` is avoided because it requires an HSM key
/// ring.
#[derive(Clone)]
pub struct GoogleKmsKeyManagement {
    kek_name: String,
    client: Arc<GoogleApi<KeyManagementServiceClient<GoogleAuthMiddleware>>>,
}

impl core::fmt::Debug for GoogleKmsKeyManagement {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("GoogleKmsKeyManagement")
            .field("kek_name", &self.kek_name)
            .finish()
    }
}

impl GoogleKmsKeyManagement {
    /// Builds a KMS client authenticated via ADC / workload identity.
    /// `kek_name` must be a full resource name such as
    /// `projects/{project}/locations/{location}/keyRings/{ring}/cryptoKeys/{key}`.
    pub async fn new(kek_name: impl Into<String>) -> Result<Self, DomainError> {
        let client = GoogleApi::from_function(KeyManagementServiceClient::new, KMS_URL, None)
            .await
            .map_err(|error| DomainError::KeyManagementError {
                reason: format!("failed to build Cloud KMS client: {error}"),
            })?;
        Ok(Self {
            kek_name: kek_name.into(),
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl KeyManagementPort for GoogleKmsKeyManagement {
    async fn generate_data_key(&self) -> Result<GeneratedDataKey, DomainError> {
        let mut dek = Zeroizing::new(vec![0u8; DEK_LEN]);
        OsRng.fill_bytes(dek.as_mut_slice());

        let response = self
            .client
            .get()
            .encrypt(tonic::Request::new(EncryptRequest {
                name: self.kek_name.clone(),
                plaintext: SecretValue::new(dek.to_vec()),
                additional_authenticated_data: WRAP_AAD.to_vec(),
                plaintext_crc32c: None,
                additional_authenticated_data_crc32c: None,
            }))
            .await
            .map_err(|error| DomainError::KeyManagementError {
                reason: format!("Cloud KMS Encrypt failed for {}: {error}", self.kek_name),
            })?;

        let ciphertext = response.into_inner().ciphertext;
        if ciphertext.is_empty() {
            return Err(DomainError::KeyManagementError {
                reason: format!("Cloud KMS returned empty ciphertext for {}", self.kek_name),
            });
        }

        Ok(GeneratedDataKey {
            plaintext: dek,
            wrapped: ciphertext,
        })
    }

    async fn unwrap_data_key(&self, wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>, DomainError> {
        let response = self
            .client
            .get()
            .decrypt(tonic::Request::new(DecryptRequest {
                name: self.kek_name.clone(),
                ciphertext: wrapped.to_vec(),
                additional_authenticated_data: WRAP_AAD.to_vec(),
                ciphertext_crc32c: None,
                additional_authenticated_data_crc32c: None,
            }))
            .await
            .map_err(|error| DomainError::KeyManagementError {
                reason: format!("Cloud KMS Decrypt failed for {}: {error}", self.kek_name),
            })?;

        let plaintext = response.into_inner().plaintext;
        Ok(Zeroizing::new(plaintext.ref_sensitive_value().to_vec()))
    }
}
