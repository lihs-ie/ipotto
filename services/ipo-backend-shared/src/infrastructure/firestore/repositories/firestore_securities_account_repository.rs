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
pub struct FirestoreSecuritiesAccountRepository<S>
where
    S: CredentialStorePort,
{
    documents: Arc<Mutex<BTreeMap<String, SecuritiesAccountDocument>>>,
    credential_store: S,
}

impl<S> core::fmt::Debug for FirestoreSecuritiesAccountRepository<S>
where
    S: CredentialStorePort,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("FirestoreSecuritiesAccountRepository")
            .finish()
    }
}

impl<S> FirestoreSecuritiesAccountRepository<S>
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

impl<S> SecuritiesAccountRepository for FirestoreSecuritiesAccountRepository<S>
where
    S: CredentialStorePort,
{
    fn find_by_id(
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
        let payload = self.credential_store.get(&document.credential_secret_key)?;
        let payload: AccountCredentialSecretPayload =
            serde_json::from_str(&payload).map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?;
        Ok(Some(document.to_domain(payload.to_domain()?)?))
    }

    fn save(&self, account: &SecuritiesAccount) -> Result<(), DomainError> {
        let secret_key = account_credential_secret_name(account.identifier());
        let payload = serde_json::to_string(&AccountCredentialSecretPayload::from_domain(
            account.credential(),
        ))
        .map_err(|error| DomainError::SecretPayloadError {
            reason: error.to_string(),
        })?;
        self.credential_store.save(&secret_key, &payload)?;
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

    fn delete(&self, identifier: &SecuritiesAccountIdentifier) -> Result<(), DomainError> {
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
                .delete(&document.credential_secret_key)?;
            self.documents
                .lock()
                .map_err(|error| DomainError::FirestoreMappingError {
                    reason: error.to_string(),
                })?
                .remove(identifier.value());
        }
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<SecuritiesAccount>, DomainError> {
        let documents: Vec<SecuritiesAccountDocument> = self
            .documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .cloned()
            .collect();

        documents
            .into_iter()
            .map(|document| {
                let payload = self.credential_store.get(&document.credential_secret_key)?;
                let payload: AccountCredentialSecretPayload = serde_json::from_str(&payload)
                    .map_err(|error| DomainError::SecretPayloadError {
                        reason: error.to_string(),
                    })?;
                document.to_domain(payload.to_domain()?)
            })
            .collect()
    }

    fn find_active(&self) -> Result<Vec<SecuritiesAccount>, DomainError> {
        self.find_all().map(|accounts| {
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

    use super::FirestoreSecuritiesAccountRepository;
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

    #[test]
    fn saves_accounts_and_removes_secret_on_delete() {
        let credential_store = InMemoryCredentialStore::new();
        let repository = FirestoreSecuritiesAccountRepository::new(credential_store.clone());
        let active = build_account(true);
        let inactive = build_account(false);

        repository.save(&active).expect("save active");
        repository.save(&inactive).expect("save inactive");

        assert_eq!(repository.find_all().expect("find all").len(), 2);
        assert_eq!(repository.find_active().expect("find active").len(), 1);
        assert!(credential_store
            .exists(&account_credential_secret_name(active.identifier()))
            .expect("secret exists"));

        repository.delete(active.identifier()).expect("delete");
        assert!(repository
            .find_by_id(active.identifier())
            .expect("find")
            .is_none());
        assert!(!credential_store
            .exists(&account_credential_secret_name(active.identifier()))
            .expect("secret removed"));
    }
}
