use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    acl::{mail::MailReaderPort, secrets::CredentialStorePort},
    domain::account::{ImageAuthenticationKeyword, MailCredential, SecuritiesAccountIdentifier},
    errors::DomainError,
    infrastructure::{
        firestore::payloads::GmailOauthSecretPayload, mail::parse_image_authentication_keyword,
        secrets::gmail_oauth_secret_name,
    },
};

/// Gmail API reader using a Secret Manager-backed OAuth refresh payload.
#[derive(Clone)]
pub struct GmailApiMailReader<S>
where
    S: CredentialStorePort,
{
    client: Client,
    credential_store: S,
    account_identifier: SecuritiesAccountIdentifier,
}

impl<S> core::fmt::Debug for GmailApiMailReader<S>
where
    S: CredentialStorePort,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("GmailApiMailReader")
            .field("account_identifier", &self.account_identifier.value())
            .finish()
    }
}

impl<S> GmailApiMailReader<S>
where
    S: CredentialStorePort,
{
    /// Creates a Gmail API mail reader.
    pub fn new(
        client: Client,
        credential_store: S,
        account_identifier: SecuritiesAccountIdentifier,
    ) -> Self {
        Self {
            client,
            credential_store,
            account_identifier,
        }
    }

    /// Builds the Gmail search query.
    pub fn build_search_query(received_after: DateTime<Utc>) -> String {
        format!(
            "from:rakuten subject:認証 after:{}",
            received_after.timestamp()
        )
    }

    async fn refresh_access_token(&self) -> Result<String, DomainError> {
        let payload = self
            .credential_store
            .get(&gmail_oauth_secret_name(&self.account_identifier))
            .await?;
        let payload: GmailOauthSecretPayload =
            serde_json::from_str(&payload).map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?;
        let response = self
            .client
            .post(payload.token_uri())
            .form(&[
                ("client_id", payload.client_id()),
                ("client_secret", payload.client_secret()),
                ("refresh_token", payload.refresh_token()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?
            .error_for_status()
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?
            .json::<TokenResponse>()
            .await
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?;
        Ok(response.access_token)
    }
}

#[async_trait]
impl<S> MailReaderPort for GmailApiMailReader<S>
where
    S: CredentialStorePort,
{
    async fn fetch_image_authentication_keywords(
        &self,
        _mail_credential: &MailCredential,
        received_after: DateTime<Utc>,
        _timeout_seconds: u32,
    ) -> Result<ImageAuthenticationKeyword, DomainError> {
        let access_token = self.refresh_access_token().await?;
        let response = self
            .client
            .get("https://gmail.googleapis.com/gmail/v1/users/me/messages")
            .bearer_auth(access_token)
            .query(&[("q", Self::build_search_query(received_after))])
            .send()
            .await
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?
            .error_for_status()
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?
            .text()
            .await
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?;
        parse_image_authentication_keyword(&response)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TokenResponse {
    access_token: String,
}
