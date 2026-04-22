use std::{env, sync::Arc};

use async_trait::async_trait;
use ipo_backend_shared::http::HttpServiceConfig;

use crate::middleware::firebase_jwk_cache::FetchedJwks;
use crate::middleware::{
    EmailAllowlistConfig, FirebaseAuthConfig, FirebaseAuthError, FirebaseJwkCache,
    FirebaseTokenVerifier, GoogleSecureTokenFetcher, JwksFetcher,
};

pub const HTTP_SERVICE_CONFIG: HttpServiceConfig = HttpServiceConfig::new("ipo-api", 8080);

pub fn ipo_browser_base_url() -> String {
    env::var("IPO_BROWSER_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
}

pub fn firebase_project_id() -> String {
    env::var("FIREBASE_PROJECT_ID").unwrap_or_else(|_| "ipotto-local".to_string())
}

pub fn notification_from_address() -> String {
    env::var("IPO_NOTIFICATION_FROM_ADDRESS").unwrap_or_else(|_| "no-reply@example.com".to_string())
}

pub fn sendgrid_api_key() -> Option<String> {
    env::var("SENDGRID_API_KEY")
        .ok()
        .filter(|value| !value.is_empty())
}

/// Overridable SendGrid endpoint. Production defaults to the live SendGrid
/// v3 Mail Send API; dev and CI smoke override this to point at the
/// notification mock server.
pub fn sendgrid_endpoint() -> String {
    env::var("SENDGRID_ENDPOINT")
        .unwrap_or_else(|_| "https://api.sendgrid.com/v3/mail/send".to_string())
}

/// Overridable LINE Notify endpoint.
pub fn line_notify_endpoint() -> String {
    env::var("LINE_NOTIFY_ENDPOINT")
        .unwrap_or_else(|_| "https://notify-api.line.me/api/notify".to_string())
}

/// Loads the email allow-list from `ALLOWED_EMAIL` (comma-separated).
pub fn build_email_allowlist() -> Result<Arc<EmailAllowlistConfig>, String> {
    EmailAllowlistConfig::from_env().map(Arc::new)
}

/// Constructs the Firebase ID token verifier used by the authentication
/// middleware. When `FIREBASE_AUTH_EMULATOR_HOST` is set the verifier runs in
/// emulator mode and never calls the upstream JWK endpoint.
pub fn build_firebase_token_verifier() -> Result<Arc<FirebaseTokenVerifier>, String> {
    let config = FirebaseAuthConfig::from_env()?;
    let fetcher: Arc<dyn JwksFetcher> = if config.emulator_host().is_some() {
        Arc::new(EmulatorModeFetcher)
    } else {
        Arc::new(GoogleSecureTokenFetcher::new(reqwest::Client::new()))
    };
    let cache = Arc::new(FirebaseJwkCache::new(fetcher));
    Ok(Arc::new(FirebaseTokenVerifier::new(config, cache)))
}

/// Placeholder fetcher returned in emulator mode. The emulator path in
/// [`FirebaseTokenVerifier::verify`] short-circuits before reaching the JWK
/// cache, so this implementation is only invoked if emulator-mode logic is
/// ever bypassed, in which case we prefer a loud failure over a silent
/// fallback to production verification.
struct EmulatorModeFetcher;

#[async_trait]
impl JwksFetcher for EmulatorModeFetcher {
    async fn fetch(&self) -> Result<FetchedJwks, FirebaseAuthError> {
        Err(FirebaseAuthError::JwksFetchFailed(
            "JWK fetch attempted in emulator mode".to_string(),
        ))
    }
}
