use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use tokio::runtime::Handle;

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

    fn block_on<F, T>(&self, future: F) -> Result<T, DomainError>
    where
        F: core::future::Future<Output = Result<T, DomainError>>,
    {
        Handle::try_current()
            .map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?
            .block_on(future)
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
        let payload = self.block_on(self.credential_store.get(&document.credential_secret_key))?;
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
        self.block_on(self.credential_store.save(&secret_key, &payload))?;
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
            self.block_on(
                self.credential_store
                    .delete(&document.credential_secret_key),
            )?;
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
        let identifiers: Vec<SecuritiesAccountIdentifier> = self
            .documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .keys()
            .cloned()
            .map(SecuritiesAccountIdentifier::new)
            .collect::<Result<_, _>>()?;
        identifiers
            .iter()
            .map(|identifier| self.find_by_id(identifier))
            .collect::<Result<Vec<_>, _>>()
            .map(|accounts| accounts.into_iter().flatten().collect())
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
