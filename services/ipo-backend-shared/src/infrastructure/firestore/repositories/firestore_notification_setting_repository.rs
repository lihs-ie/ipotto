use std::sync::Arc;

use async_trait::async_trait;
use firestore::FirestoreDb;
use serde::{Deserialize, Serialize};

use crate::{
    domain::notification::{
        NotificationSetting, NotificationSettingIdentifier, NotificationSettingRepository,
    },
    errors::DomainError,
    infrastructure::firestore::{
        collections,
        documents::{NotificationChannelDocument, NotificationSettingDocument},
    },
};

/// Combined document storing the `NotificationSetting` aggregate and
/// its list of channels inside a single Firestore document. Using a
/// single document avoids the round-trip cost of a subcollection and
/// keeps the aggregate consistent under Firestore's strong-consistency
/// guarantees for individual document writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationSettingAggregateDocument {
    #[serde(flatten)]
    parent: NotificationSettingDocument,
    channels: Vec<NotificationChannelDocument>,
}

/// Production Firestore-backed implementation of
/// [`NotificationSettingRepository`].
#[derive(Debug, Clone)]
pub struct FirestoreNotificationSettingRepository {
    db: Arc<FirestoreDb>,
}

impl FirestoreNotificationSettingRepository {
    pub fn new(db: Arc<FirestoreDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl NotificationSettingRepository for FirestoreNotificationSettingRepository {
    async fn find_by_id(
        &self,
        identifier: &NotificationSettingIdentifier,
    ) -> Result<Option<NotificationSetting>, DomainError> {
        let aggregate: Option<NotificationSettingAggregateDocument> = self
            .db
            .fluent()
            .select()
            .by_id_in(collections::NOTIFICATION_SETTINGS)
            .obj()
            .one(identifier.value().to_string())
            .await
            .map_err(map_firestore_error)?;
        match aggregate {
            Some(aggregate) => {
                let channels = aggregate
                    .channels
                    .into_iter()
                    .map(|channel| channel.to_domain())
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Some(aggregate.parent.to_domain(channels)?))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, setting: &NotificationSetting) -> Result<(), DomainError> {
        let aggregate = NotificationSettingAggregateDocument {
            parent: NotificationSettingDocument::from_domain(setting),
            channels: setting
                .channels()
                .iter()
                .map(NotificationChannelDocument::from_domain)
                .collect(),
        };
        self.db
            .fluent()
            .update()
            .in_col(collections::NOTIFICATION_SETTINGS)
            .document_id(setting.identifier().value())
            .object(&aggregate)
            .execute::<()>()
            .await
            .map_err(map_firestore_error)?;
        Ok(())
    }

    async fn find_default(&self) -> Result<NotificationSetting, DomainError> {
        self.find_by_id(&NotificationSettingIdentifier::default_id())
            .await?
            .ok_or_else(|| DomainError::FirestoreMappingError {
                reason: "default notification setting not found".to_string(),
            })
    }
}

fn map_firestore_error(error: firestore::errors::FirestoreError) -> DomainError {
    DomainError::FirestoreMappingError {
        reason: error.to_string(),
    }
}
