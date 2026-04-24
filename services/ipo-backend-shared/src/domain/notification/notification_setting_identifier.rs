use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::errors::DomainError;

/// Identifier for a notification setting aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NotificationSettingIdentifier(String);

impl NotificationSettingIdentifier {
    /// Creates an identifier from an existing value.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value == "default" {
            return Ok(Self(value));
        }
        Ulid::from_string(&value).map_err(|error| DomainError::InvalidIdentifier {
            kind: "notification_setting_identifier".to_string(),
            reason: error.to_string(),
        })?;
        Ok(Self(value))
    }

    /// Returns the default notification setting identifier.
    pub fn default_id() -> Self {
        Self("default".to_string())
    }

    /// Generates a new identifier.
    pub fn generate() -> Self {
        Self(Ulid::new().to_string())
    }

    /// Returns the raw identifier value.
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for NotificationSettingIdentifier {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<NotificationSettingIdentifier> for String {
    fn from(value: NotificationSettingIdentifier) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::NotificationSettingIdentifier;

    #[test]
    fn accepts_default_identifier() {
        let identifier = NotificationSettingIdentifier::new("default").expect("default identifier");
        assert_eq!(identifier.value(), "default");
    }

    #[test]
    fn generate_returns_ulid() {
        let identifier = NotificationSettingIdentifier::generate();
        assert!(NotificationSettingIdentifier::new(identifier.value()).is_ok());
    }
}
