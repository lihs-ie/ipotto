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
    api_base_url: String,
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
        Self::new_with_api_base_url(
            client,
            credential_store,
            account_identifier,
            "https://gmail.googleapis.com",
        )
    }

    /// Creates a Gmail API mail reader with a custom API base URL.
    pub fn new_with_api_base_url(
        client: Client,
        credential_store: S,
        account_identifier: SecuritiesAccountIdentifier,
        api_base_url: impl Into<String>,
    ) -> Self {
        Self {
            client,
            credential_store,
            account_identifier,
            api_base_url: api_base_url.into(),
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
            .get(format!("{}/gmail/v1/users/me/messages", self.api_base_url))
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
                "{}/gmail/v1/users/me/messages/{message_id}",
                self.api_base_url
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

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::GmailApiMailReader;
    use crate::{
        acl::{mail::MailReaderPort, secrets::CredentialStorePort},
        domain::account::{
            ImapHost, ImapPort, MailAddress, MailCredential, MailPassword,
            SecuritiesAccountIdentifier,
        },
        infrastructure::{
            firestore::payloads::GmailOauthSecretPayload,
            secrets::{gmail_oauth_secret_name, InMemoryCredentialStore},
        },
    };

    fn build_mail_credential() -> MailCredential {
        MailCredential::new(
            MailAddress::new("test@example.com").expect("mail"),
            MailPassword::new("mail-password").expect("mail password"),
            ImapHost::new("imap.example.com").expect("host"),
            ImapPort::new(993).expect("port"),
        )
        .expect("mail credential")
    }

    #[tokio::test]
    async fn fetches_keywords_via_gmail_api_flow() {
        let server = MockServer::start().await;
        let account_identifier = SecuritiesAccountIdentifier::generate();
        let store = InMemoryCredentialStore::new();
        let payload = GmailOauthSecretPayload::new(
            "client-id",
            "client-secret",
            "refresh-token",
            format!("{}/oauth/token", server.uri()),
        );
        store
            .save(
                &gmail_oauth_secret_name(&account_identifier),
                &serde_json::to_string(&payload).expect("payload"),
            )
            .expect("save oauth payload");

        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "access-token"
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "messages": [{"id": "message-1"}]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/messages/message-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "payload": {
                    "body": {
                        "data": URL_SAFE_NO_PAD.encode("みかん + りんご")
                    }
                }
            })))
            .mount(&server)
            .await;

        let reader = GmailApiMailReader::new_with_api_base_url(
            reqwest::Client::new(),
            store,
            account_identifier,
            server.uri(),
        );
        let keyword = reader
            .fetch_image_authentication_keywords(
                &build_mail_credential(),
                Utc.with_ymd_and_hms(2026, 4, 1, 9, 0, 0)
                    .single()
                    .expect("received after"),
                5,
            )
            .await
            .expect("keyword");

        assert_eq!(keyword.first_keyword(), "みかん");
        assert_eq!(keyword.second_keyword(), "りんご");
    }
}
