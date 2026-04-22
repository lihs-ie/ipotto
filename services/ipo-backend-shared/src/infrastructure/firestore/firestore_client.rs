use std::env;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use firestore::{FirestoreDb, FirestoreDbOptions};
use gcloud_sdk::{Source, Token, TokenSourceType, GCP_DEFAULT_SCOPES};

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
///
/// The Firestore emulator does not require OAuth credentials, but
/// `firestore = 0.48` inherits `TokenSourceType::Default` from
/// `FirestoreDb::with_options`, which tries to fetch a token from ADC
/// (Application Default Credentials) or the GCE metadata server and
/// fails in environments without GCP auth (local dev, CI). When
/// `FIRESTORE_EMULATOR_HOST` is present we swap in an
/// [`EmulatorTokenSource`] that yields a dummy bearer token — the
/// emulator accepts any `Authorization` header — so the gRPC client
/// can complete its handshake.
pub async fn build_firestore_client(project_id: &str) -> Result<FirestoreDb, DomainError> {
    match env::var("FIRESTORE_EMULATOR_HOST") {
        Ok(raw) => {
            let options = FirestoreDbOptions::new(project_id.to_string())
                .with_firebase_api_url(normalize_emulator_host(&raw));
            FirestoreDb::with_options_token_source(
                options,
                GCP_DEFAULT_SCOPES.clone(),
                TokenSourceType::ExternalSource(Box::new(EmulatorTokenSource)),
            )
            .await
        }
        Err(_) => {
            let options = FirestoreDbOptions::new(project_id.to_string());
            FirestoreDb::with_options(options).await
        }
    }
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

/// Token source that hands out a constant dummy bearer token. The
/// Firestore emulator ignores the token value, so this keeps the
/// gcloud-sdk auth pipeline happy without needing real GCP credentials.
#[derive(Debug)]
struct EmulatorTokenSource;

#[async_trait]
impl Source for EmulatorTokenSource {
    async fn token(&self) -> gcloud_sdk::error::Result<Token> {
        Ok(Token::new(
            "Bearer".to_string(),
            gcloud_sdk::SecretValue::new("firestore-emulator-dummy-token".to_string().into_bytes()),
            Utc::now() + Duration::hours(1),
        ))
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

    #[tokio::test]
    async fn emulator_token_source_returns_bearer_with_future_expiry() {
        let source = EmulatorTokenSource;
        let token = source.token().await.expect("token should succeed");
        assert_eq!(token.token_type, "Bearer");
        assert!(!token.token.as_sensitive_str().is_empty());
        assert!(token.expiry > Utc::now());
    }
}
