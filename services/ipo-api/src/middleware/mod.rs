mod email_allowlist;
mod firebase_auth;
mod firebase_error;
pub(crate) mod firebase_jwk_cache;
mod firebase_token_verifier;
mod google_secure_token_fetcher;
mod request_logging;
#[cfg(test)]
mod test_keypair;
mod verified_token;

pub use email_allowlist::{email_allowlist_middleware, EmailAllowlistConfig};
pub use firebase_auth::firebase_auth_middleware;
pub use firebase_error::FirebaseAuthError;
pub use firebase_jwk_cache::{FirebaseJwkCache, JwksFetcher};
pub use firebase_token_verifier::{FirebaseAuthConfig, FirebaseTokenVerifier};
pub use google_secure_token_fetcher::GoogleSecureTokenFetcher;
pub use request_logging::request_logging_middleware;
pub use verified_token::VerifiedToken;
