use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::time::sleep;

use crate::{
    acl::mail::MailReaderPort,
    domain::account::{ImageAuthenticationKeyword, MailCredential},
    errors::DomainError,
    infrastructure::mail::{parse_image_authentication_keyword, MailPollingConfig},
};

/// IMAP-based mail reader with injectable message supplier.
#[derive(Clone)]
pub struct ImapMailReader<F>
where
    F: Fn(&MailCredential, DateTime<Utc>) -> Result<Option<String>, DomainError> + Send + Sync,
{
    polling_config: MailPollingConfig,
    message_supplier: F,
}

impl<F> core::fmt::Debug for ImapMailReader<F>
where
    F: Fn(&MailCredential, DateTime<Utc>) -> Result<Option<String>, DomainError> + Send + Sync,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ImapMailReader")
            .field("polling_config", &self.polling_config)
            .finish()
    }
}

impl<F> ImapMailReader<F>
where
    F: Fn(&MailCredential, DateTime<Utc>) -> Result<Option<String>, DomainError> + Send + Sync,
{
    /// Creates an IMAP mail reader.
    pub fn new(polling_config: MailPollingConfig, message_supplier: F) -> Self {
        Self {
            polling_config,
            message_supplier,
        }
    }
}

#[async_trait]
impl<F> MailReaderPort for ImapMailReader<F>
where
    F: Fn(&MailCredential, DateTime<Utc>) -> Result<Option<String>, DomainError> + Send + Sync,
{
    async fn fetch_image_authentication_keywords(
        &self,
        mail_credential: &MailCredential,
        received_after: DateTime<Utc>,
        timeout_seconds: u32,
    ) -> Result<ImageAuthenticationKeyword, DomainError> {
        let limit = timeout_seconds.min(self.polling_config.max_timeout_seconds());
        let mut elapsed = 0;
        while elapsed <= limit {
            if let Some(body) = (self.message_supplier)(mail_credential, received_after)? {
                return parse_image_authentication_keyword(&body);
            }
            if elapsed >= limit {
                break;
            }
            let remaining = limit - elapsed;
            let sleep_seconds = self.polling_config.polling_interval_seconds().min(remaining);
            sleep(Duration::from_secs(sleep_seconds.into())).await;
            elapsed += sleep_seconds;
        }
        Err(DomainError::MailRetrievalTimeout)
    }
}
