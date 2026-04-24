use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::account::{
        AccountActivation, AccountCredential, ConnectionTestResult, SecuritiesAccount,
        SecuritiesAccountIdentifier, SecuritiesCompany,
    },
    errors::DomainError,
};

/// Firestore document model for `SecuritiesAccount`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecuritiesAccountDocument {
    pub identifier: String,
    pub securities_company: String,
    pub credential_secret_key: String,
    pub is_active: bool,
    pub connection_test_success: Option<bool>,
    pub connection_test_message: Option<String>,
    pub connection_tested_at: Option<DateTime<Utc>>,
}

impl SecuritiesAccountDocument {
    pub fn from_domain(account: &SecuritiesAccount, credential_secret_key: String) -> Self {
        Self {
            identifier: account.identifier().value().to_string(),
            securities_company: account.securities_company().as_str().to_string(),
            credential_secret_key,
            is_active: account.activation().is_active(),
            connection_test_success: account.connection_test().map(|value| value.success()),
            connection_test_message: account
                .connection_test()
                .map(|value| value.message().to_string()),
            connection_tested_at: account.connection_test().map(|value| value.tested_at()),
        }
    }

    pub fn to_domain(
        &self,
        credential: AccountCredential,
    ) -> Result<SecuritiesAccount, DomainError> {
        SecuritiesAccount::reconstruct(
            SecuritiesAccountIdentifier::new(self.identifier.clone())?,
            SecuritiesCompany::new(self.securities_company.clone())?,
            credential,
            AccountActivation::new(self.is_active),
            match (
                self.connection_test_success,
                self.connection_test_message.clone(),
                self.connection_tested_at,
            ) {
                (Some(success), Some(message), Some(tested_at)) => {
                    Some(ConnectionTestResult::new(success, message, tested_at))
                }
                _ => None,
            },
        )
    }
}
