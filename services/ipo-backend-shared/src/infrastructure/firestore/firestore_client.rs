use std::env;

use firestore::{FirestoreDb, FirestoreDbOptions};

use crate::errors::DomainError;

/// Firestore collection names used by the application. Centralised here
/// so every repository references the same string constants, and the
/// terraform-managed composite indexes can cross-check spelling.
pub mod collections {
    pub const IPO_STOCKS: &str = "ipo_stocks";
    pub const LOTTERY_APPLICATIONS: &str = "lottery_applications";
    pub const SECURITIES_ACCOUNTS: &str = "securities_accounts";
    pub const EXCLUSIONS: &str = "exclusions";
    pub const NOTIFICATION_SETTINGS: &str = "notification_settings";
    pub const NOTIFICATION_CHANNELS: &str = "notification_channels";
    pub const OPERATION_LOGS: &str = "operation_logs";
}

/// Builds a [`FirestoreDb`] client that honours the
/// `FIRESTORE_EMULATOR_HOST` environment variable so local docker-compose
/// and CI runs target the emulator without any code changes, while
/// production invocations resolve to the real Google Cloud project.
pub async fn build_firestore_client(project_id: &str) -> Result<FirestoreDb, DomainError> {
    let mut options = FirestoreDbOptions::new(project_id.to_string());
    if let Ok(raw) = env::var("FIRESTORE_EMULATOR_HOST") {
        let host = normalize_emulator_host(&raw);
        options = options.with_firebase_api_url(host);
    }
    FirestoreDb::with_options(options)
        .await
        .map_err(|error| DomainError::FirestoreMappingError {
            reason: format!("failed to initialise Firestore client: {error}"),
        })
}

fn normalize_emulator_host(value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("http://{value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_emulator_host_adds_http_scheme_when_missing() {
        assert_eq!(
            normalize_emulator_host("firebase-emulator:8088"),
            "http://firebase-emulator:8088"
        );
    }

    #[test]
    fn normalize_emulator_host_preserves_existing_scheme() {
        assert_eq!(
            normalize_emulator_host("https://firebase-emulator:8088"),
            "https://firebase-emulator:8088"
        );
    }
}
