use std::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::{LoginId, LoginPassword, MailCredential, TradingPassword};

/// Credential bundle for a securities account.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCredential {
    login_id: LoginId,
    login_password: LoginPassword,
    trading_password: TradingPassword,
    mail_credential: MailCredential,
}

impl AccountCredential {
    /// Creates an account credential bundle.
    pub fn new(
        login_id: LoginId,
        login_password: LoginPassword,
        trading_password: TradingPassword,
        mail_credential: MailCredential,
    ) -> Result<Self, DomainError> {
        if login_id.value().is_empty()
            || login_password.value().is_empty()
            || trading_password.value().is_empty()
        {
            return Err(DomainError::IncompleteCredential);
        }
        Ok(Self {
            login_id,
            login_password,
            trading_password,
            mail_credential,
        })
    }

    /// Returns the login ID.
    pub fn login_id(&self) -> &LoginId {
        &self.login_id
    }

    /// Returns the login password.
    pub fn login_password(&self) -> &LoginPassword {
        &self.login_password
    }

    /// Returns the trading password.
    pub fn trading_password(&self) -> &TradingPassword {
        &self.trading_password
    }

    /// Returns the mail credential.
    pub fn mail_credential(&self) -> &MailCredential {
        &self.mail_credential
    }
}

impl fmt::Debug for AccountCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AccountCredential")
            .field("login_id", &"********")
            .field("login_password", &"********")
            .field("trading_password", &"********")
            .field("mail_credential", &self.mail_credential)
            .finish()
    }
}
