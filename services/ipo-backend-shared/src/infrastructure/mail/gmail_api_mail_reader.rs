use std::time::Duration;

use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

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
            .get(&gmail_oauth_secret_name(&self.account_identifier))?;
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

    async fn fetch_latest_message_body(
        &self,
        access_token: &str,
        received_after: DateTime<Utc>,
    ) -> Result<String, DomainError> {
        let message_list = self
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
            .json::<GmailMessageListResponse>()
            .await
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?;
        let message_id = message_list
            .messages
            .and_then(|messages| messages.into_iter().next())
            .map(|message| message.id)
            .ok_or_else(|| DomainError::MailParseError {
                reason: "gmail message not found".to_string(),
            })?;

        let message = self
            .client
            .get(format!(
                "https://gmail.googleapis.com/gmail/v1/users/me/messages/{message_id}"
            ))
            .bearer_auth(access_token)
            .query(&[("format", "full")])
            .send()
            .await
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?
            .error_for_status()
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?
            .json::<GmailMessageResponse>()
            .await
            .map_err(|error| DomainError::GmailApiError {
                reason: error.to_string(),
            })?;

        if let Some(decoded) = message.decoded_body()? {
            return Ok(decoded);
        }
        message.snippet.ok_or_else(|| DomainError::MailParseError {
            reason: "gmail message body not found".to_string(),
        })
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
        timeout_seconds: u32,
    ) -> Result<ImageAuthenticationKeyword, DomainError> {
        let future = async {
            let access_token = self.refresh_access_token().await?;
            let response = self
                .fetch_latest_message_body(&access_token, received_after)
                .await?;
            parse_image_authentication_keyword(&response)
        };
        timeout(Duration::from_secs(u64::from(timeout_seconds)), future)
            .await
            .map_err(|_| DomainError::MailRetrievalTimeout)?
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GmailMessageListResponse {
    messages: Option<Vec<GmailMessageReference>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GmailMessageReference {
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GmailMessageResponse {
    snippet: Option<String>,
    payload: Option<GmailMessagePayload>,
}

impl GmailMessageResponse {
    fn decoded_body(&self) -> Result<Option<String>, DomainError> {
        self.payload
            .as_ref()
            .map(GmailMessagePayload::decoded_body)
            .transpose()
            .map(Option::flatten)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct GmailMessagePayload {
    body: Option<GmailMessageBody>,
    parts: Option<Vec<GmailMessagePayload>>,
}

impl GmailMessagePayload {
    fn decoded_body(&self) -> Result<Option<String>, DomainError> {
        if let Some(body) = &self.body {
            if let Some(decoded) = body.decode()? {
                if !decoded.trim().is_empty() {
                    return Ok(Some(decoded));
                }
            }
        }
        if let Some(parts) = &self.parts {
            for part in parts {
                if let Some(decoded) = part.decoded_body()? {
                    if !decoded.trim().is_empty() {
                        return Ok(Some(decoded));
                    }
                }
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct GmailMessageBody {
    data: Option<String>,
}

impl GmailMessageBody {
    fn decode(&self) -> Result<Option<String>, DomainError> {
        let Some(data) = &self.data else {
            return Ok(None);
        };
        let bytes = URL_SAFE_NO_PAD
            .decode(data)
            .map_err(|error| DomainError::MailParseError {
                reason: error.to_string(),
            })?;
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|error| DomainError::MailParseError {
                reason: error.to_string(),
            })
    }
}
