use std::sync::Arc;

use async_trait::async_trait;
use firestore::{path, FirestoreDb};

use crate::{
    acl::secrets::CredentialStorePort,
    domain::account::{
        SecuritiesAccount, SecuritiesAccountIdentifier, SecuritiesAccountRepository,
    },
    errors::DomainError,
    infrastructure::{
        firestore::{
            collections, documents::SecuritiesAccountDocument,
            payloads::AccountCredentialSecretPayload,
        },
        secrets::account_credential_secret_name,
    },
};

/// Production Firestore-backed implementation of
/// [`SecuritiesAccountRepository`]. Stores the non-sensitive document
/// shape in the `securities_accounts` collection and offloads the
/// AES-256 credential payload to the injected [`CredentialStorePort`],
/// which in production resolves to Google Secret Manager.
#[derive(Clone)]
pub struct FirestoreSecuritiesAccountRepository {
    db: Arc<FirestoreDb>,
    credential_store: Arc<dyn CredentialStorePort>,
}

impl core::fmt::Debug for FirestoreSecuritiesAccountRepository {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("FirestoreSecuritiesAccountRepository")
            .finish()
    }
}

impl FirestoreSecuritiesAccountRepository {
    pub fn new(db: Arc<FirestoreDb>, credential_store: Arc<dyn CredentialStorePort>) -> Self {
        Self {
            db,
            credential_store,
        }
    }

    async fn read_credential(
        &self,
        document: &SecuritiesAccountDocument,
    ) -> Result<AccountCredentialSecretPayload, DomainError> {
        let payload = self
            .credential_store
            .get(&document.credential_secret_key)
            .await?;
        serde_json::from_str(&payload).map_err(|error| DomainError::SecretPayloadError {
            reason: error.to_string(),
        })
    }
}

#[async_trait]
impl SecuritiesAccountRepository for FirestoreSecuritiesAccountRepository {
    async fn find_by_id(
        &self,
        identifier: &SecuritiesAccountIdentifier,
    ) -> Result<Option<SecuritiesAccount>, DomainError> {
        let document: Option<SecuritiesAccountDocument> = self
            .db
            .fluent()
            .select()
            .by_id_in(collections::SECURITIES_ACCOUNTS)
            .obj()
            .one(identifier.value().to_string())
            .await
            .map_err(map_firestore_error)?;
        match document {
            Some(document) => {
                let payload = self.read_credential(&document).await?;
                Ok(Some(document.to_domain(payload.to_domain()?)?))
            }
            None => Ok(None),
        }
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
        let document = SecuritiesAccountDocument::from_domain(account, secret_key);
        self.db
            .fluent()
            .update()
            .in_col(collections::SECURITIES_ACCOUNTS)
            .document_id(account.identifier().value())
            .object(&document)
            .execute::<()>()
            .await
            .map_err(map_firestore_error)?;
        Ok(())
    }

    async fn delete(&self, identifier: &SecuritiesAccountIdentifier) -> Result<(), DomainError> {
        let document: Option<SecuritiesAccountDocument> = self
            .db
            .fluent()
            .select()
            .by_id_in(collections::SECURITIES_ACCOUNTS)
            .obj()
            .one(identifier.value().to_string())
            .await
            .map_err(map_firestore_error)?;
        if let Some(document) = document {
            self.credential_store
                .delete(&document.credential_secret_key)
                .await?;
            self.db
                .fluent()
                .delete()
                .from(collections::SECURITIES_ACCOUNTS)
                .document_id(identifier.value())
                .execute()
                .await
                .map_err(map_firestore_error)?;
        }
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<SecuritiesAccount>, DomainError> {
        let documents: Vec<SecuritiesAccountDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::SECURITIES_ACCOUNTS)
            .obj::<SecuritiesAccountDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;

        let mut accounts = Vec::with_capacity(documents.len());
        for document in documents {
            let payload = self.read_credential(&document).await?;
            accounts.push(document.to_domain(payload.to_domain()?)?);
        }
        Ok(accounts)
    }

    async fn find_active(&self) -> Result<Vec<SecuritiesAccount>, DomainError> {
        let documents: Vec<SecuritiesAccountDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::SECURITIES_ACCOUNTS)
            .filter(|q| {
                q.for_all([q
                    .field(path!(SecuritiesAccountDocument::is_active))
                    .eq(true)])
            })
            .obj::<SecuritiesAccountDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;

        let mut accounts = Vec::with_capacity(documents.len());
        for document in documents {
            let payload = self.read_credential(&document).await?;
            accounts.push(document.to_domain(payload.to_domain()?)?);
        }
        Ok(accounts)
    }
}

fn map_firestore_error(error: firestore::errors::FirestoreError) -> DomainError {
    DomainError::FirestoreMappingError {
        reason: error.to_string(),
    }
}
