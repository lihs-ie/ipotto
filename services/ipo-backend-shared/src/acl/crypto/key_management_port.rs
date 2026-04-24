use async_trait::async_trait;
use zeroize::Zeroizing;

use crate::errors::DomainError;

/// A freshly generated data encryption key paired with its KEK-wrapped form.
///
/// The plaintext is held in [`Zeroizing`] so the key material is wiped from
/// memory as soon as it goes out of scope. `wrapped` is the blob that is safe
/// to persist alongside the ciphertext (Secret Manager etc.).
pub struct GeneratedDataKey {
    pub plaintext: Zeroizing<Vec<u8>>,
    pub wrapped: Vec<u8>,
}

/// Port for KEK-managed key wrapping used by envelope encryption.
///
/// Implementations wrap a freshly-generated data encryption key with a
/// key-encryption key held by the provider (Cloud KMS in production, a fixed
/// key in tests). Callers never see the raw KEK.
#[async_trait]
pub trait KeyManagementPort: Send + Sync {
    async fn generate_data_key(&self) -> Result<GeneratedDataKey, DomainError>;

    async fn unwrap_data_key(&self, wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>, DomainError>;
}
