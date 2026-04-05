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
            let sleep_seconds = self
                .polling_config
                .polling_interval_seconds()
                .min(remaining);
            sleep(Duration::from_secs(sleep_seconds.into())).await;
            elapsed += sleep_seconds;
        }
        Err(DomainError::MailRetrievalTimeout)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::{TimeZone, Utc};

    use super::ImapMailReader;
    use crate::{
        acl::mail::MailReaderPort,
        domain::account::{ImapHost, ImapPort, MailAddress, MailCredential, MailPassword},
        errors::DomainError,
        infrastructure::mail::MailPollingConfig,
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
    async fn returns_keywords_from_supplied_mail_body() {
        let observed_received_after = Arc::new(Mutex::new(None));
        let capture = observed_received_after.clone();
        let reader = ImapMailReader::new(
            MailPollingConfig::new(1, 5),
            move |_credential: &MailCredential, received_after| {
                *capture.lock().expect("capture lock") = Some(received_after);
                Ok(Some("みかん + りんご".to_string()))
            },
        );
        let received_after = Utc
            .with_ymd_and_hms(2026, 4, 1, 9, 0, 0)
            .single()
            .expect("received after");

        let keyword = reader
            .fetch_image_authentication_keywords(&build_mail_credential(), received_after, 5)
            .await
            .expect("keyword");

        assert_eq!(keyword.first_keyword(), "みかん");
        assert_eq!(keyword.second_keyword(), "りんご");
        assert_eq!(
            *observed_received_after.lock().expect("observed lock"),
            Some(received_after)
        );
    }

    #[tokio::test]
    async fn returns_timeout_when_no_mail_is_found() {
        let reader = ImapMailReader::new(
            MailPollingConfig::new(1, 0),
            |_credential: &MailCredential, _received_after| Ok::<Option<String>, DomainError>(None),
        );

        let result = reader
            .fetch_image_authentication_keywords(&build_mail_credential(), Utc::now(), 0)
            .await;

        assert!(matches!(result, Err(DomainError::MailRetrievalTimeout)));
    }
}
