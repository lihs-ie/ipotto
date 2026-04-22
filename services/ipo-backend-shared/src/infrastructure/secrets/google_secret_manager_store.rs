use std::sync::Arc;

use async_trait::async_trait;
use gcloud_sdk::{
    google::cloud::secretmanager::v1::{
        secret_manager_service_client::SecretManagerServiceClient, AccessSecretVersionRequest,
        AddSecretVersionRequest, CreateSecretRequest, DeleteSecretRequest, GetSecretRequest,
        Replication, Secret,
    },
    proto_ext::secretmanager::SecretPayload,
    tonic, GoogleApi, GoogleAuthMiddleware,
};

use crate::{acl::secrets::CredentialStorePort, errors::DomainError};

const SECRET_MANAGER_URL: &str = "https://secretmanager.googleapis.com";

/// Production-grade [`CredentialStorePort`] implementation backed by
/// Google Secret Manager. Uses the `gcloud-sdk` GoogleApi wrapper so that
/// ADC / workload identity / metadata-server auth is resolved
/// automatically when the service runs on Cloud Run.
///
/// Secrets are identified by their short *secret ID* (e.g.
/// `rakuten-account-cred-01JABCXYZ`) and stored under
/// `projects/{project_id}/secrets/{secret_id}` with the `latest`
/// alias always resolving to the most recent version.
#[derive(Clone)]
pub struct GoogleSecretManagerStore {
    project_id: String,
    client: Arc<GoogleApi<SecretManagerServiceClient<GoogleAuthMiddleware>>>,
}

impl core::fmt::Debug for GoogleSecretManagerStore {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("GoogleSecretManagerStore")
            .field("project_id", &self.project_id)
            .finish()
    }
}

impl GoogleSecretManagerStore {
    /// Builds a Secret Manager client authenticated via ADC / workload
    /// identity. `project_id` is the GCP project that owns the secrets
    /// (Secret Manager does not ship an emulator, so local/CI tests use
    /// `InMemoryCredentialStore` instead).
    pub async fn new(project_id: impl Into<String>) -> Result<Self, DomainError> {
        let client =
            GoogleApi::from_function(SecretManagerServiceClient::new, SECRET_MANAGER_URL, None)
                .await
                .map_err(|error| DomainError::SecretPayloadError {
                    reason: format!("failed to build Secret Manager client: {error}"),
                })?;
        Ok(Self {
            project_id: project_id.into(),
            client: Arc::new(client),
        })
    }

    async fn ensure_secret_exists(&self, key: &str) -> Result<(), DomainError> {
        let existing = self
            .client
            .get()
            .get_secret(tonic::Request::new(GetSecretRequest {
                name: secret_path(&self.project_id, key),
            }))
            .await;
        match existing {
            Ok(_) => Ok(()),
            Err(status) if status.code() == tonic::Code::NotFound => {
                self.client
                    .get()
                    .create_secret(tonic::Request::new(CreateSecretRequest {
                        parent: project_path(&self.project_id),
                        secret_id: key.to_string(),
                        secret: Some(Secret {
                            replication: Some(Replication {
                                replication: Some(
                                    gcloud_sdk::google::cloud::secretmanager::v1::replication::Replication::Automatic(
                                        gcloud_sdk::google::cloud::secretmanager::v1::replication::Automatic::default(),
                                    ),
                                ),
                            }),
                            ..Default::default()
                        }),
                    }))
                    .await
                    .map_err(|error| DomainError::SecretPayloadError {
                        reason: format!("failed to create secret {key}: {error}"),
                    })?;
                Ok(())
            }
            Err(status) => Err(DomainError::SecretPayloadError {
                reason: format!("failed to inspect secret {key}: {status}"),
            }),
        }
    }
}

#[async_trait]
impl CredentialStorePort for GoogleSecretManagerStore {
    async fn save(&self, key: &str, value: &str) -> Result<(), DomainError> {
        self.ensure_secret_exists(key).await?;
        self.client
            .get()
            .add_secret_version(tonic::Request::new(AddSecretVersionRequest {
                parent: secret_path(&self.project_id, key),
                payload: Some(SecretPayload {
                    data: gcloud_sdk::SecretValue::new(value.as_bytes().to_vec()),
                    data_crc32c: None,
                }),
            }))
            .await
            .map_err(|error| DomainError::SecretPayloadError {
                reason: format!("failed to add secret version {key}: {error}"),
            })?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<String, DomainError> {
        let response = self
            .client
            .get()
            .access_secret_version(tonic::Request::new(AccessSecretVersionRequest {
                name: latest_version_path(&self.project_id, key),
            }))
            .await
            .map_err(|error| DomainError::SecretPayloadError {
                reason: if error.code() == tonic::Code::NotFound {
                    format!("secret not found: {key}")
                } else {
                    format!("failed to access secret {key}: {error}")
                },
            })?;
        let payload =
            response
                .into_inner()
                .payload
                .ok_or_else(|| DomainError::SecretPayloadError {
                    reason: format!("Secret Manager returned no payload for {key}"),
                })?;
        String::from_utf8(payload.data.as_sensitive_bytes().to_vec()).map_err(|error| {
            DomainError::SecretPayloadError {
                reason: format!("secret {key} is not valid UTF-8: {error}"),
            }
        })
    }

    async fn delete(&self, key: &str) -> Result<(), DomainError> {
        match self
            .client
            .get()
            .delete_secret(tonic::Request::new(DeleteSecretRequest {
                name: secret_path(&self.project_id, key),
                etag: String::new(),
            }))
            .await
        {
            Ok(_) => Ok(()),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(()),
            Err(status) => Err(DomainError::SecretPayloadError {
                reason: format!("failed to delete secret {key}: {status}"),
            }),
        }
    }

    async fn exists(&self, key: &str) -> Result<bool, DomainError> {
        match self
            .client
            .get()
            .get_secret(tonic::Request::new(GetSecretRequest {
                name: secret_path(&self.project_id, key),
            }))
            .await
        {
            Ok(_) => Ok(true),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(false),
            Err(status) => Err(DomainError::SecretPayloadError {
                reason: format!("failed to check secret {key}: {status}"),
            }),
        }
    }
}

fn project_path(project_id: &str) -> String {
    format!("projects/{project_id}")
}

fn secret_path(project_id: &str, key: &str) -> String {
    format!("projects/{project_id}/secrets/{key}")
}

fn latest_version_path(project_id: &str, key: &str) -> String {
    format!("projects/{project_id}/secrets/{key}/versions/latest")
}

#[cfg(test)]
mod tests {
    use super::{latest_version_path, project_path, secret_path};

    #[test]
    fn project_path_formats_resource_name() {
        assert_eq!(project_path("ipotto-dev"), "projects/ipotto-dev");
    }

    #[test]
    fn secret_path_formats_resource_name() {
        assert_eq!(
            secret_path("ipotto-dev", "api-key-primary"),
            "projects/ipotto-dev/secrets/api-key-primary"
        );
    }

    #[test]
    fn latest_version_path_targets_latest_alias() {
        assert_eq!(
            latest_version_path("ipotto-dev", "api-key-primary"),
            "projects/ipotto-dev/secrets/api-key-primary/versions/latest"
        );
    }
}
