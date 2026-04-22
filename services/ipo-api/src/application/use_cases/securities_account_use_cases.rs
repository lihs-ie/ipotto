use std::sync::Arc;

use ipo_backend_shared::{
    acl::browser::BrokerBrowserPort,
    domain::account::{
        AccountCredential, ConnectionTestResult, ImapHost, ImapPort, LoginId, LoginPassword,
        MailAddress, MailCredential, MailPassword, SecuritiesAccount, SecuritiesAccountIdentifier,
        SecuritiesAccountRepository, SecuritiesCompany, TradingPassword,
    },
    errors::DomainError,
    logging::{mask_email, mask_prefix},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterSecuritiesAccountInput {
    pub securities_company: String,
    pub login_id: String,
    pub login_password: String,
    pub trading_password: String,
    pub mail_address: String,
    pub mail_password: String,
    pub imap_host: String,
    pub imap_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterSecuritiesAccountOutput {
    pub identifier: String,
    pub securities_company: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSecuritiesAccountInput {
    // Populated from the URL path in the handler; accepted (but ignored)
    // if also provided in the body to preserve compatibility with clients
    // that mirror the identifier.
    #[serde(default)]
    pub account_identifier: String,
    pub login_id: Option<String>,
    pub login_password: Option<String>,
    pub trading_password: Option<String>,
    pub mail_address: Option<String>,
    pub mail_password: Option<String>,
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateSecuritiesAccountOutput {
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestOutput {
    pub success: bool,
    pub message: String,
    pub tested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritiesAccountOutput {
    pub identifier: String,
    pub securities_company: String,
    pub login_id: String,
    pub mail_address: String,
    pub is_active: bool,
    pub connection_test: Option<ConnectionTestOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSecuritiesAccountsOutput {
    pub items: Vec<SecuritiesAccountOutput>,
    pub total_count: usize,
}

pub struct RegisterSecuritiesAccountUseCase {
    repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
}

impl RegisterSecuritiesAccountUseCase {
    pub fn new(repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: RegisterSecuritiesAccountInput,
    ) -> Result<RegisterSecuritiesAccountOutput, DomainError> {
        let account = SecuritiesAccount::create(
            SecuritiesCompany::new(input.securities_company)?,
            build_credential(
                input.login_id,
                input.login_password,
                input.trading_password,
                input.mail_address,
                input.mail_password,
                input.imap_host,
                input.imap_port,
            )?,
        )?;
        self.repository.save(&account).await?;
        Ok(RegisterSecuritiesAccountOutput {
            identifier: account.identifier().value().to_string(),
            securities_company: account.securities_company().as_str().to_string(),
        })
    }
}

pub struct UpdateSecuritiesAccountUseCase {
    repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
}

impl UpdateSecuritiesAccountUseCase {
    pub fn new(repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: UpdateSecuritiesAccountInput,
    ) -> Result<UpdateSecuritiesAccountOutput, DomainError> {
        let identifier = SecuritiesAccountIdentifier::new(input.account_identifier)?;
        let account = self
            .repository
            .find_by_id(&identifier)
            .await?
            .ok_or_else(|| DomainError::NotFound {
                resource: "securities_account".to_string(),
                identifier: identifier.value().to_string(),
            })?;

        let credential = build_credential(
            input
                .login_id
                .unwrap_or_else(|| account.credential().login_id().value().to_string()),
            input
                .login_password
                .unwrap_or_else(|| account.credential().login_password().value().to_string()),
            input
                .trading_password
                .unwrap_or_else(|| account.credential().trading_password().value().to_string()),
            input.mail_address.unwrap_or_else(|| {
                account
                    .credential()
                    .mail_credential()
                    .mail_address()
                    .value()
                    .to_string()
            }),
            input.mail_password.unwrap_or_else(|| {
                account
                    .credential()
                    .mail_credential()
                    .mail_password()
                    .value()
                    .to_string()
            }),
            input.imap_host.unwrap_or_else(|| {
                account
                    .credential()
                    .mail_credential()
                    .imap_host()
                    .value()
                    .to_string()
            }),
            input
                .imap_port
                .unwrap_or_else(|| account.credential().mail_credential().imap_port().value()),
        )?;

        let updated = SecuritiesAccount::reconstruct(
            identifier.clone(),
            account.securities_company(),
            credential,
            account.activation(),
            account.connection_test().cloned(),
        )?;
        self.repository.save(&updated).await?;
        Ok(UpdateSecuritiesAccountOutput {
            identifier: identifier.value().to_string(),
        })
    }
}

pub struct DeleteSecuritiesAccountUseCase {
    repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
}

impl DeleteSecuritiesAccountUseCase {
    pub fn new(repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, account_identifier: &str) -> Result<(), DomainError> {
        let identifier = SecuritiesAccountIdentifier::new(account_identifier)?;
        if self.repository.find_by_id(&identifier).await?.is_none() {
            return Err(DomainError::NotFound {
                resource: "securities_account".to_string(),
                identifier: identifier.value().to_string(),
            });
        }
        self.repository.delete(&identifier).await
    }
}

pub struct TestSecuritiesAccountConnectionUseCase {
    repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
    browser_port: Arc<dyn BrokerBrowserPort>,
}

impl TestSecuritiesAccountConnectionUseCase {
    pub fn new(
        repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
        browser_port: Arc<dyn BrokerBrowserPort>,
    ) -> Self {
        Self {
            repository,
            browser_port,
        }
    }

    pub async fn execute(
        &self,
        account_identifier: &str,
    ) -> Result<ConnectionTestOutput, DomainError> {
        let identifier = SecuritiesAccountIdentifier::new(account_identifier)?;
        let mut account = self
            .repository
            .find_by_id(&identifier)
            .await?
            .ok_or_else(|| DomainError::NotFound {
                resource: "securities_account".to_string(),
                identifier: identifier.value().to_string(),
            })?;

        let result = self
            .browser_port
            .test_connection(account.credential())
            .await?;
        account.record_test_result(result.clone());
        self.repository.save(&account).await?;

        Ok(connection_test_to_output(&result))
    }
}

pub struct ListSecuritiesAccountsUseCase {
    repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
}

impl ListSecuritiesAccountsUseCase {
    pub fn new(repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<ListSecuritiesAccountsOutput, DomainError> {
        let accounts = self.repository.find_all().await?;
        let items = accounts.iter().map(account_to_output).collect::<Vec<_>>();
        Ok(ListSecuritiesAccountsOutput {
            total_count: items.len(),
            items,
        })
    }
}

fn build_credential(
    login_id: String,
    login_password: String,
    trading_password: String,
    mail_address: String,
    mail_password: String,
    imap_host: String,
    imap_port: u16,
) -> Result<AccountCredential, DomainError> {
    AccountCredential::new(
        LoginId::new(login_id)?,
        LoginPassword::new(login_password)?,
        TradingPassword::new(trading_password)?,
        MailCredential::new(
            MailAddress::new(mail_address)?,
            MailPassword::new(mail_password)?,
            ImapHost::new(imap_host)?,
            ImapPort::new(imap_port)?,
        )?,
    )
}

fn account_to_output(account: &SecuritiesAccount) -> SecuritiesAccountOutput {
    SecuritiesAccountOutput {
        identifier: account.identifier().value().to_string(),
        securities_company: account.securities_company().as_str().to_string(),
        login_id: mask_prefix(account.credential().login_id().value()),
        mail_address: mask_email(
            account
                .credential()
                .mail_credential()
                .mail_address()
                .value(),
        ),
        is_active: account.activation().is_active(),
        connection_test: account.connection_test().map(connection_test_to_output),
    }
}

fn connection_test_to_output(result: &ConnectionTestResult) -> ConnectionTestOutput {
    ConnectionTestOutput {
        success: result.success(),
        message: result.message().to_string(),
        tested_at: result.tested_at().to_rfc3339(),
    }
}
