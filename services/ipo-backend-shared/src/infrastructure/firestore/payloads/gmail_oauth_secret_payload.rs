use serde::{Deserialize, Serialize};

/// Secret payload used for Gmail OAuth refresh-token flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GmailOauthSecretPayload {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    token_uri: String,
}

impl GmailOauthSecretPayload {
    /// Creates a Gmail OAuth payload.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        refresh_token: impl Into<String>,
        token_uri: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            refresh_token: refresh_token.into(),
            token_uri: token_uri.into(),
        }
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
    pub fn token_uri(&self) -> &str {
        &self.token_uri
    }
}
