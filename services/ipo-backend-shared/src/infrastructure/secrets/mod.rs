pub mod google_secret_manager_store;
pub mod in_memory_credential_store;
pub mod secret_name;

use std::sync::Arc;

pub use google_secret_manager_store::GoogleSecretManagerStore;
pub use in_memory_credential_store::InMemoryCredentialStore;
pub use secret_name::{
    account_credential_secret_name, gmail_oauth_secret_name, sendgrid_api_key_secret_name,
};

use crate::{acl::secrets::CredentialStorePort, errors::DomainError};

/// Builds the [`CredentialStorePort`] implementation selected by the
/// `CREDENTIAL_STORE_BACKEND` environment variable (values:
/// `in-memory` or `google-secret-manager`). When the variable is
/// missing, `google-secret-manager` is the default so production
/// deployments always reach real Secret Manager. Local `make up` runs
/// set `CREDENTIAL_STORE_BACKEND=in-memory` to avoid needing a live
/// project.
pub async fn build_credential_store(
    project_id: &str,
) -> Result<Arc<dyn CredentialStorePort>, DomainError> {
    let backend = std::env::var("CREDENTIAL_STORE_BACKEND")
        .unwrap_or_else(|_| "google-secret-manager".to_string());
    match backend.as_str() {
        "in-memory" => Ok(Arc::new(InMemoryCredentialStore::new())),
        "google-secret-manager" => {
            Ok(Arc::new(GoogleSecretManagerStore::new(project_id).await?))
        }
        other => Err(DomainError::SecretPayloadError {
            reason: format!(
                "unknown CREDENTIAL_STORE_BACKEND '{other}' (expected 'in-memory' or 'google-secret-manager')"
            ),
        }),
    }
}
