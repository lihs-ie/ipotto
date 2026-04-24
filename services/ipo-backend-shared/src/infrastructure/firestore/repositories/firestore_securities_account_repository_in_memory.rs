use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::{
    acl::secrets::CredentialStorePort,
    domain::account::{
        SecuritiesAccount, SecuritiesAccountIdentifier, SecuritiesAccountRepository,
    },
    errors::DomainError,
    infrastructure::{
        firestore::{
            documents::SecuritiesAccountDocument, payloads::AccountCredentialSecretPayload,
        },
        secrets::account_credential_secret_name,
    },
};

/// Concrete securities account repository with Firestore-oriented document mapping.
#[derive(Clone)]
pub struct FirestoreSecuritiesAccountRepositoryInMemory<S>
where
    S: CredentialStorePort,
{
    documents: Arc<Mutex<BTreeMap<String, SecuritiesAccountDocument>>>,
    credential_store: S,
}

impl<S> core::fmt::Debug for FirestoreSecuritiesAccountRepositoryInMemory<S>
where
    S: CredentialStorePort,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("FirestoreSecuritiesAccountRepositoryInMemory")
            .finish()
    }
}

impl<S> FirestoreSecuritiesAccountRepositoryInMemory<S>
where
    S: CredentialStorePort,
{
    pub fn new(credential_store: S) -> Self {
        Self {
            documents: Arc::new(Mutex::new(BTreeMap::new())),
            credential_store,
        }
    }
}

#[async_trait::async_trait]
impl<S> SecuritiesAccountRepository for FirestoreSecuritiesAccountRepositoryInMemory<S>
where
    S: CredentialStorePort + 'static,
{
    async fn find_by_id(
        &self,
        identifier: &SecuritiesAccountIdentifier,
    ) -> Result<Option<SecuritiesAccount>, DomainError> {
        let document = self
            .documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .get(identifier.value())
            .cloned();
        let Some(document) = document else {
            return Ok(None);
        };
        let payload = self
            .credential_store
            .get(&document.credential_secret_key)
            .await?;
        let payload: AccountCredentialSecretPayload =
            serde_json::from_str(&payload).map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?;
        Ok(Some(document.to_domain(payload.to_domain()?)?))
    }

    async fn save(&self, account: &SecuritiesAccount) -> Result<(), DomainError> {
        let secret_key = account_credential_secret_name(account.identifier());
        let payload = serde_json::to_string(&AccountCredentialSecretPayload::from_domain(
            account.credential(),
        ))
        .map_err(|error| DomainError::SecretPayloadError {
            reason: error.to_string(),
        })?;
        self.credential_store.save(&secret_key, &payload).await?;
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .insert(
                account.identifier().value().to_string(),
                SecuritiesAccountDocument::from_domain(account, secret_key),
            );
        Ok(())
    }

    async fn delete(&self, identifier: &SecuritiesAccountIdentifier) -> Result<(), DomainError> {
        let document = self
            .documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .get(identifier.value())
            .cloned();
        if let Some(document) = document {
            self.credential_store
                .delete(&document.credential_secret_key)
                .await?;
            self.documents
                .lock()
                .map_err(|error| DomainError::FirestoreMappingError {
                    reason: error.to_string(),
                })?
                .remove(identifier.value());
        }
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<SecuritiesAccount>, DomainError> {
        let documents: Vec<SecuritiesAccountDocument> = self
            .documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .cloned()
            .collect();

        let mut accounts = Vec::with_capacity(documents.len());
        for document in documents {
            let payload = self
                .credential_store
                .get(&document.credential_secret_key)
                .await?;
            let payload: AccountCredentialSecretPayload =
                serde_json::from_str(&payload).map_err(|error| {
                    DomainError::SecretPayloadError {
                        reason: error.to_string(),
                    }
                })?;
            accounts.push(document.to_domain(payload.to_domain()?)?);
        }
        Ok(accounts)
    }

    async fn find_active(&self) -> Result<Vec<SecuritiesAccount>, DomainError> {
        self.find_all().await.map(|accounts| {
            accounts
                .into_iter()
                .filter(|account| account.activation().is_active())
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::FirestoreSecuritiesAccountRepositoryInMemory;
    use crate::{
        acl::secrets::CredentialStorePort,
        domain::account::{
            AccountCredential, ImapHost, ImapPort, LoginId, LoginPassword, MailAddress,
            MailCredential, MailPassword, SecuritiesAccount, SecuritiesAccountRepository,
            SecuritiesCompany, TradingPassword,
        },
        infrastructure::secrets::{account_credential_secret_name, InMemoryCredentialStore},
    };

    fn build_account(active: bool) -> SecuritiesAccount {
        let mut account = SecuritiesAccount::create(
            SecuritiesCompany::Rakuten,
            AccountCredential::new(
                LoginId::new("login").expect("login id"),
                LoginPassword::new("password").expect("login password"),
                TradingPassword::new("1234").expect("trading password"),
                MailCredential::new(
                    MailAddress::new("test@example.com").expect("mail"),
                    MailPassword::new("mail-password").expect("mail password"),
                    ImapHost::new("imap.example.com").expect("host"),
                    ImapPort::new(993).expect("port"),
                )
                .expect("mail credential"),
            )
            .expect("credential"),
        )
        .expect("account");
        if !active {
            account.deactivate();
        }
        account.record_test_result(crate::domain::account::ConnectionTestResult::new(
            true,
            "ok",
            Utc.with_ymd_and_hms(2026, 4, 1, 9, 0, 0)
                .single()
                .expect("tested at"),
        ));
        account
    }

    #[tokio::test]
    async fn saves_accounts_and_removes_secret_on_delete() {
        let credential_store = InMemoryCredentialStore::new();
        let repository =
            FirestoreSecuritiesAccountRepositoryInMemory::new(credential_store.clone());
        let active = build_account(true);
        let inactive = build_account(false);

        repository.save(&active).await.expect("save active");
        repository.save(&inactive).await.expect("save inactive");

        assert_eq!(repository.find_all().await.expect("find all").len(), 2);
        assert_eq!(
            repository.find_active().await.expect("find active").len(),
            1
        );
        assert!(credential_store
            .exists(&account_credential_secret_name(active.identifier()))
            .await
            .expect("secret exists"));

        repository
            .delete(active.identifier())
            .await
            .expect("delete");
        assert!(repository
            .find_by_id(active.identifier())
            .await
            .expect("find")
            .is_none());
        assert!(!credential_store
            .exists(&account_credential_secret_name(active.identifier()))
            .await
            .expect("secret removed"));
    }
}
