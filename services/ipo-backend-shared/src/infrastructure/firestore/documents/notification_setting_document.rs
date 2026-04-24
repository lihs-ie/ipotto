use serde::{Deserialize, Serialize};

use crate::{
    domain::notification::{
        NotificationChannel, NotificationSetting, NotificationSettingIdentifier,
    },
    errors::DomainError,
};

/// Firestore parent document for `NotificationSetting`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationSettingDocument {
    pub identifier: String,
    pub enabled: bool,
}

impl NotificationSettingDocument {
    pub fn from_domain(setting: &NotificationSetting) -> Self {
        Self {
            identifier: setting.identifier().value().to_string(),
            enabled: setting.enabled(),
        }
    }

    pub fn to_domain(
        &self,
        channels: Vec<NotificationChannel>,
    ) -> Result<NotificationSetting, DomainError> {
        NotificationSetting::reconstruct(
            NotificationSettingIdentifier::new(self.identifier.clone())?,
            self.enabled,
            channels,
        )
    }
}
