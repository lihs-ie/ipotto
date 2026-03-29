use serde::{Deserialize, Serialize};

use crate::{
    domain::account::{
        AccountCredential, ImapHost, ImapPort, LoginId, LoginPassword, MailAddress, MailCredential,
        MailPassword, TradingPassword,
    },
    errors::DomainError,
};

/// Secret payload used to persist account credentials outside Firestore documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCredentialSecretPayload {
    login_id: String,
    login_password: String,
    trading_password: String,
    mail_address: String,
    mail_password: String,
    imap_host: String,
    imap_port: u16,
}

impl AccountCredentialSecretPayload {
    /// Builds a secret payload from a domain credential.
    pub fn from_domain(credential: &AccountCredential) -> Self {
        Self {
            login_id: credential.login_id().value().to_string(),
            login_password: credential.login_password().value().to_string(),
            trading_password: credential.trading_password().value().to_string(),
            mail_address: credential
                .mail_credential()
                .mail_address()
                .value()
                .to_string(),
            mail_password: credential
                .mail_credential()
                .mail_password()
                .value()
                .to_string(),
            imap_host: credential.mail_credential().imap_host().value().to_string(),
            imap_port: credential.mail_credential().imap_port().value(),
        }
    }

    /// Restores a domain credential from the payload.
    pub fn to_domain(&self) -> Result<AccountCredential, DomainError> {
        AccountCredential::new(
            LoginId::new(self.login_id.clone())?,
            LoginPassword::new(self.login_password.clone())?,
            TradingPassword::new(self.trading_password.clone())?,
            MailCredential::new(
                MailAddress::new(self.mail_address.clone())?,
                MailPassword::new(self.mail_password.clone())?,
                ImapHost::new(self.imap_host.clone())?,
                ImapPort::new(self.imap_port)?,
            )?,
        )
    }
}
