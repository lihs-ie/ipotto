use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    domain::account::{ImageAuthenticationKeyword, MailCredential},
    errors::DomainError,
};

/// Mail reader port for image authentication mail retrieval.
#[async_trait]
pub trait MailReaderPort: Send + Sync {
    /// Fetches image authentication keywords after a given timestamp.
    async fn fetch_image_authentication_keywords(
        &self,
        mail_credential: &MailCredential,
        received_after: DateTime<Utc>,
        timeout_seconds: u32,
    ) -> Result<ImageAuthenticationKeyword, DomainError>;
}
