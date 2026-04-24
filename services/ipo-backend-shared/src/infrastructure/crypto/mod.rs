pub mod encrypted_credential_store;
pub mod google_kms_key_management;
pub mod in_memory_key_management;

use std::sync::Arc;

pub use encrypted_credential_store::EncryptedCredentialStore;
pub use google_kms_key_management::GoogleKmsKeyManagement;
pub use in_memory_key_management::InMemoryKeyManagement;

use crate::{acl::crypto::KeyManagementPort, errors::DomainError};

/// Builds the [`KeyManagementPort`] implementation selected by the
/// `KEY_MANAGEMENT_BACKEND` environment variable (values: `in-memory` or
/// `google-kms`). When the variable is missing, `google-kms` is the default so
/// production deployments always reach Cloud KMS. Local `make up` runs should
/// set `KEY_MANAGEMENT_BACKEND=in-memory` to avoid needing a live project.
///
/// `kek_name` is required when the backend is `google-kms`; for `in-memory`
/// the argument is ignored and a random KEK is generated in-process.
pub async fn build_key_management(
    kek_name: &str,
) -> Result<Arc<dyn KeyManagementPort>, DomainError> {
    let backend =
        std::env::var("KEY_MANAGEMENT_BACKEND").unwrap_or_else(|_| "google-kms".to_string());
    match backend.as_str() {
        "in-memory" => Ok(Arc::new(InMemoryKeyManagement::random_for_test())),
        "google-kms" => {
            if kek_name.is_empty() {
                return Err(DomainError::KeyManagementError {
                    reason: "IPOTTO_CREDENTIAL_KEK_NAME must be set for google-kms backend"
                        .to_string(),
                });
            }
            Ok(Arc::new(GoogleKmsKeyManagement::new(kek_name).await?))
        }
        other => Err(DomainError::KeyManagementError {
            reason: format!(
                "unknown KEY_MANAGEMENT_BACKEND '{other}' (expected 'in-memory' or 'google-kms')"
            ),
        }),
    }
}
