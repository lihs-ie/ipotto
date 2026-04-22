use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::{
    AccountActivation, AccountCredential, ConnectionTestResult, SecuritiesAccountIdentifier,
    SecuritiesCompany,
};

/// Securities account aggregate root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecuritiesAccount {
    identifier: SecuritiesAccountIdentifier,
    securities_company: SecuritiesCompany,
    credential: AccountCredential,
    activation: AccountActivation,
    connection_test: Option<ConnectionTestResult>,
}

impl SecuritiesAccount {
    /// Creates a new securities account.
    pub fn create(
        securities_company: SecuritiesCompany,
        credential: AccountCredential,
    ) -> Result<Self, DomainError> {
        Self::reconstruct(
            SecuritiesAccountIdentifier::generate(),
            securities_company,
            credential,
            AccountActivation::new(true),
            None,
        )
    }

    /// Reconstructs a securities account from persisted state.
    pub fn reconstruct(
        identifier: SecuritiesAccountIdentifier,
        securities_company: SecuritiesCompany,
        credential: AccountCredential,
        activation: AccountActivation,
        connection_test: Option<ConnectionTestResult>,
    ) -> Result<Self, DomainError> {
        if credential.login_id().value().is_empty()
            || credential.login_password().value().is_empty()
            || credential.trading_password().value().is_empty()
        {
            return Err(DomainError::IncompleteCredential);
        }
        Ok(Self {
            identifier,
            securities_company,
            credential,
            activation,
            connection_test,
        })
    }

    /// Activates the account.
    pub fn activate(&mut self) {
        self.activation = AccountActivation::new(true);
    }

    /// Deactivates the account.
    pub fn deactivate(&mut self) {
        self.activation = AccountActivation::new(false);
    }

    /// Records a connection test result.
    pub fn record_test_result(&mut self, result: ConnectionTestResult) {
        self.connection_test = Some(result);
    }

    /// Returns the identifier.
    pub fn identifier(&self) -> &SecuritiesAccountIdentifier {
        &self.identifier
    }

    /// Returns the securities company.
    pub fn securities_company(&self) -> SecuritiesCompany {
        self.securities_company
    }

    /// Returns the credential bundle.
    pub fn credential(&self) -> &AccountCredential {
        &self.credential
    }

    /// Returns the activation value.
    pub fn activation(&self) -> AccountActivation {
        self.activation
    }

    /// Returns the latest connection test result.
    pub fn connection_test(&self) -> Option<&ConnectionTestResult> {
        self.connection_test.as_ref()
    }
}

/// Repository contract for securities accounts.
#[async_trait::async_trait]
pub trait SecuritiesAccountRepository: Send + Sync {
    /// Finds an account by identifier.
    async fn find_by_id(
        &self,
        identifier: &SecuritiesAccountIdentifier,
    ) -> Result<Option<SecuritiesAccount>, DomainError>;

    /// Saves an account aggregate.
    async fn save(&self, account: &SecuritiesAccount) -> Result<(), DomainError>;

    /// Deletes an account by identifier.
    async fn delete(&self, identifier: &SecuritiesAccountIdentifier) -> Result<(), DomainError>;

    /// Returns all accounts.
    async fn find_all(&self) -> Result<Vec<SecuritiesAccount>, DomainError>;

    /// Returns all active accounts.
    async fn find_active(&self) -> Result<Vec<SecuritiesAccount>, DomainError>;
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::SecuritiesAccount;
    use crate::domain::account::{
        AccountCredential, ConnectionTestResult, ImapHost, ImapPort, LoginId, LoginPassword,
        MailAddress, MailCredential, MailPassword, SecuritiesCompany, TradingPassword,
    };

    fn build_credential() -> AccountCredential {
        AccountCredential::new(
            LoginId::new("user").expect("login id"),
            LoginPassword::new("password").expect("login password"),
            TradingPassword::new("trade").expect("trading password"),
            MailCredential::new(
                MailAddress::new("user@example.com").expect("mail address"),
                MailPassword::new("mailpass").expect("mail password"),
                ImapHost::new("imap.example.com").expect("imap host"),
                ImapPort::new(993).expect("imap port"),
            )
            .expect("mail credential"),
        )
        .expect("credential")
    }

    #[test]
    fn records_connection_test_result() {
        let mut account = SecuritiesAccount::create(SecuritiesCompany::Rakuten, build_credential())
            .expect("account");
        account.record_test_result(ConnectionTestResult::new(
            true,
            "ok",
            Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0)
                .single()
                .expect("timestamp"),
        ));

        assert!(account.connection_test().is_some());
    }

    #[test]
    fn toggles_activation() {
        let mut account = SecuritiesAccount::create(SecuritiesCompany::Rakuten, build_credential())
            .expect("account");
        account.deactivate();
        assert!(!account.activation().is_active());
        account.activate();
        assert!(account.activation().is_active());
    }
}
