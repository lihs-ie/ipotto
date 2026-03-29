use std::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::{ImapHost, ImapPort, MailAddress, MailPassword};

/// Mail credential used to fetch image authentication keywords.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailCredential {
    mail_address: MailAddress,
    mail_password: MailPassword,
    imap_host: ImapHost,
    imap_port: ImapPort,
}

impl MailCredential {
    /// Creates a mail credential.
    pub fn new(
        mail_address: MailAddress,
        mail_password: MailPassword,
        imap_host: ImapHost,
        imap_port: ImapPort,
    ) -> Result<Self, DomainError> {
        if mail_address.value().is_empty() || imap_host.value().is_empty() || imap_port.value() == 0
        {
            return Err(DomainError::IncompleteCredential);
        }
        Ok(Self {
            mail_address,
            mail_password,
            imap_host,
            imap_port,
        })
    }

    /// Returns the mail address.
    pub fn mail_address(&self) -> &MailAddress {
        &self.mail_address
    }

    /// Returns the mail password.
    pub fn mail_password(&self) -> &MailPassword {
        &self.mail_password
    }

    /// Returns the IMAP host.
    pub fn imap_host(&self) -> &ImapHost {
        &self.imap_host
    }

    /// Returns the IMAP port.
    pub fn imap_port(&self) -> ImapPort {
        self.imap_port
    }
}

impl fmt::Debug for MailCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MailCredential")
            .field("mail_address", &"********")
            .field("mail_password", &"********")
            .field("imap_host", &self.imap_host)
            .field("imap_port", &self.imap_port)
            .finish()
    }
}
