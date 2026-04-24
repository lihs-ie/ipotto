use std::sync::Arc;

use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use jsonwebtoken::{decode, decode_header, Algorithm, Validation};
use serde::Deserialize;

use super::{FirebaseAuthError, FirebaseJwkCache, VerifiedToken};

const FIREBASE_ISSUER_PREFIX: &str = "https://securetoken.google.com/";

/// Runtime configuration for Firebase Authentication token verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirebaseAuthConfig {
    project_id: String,
    emulator_host: Option<String>,
}

impl FirebaseAuthConfig {
    pub fn new(project_id: impl Into<String>, emulator_host: Option<String>) -> Self {
        Self {
            project_id: project_id.into(),
            emulator_host: emulator_host.filter(|value| !value.is_empty()),
        }
    }

    /// Loads configuration from environment variables.
    ///
    /// Requires `FIREBASE_PROJECT_ID`. `FIREBASE_AUTH_EMULATOR_HOST`, when
    /// set to a non-empty value, puts the verifier in emulator mode.
    pub fn from_env() -> Result<Self, String> {
        let project_id = std::env::var("FIREBASE_PROJECT_ID")
            .map_err(|_| "FIREBASE_PROJECT_ID is not set".to_string())?;
        let emulator_host = std::env::var("FIREBASE_AUTH_EMULATOR_HOST")
            .ok()
            .filter(|value| !value.is_empty());
        Ok(Self::new(project_id, emulator_host))
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn emulator_host(&self) -> Option<&str> {
        self.emulator_host.as_deref()
    }

    fn expected_issuer(&self) -> String {
        format!("{FIREBASE_ISSUER_PREFIX}{}", self.project_id)
    }
}

/// Verifier that turns a raw Firebase Authentication ID token into a
/// [`VerifiedToken`] after validating its signature, issuer, audience, and
/// expiration. When the configuration points at a Firebase Auth emulator,
/// signature verification is intentionally skipped (the emulator issues
/// `alg:none` tokens).
pub struct FirebaseTokenVerifier {
    config: FirebaseAuthConfig,
    jwk_cache: Arc<FirebaseJwkCache>,
}

impl FirebaseTokenVerifier {
    pub fn new(config: FirebaseAuthConfig, jwk_cache: Arc<FirebaseJwkCache>) -> Self {
        Self { config, jwk_cache }
    }

    pub async fn verify(&self, token: &str) -> Result<VerifiedToken, FirebaseAuthError> {
        if self.config.emulator_host.is_some() {
            verify_emulator_token(token, self.config.expected_issuer().as_str(), &self.config)
        } else {
            self.verify_production_token(token).await
        }
    }

    async fn verify_production_token(
        &self,
        token: &str,
    ) -> Result<VerifiedToken, FirebaseAuthError> {
        let header = decode_header(token).map_err(|_| FirebaseAuthError::MalformedToken)?;
        if header.alg != Algorithm::RS256 {
            return Err(FirebaseAuthError::UnsupportedAlgorithm);
        }
        let kid = header.kid.ok_or(FirebaseAuthError::MissingKeyId)?;
        let decoding_key = self.jwk_cache.decoding_key_for(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.config.project_id()]);
        validation.set_issuer(&[self.config.expected_issuer()]);
        validation.set_required_spec_claims(&["exp", "iat", "iss", "aud", "sub"]);
        validation.leeway = 0;

        let token_data =
            decode::<FirebaseClaims>(token, &decoding_key, &validation).map_err(map_jwt_error)?;

        let verified = VerifiedToken::try_from(token_data.claims)?;
        enforce_google_provider(&verified)?;
        Ok(verified)
    }
}

fn verify_emulator_token(
    token: &str,
    expected_issuer: &str,
    config: &FirebaseAuthConfig,
) -> Result<VerifiedToken, FirebaseAuthError> {
    let claims = decode_unsigned_claims(token)?;
    if claims.iss != expected_issuer {
        return Err(FirebaseAuthError::InvalidIssuer);
    }
    if claims.aud != config.project_id() {
        return Err(FirebaseAuthError::InvalidAudience);
    }
    let now = Utc::now().timestamp();
    if claims.exp <= now {
        return Err(FirebaseAuthError::ExpiredToken);
    }
    VerifiedToken::try_from(claims)
}

fn decode_unsigned_claims(token: &str) -> Result<FirebaseClaims, FirebaseAuthError> {
    let mut parts = token.split('.');
    let _header = parts.next().ok_or(FirebaseAuthError::MalformedToken)?;
    let payload = parts.next().ok_or(FirebaseAuthError::MalformedToken)?;
    // Signature segment is ignored for emulator tokens (alg:none) but the
    // token is still required to have three dot-separated segments.
    let _signature = parts.next().ok_or(FirebaseAuthError::MalformedToken)?;
    if parts.next().is_some() {
        return Err(FirebaseAuthError::MalformedToken);
    }
    let raw = general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| FirebaseAuthError::MalformedToken)?;
    serde_json::from_slice(&raw).map_err(|_| FirebaseAuthError::MalformedToken)
}

fn enforce_google_provider(token: &VerifiedToken) -> Result<(), FirebaseAuthError> {
    match token.sign_in_provider.as_deref() {
        Some("google.com") => Ok(()),
        _ => Err(FirebaseAuthError::ProviderNotSupported),
    }
}

fn map_jwt_error(error: jsonwebtoken::errors::Error) -> FirebaseAuthError {
    use jsonwebtoken::errors::ErrorKind;
    match error.kind() {
        ErrorKind::ExpiredSignature => FirebaseAuthError::ExpiredToken,
        ErrorKind::InvalidIssuer => FirebaseAuthError::InvalidIssuer,
        ErrorKind::InvalidAudience => FirebaseAuthError::InvalidAudience,
        ErrorKind::InvalidSignature | ErrorKind::InvalidRsaKey(_) | ErrorKind::Crypto(_) => {
            FirebaseAuthError::InvalidSignature
        }
        ErrorKind::MissingRequiredClaim(_) => FirebaseAuthError::MissingSubject,
        _ => FirebaseAuthError::MalformedToken,
    }
}

#[derive(Debug, Deserialize)]
struct FirebaseClaims {
    iss: String,
    aud: String,
    sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    email_verified: Option<bool>,
    #[serde(default)]
    firebase: Option<FirebaseIdentities>,
    iat: i64,
    exp: i64,
}

#[derive(Debug, Deserialize)]
struct FirebaseIdentities {
    sign_in_provider: Option<String>,
}

impl TryFrom<FirebaseClaims> for VerifiedToken {
    type Error = FirebaseAuthError;

    fn try_from(claims: FirebaseClaims) -> Result<Self, Self::Error> {
        if claims.sub.is_empty() {
            return Err(FirebaseAuthError::MissingSubject);
        }
        let sign_in_provider = claims.firebase.and_then(|f| f.sign_in_provider);
        Ok(VerifiedToken::new(
            claims.sub,
            claims.email,
            claims.email_verified.unwrap_or(false),
            sign_in_provider,
            claims.iat,
            claims.exp,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;
    use base64::Engine as _;
    use chrono::Utc;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::Serialize;

    use super::{
        FirebaseAuthConfig, FirebaseAuthError, FirebaseJwkCache, FirebaseTokenVerifier,
        FIREBASE_ISSUER_PREFIX,
    };
    use crate::middleware::{
        firebase_jwk_cache::{FetchedJwks, JwksFetcher},
        test_keypair::test_keypair,
    };

    const TEST_KID: &str = "test-kid-1";
    const TEST_PROJECT_ID: &str = "ipotto-test";

    struct SinglePemFetcher;

    #[async_trait]
    impl JwksFetcher for SinglePemFetcher {
        async fn fetch(&self) -> Result<FetchedJwks, FirebaseAuthError> {
            let mut keys = HashMap::new();
            keys.insert(TEST_KID.to_string(), test_keypair().public_pem.clone());
            Ok(FetchedJwks {
                keys,
                ttl: Duration::from_secs(60),
            })
        }
    }

    #[derive(Serialize)]
    struct TestClaims {
        iss: String,
        aud: String,
        sub: String,
        email: Option<String>,
        email_verified: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        firebase: Option<TestFirebaseIdentities>,
        iat: i64,
        exp: i64,
    }

    #[derive(Serialize)]
    struct TestFirebaseIdentities {
        sign_in_provider: String,
    }

    fn sign_token(claims: &TestClaims, kid: Option<&str>, alg: Algorithm) -> String {
        let mut header = Header::new(alg);
        header.kid = kid.map(str::to_string);
        let key =
            EncodingKey::from_rsa_pem(&test_keypair().private_pem).expect("valid private key");
        encode(&header, claims, &key).expect("jwt encode")
    }

    fn valid_claims() -> TestClaims {
        let now = Utc::now().timestamp();
        TestClaims {
            iss: format!("{FIREBASE_ISSUER_PREFIX}{TEST_PROJECT_ID}"),
            aud: TEST_PROJECT_ID.to_string(),
            sub: "uid-abc".to_string(),
            email: Some("user@example.com".to_string()),
            email_verified: true,
            firebase: Some(TestFirebaseIdentities {
                sign_in_provider: "google.com".to_string(),
            }),
            iat: now - 10,
            exp: now + 600,
        }
    }

    fn build_verifier(emulator: Option<String>) -> FirebaseTokenVerifier {
        let cache = FirebaseJwkCache::new(Arc::new(SinglePemFetcher));
        let config = FirebaseAuthConfig::new(TEST_PROJECT_ID, emulator);
        FirebaseTokenVerifier::new(config, Arc::new(cache))
    }

    #[tokio::test]
    async fn verifies_production_token_with_rs256_signature() {
        let verifier = build_verifier(None);
        let token = sign_token(&valid_claims(), Some(TEST_KID), Algorithm::RS256);

        let verified = verifier.verify(&token).await.expect("token accepted");
        assert_eq!(verified.uid, "uid-abc");
        assert_eq!(verified.email.as_deref(), Some("user@example.com"));
        assert!(verified.email_verified);
    }

    #[tokio::test]
    async fn rejects_expired_token() {
        let verifier = build_verifier(None);
        let mut claims = valid_claims();
        let now = Utc::now().timestamp();
        claims.iat = now - 3_600;
        claims.exp = now - 60;

        let token = sign_token(&claims, Some(TEST_KID), Algorithm::RS256);
        let result = verifier.verify(&token).await;

        assert_eq!(result.unwrap_err(), FirebaseAuthError::ExpiredToken);
    }

    #[tokio::test]
    async fn rejects_wrong_issuer() {
        let verifier = build_verifier(None);
        let mut claims = valid_claims();
        claims.iss = "https://accounts.google.com".to_string();
        let token = sign_token(&claims, Some(TEST_KID), Algorithm::RS256);

        let result = verifier.verify(&token).await;
        assert_eq!(result.unwrap_err(), FirebaseAuthError::InvalidIssuer);
    }

    #[tokio::test]
    async fn rejects_wrong_audience() {
        let verifier = build_verifier(None);
        let mut claims = valid_claims();
        claims.aud = "some-other-project".to_string();
        let token = sign_token(&claims, Some(TEST_KID), Algorithm::RS256);

        let result = verifier.verify(&token).await;
        assert_eq!(result.unwrap_err(), FirebaseAuthError::InvalidAudience);
    }

    #[tokio::test]
    async fn rejects_token_without_kid() {
        let verifier = build_verifier(None);
        let token = sign_token(&valid_claims(), None, Algorithm::RS256);

        let result = verifier.verify(&token).await;
        assert_eq!(result.unwrap_err(), FirebaseAuthError::MissingKeyId);
    }

    #[tokio::test]
    async fn rejects_token_with_unknown_kid() {
        let verifier = build_verifier(None);
        let token = sign_token(&valid_claims(), Some("unknown-kid"), Algorithm::RS256);

        let result = verifier.verify(&token).await;
        assert_eq!(result.unwrap_err(), FirebaseAuthError::UnknownKeyId);
    }

    #[tokio::test]
    async fn rejects_non_google_provider() {
        let verifier = build_verifier(None);
        let mut claims = valid_claims();
        claims.firebase = Some(TestFirebaseIdentities {
            sign_in_provider: "password".to_string(),
        });
        let token = sign_token(&claims, Some(TEST_KID), Algorithm::RS256);

        let result = verifier.verify(&token).await;
        assert_eq!(result.unwrap_err(), FirebaseAuthError::ProviderNotSupported);
    }

    #[tokio::test]
    async fn rejects_token_without_firebase_claim() {
        let verifier = build_verifier(None);
        let mut claims = valid_claims();
        claims.firebase = None;
        let token = sign_token(&claims, Some(TEST_KID), Algorithm::RS256);

        let result = verifier.verify(&token).await;
        assert_eq!(result.unwrap_err(), FirebaseAuthError::ProviderNotSupported);
    }

    #[tokio::test]
    async fn emulator_mode_accepts_unsigned_token() {
        let verifier = build_verifier(Some("127.0.0.1:9099".to_string()));

        // Manually construct an `alg:none` token (header + payload + empty signature).
        let claims = valid_claims();
        let header = r#"{"alg":"none","typ":"JWT"}"#;
        let encoded_header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header);
        let payload_json = serde_json::to_vec(&claims).expect("claims encode");
        let encoded_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload_json);
        let token = format!("{encoded_header}.{encoded_payload}.");

        let verified = verifier.verify(&token).await.expect("emulator accepts");
        assert_eq!(verified.uid, "uid-abc");
    }

    #[tokio::test]
    async fn emulator_mode_rejects_expired_claims() {
        let verifier = build_verifier(Some("127.0.0.1:9099".to_string()));
        let mut claims = valid_claims();
        let now = Utc::now().timestamp();
        claims.iat = now - 3_600;
        claims.exp = now - 60;

        let header = r#"{"alg":"none","typ":"JWT"}"#;
        let encoded_header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header);
        let payload_json = serde_json::to_vec(&claims).expect("claims encode");
        let encoded_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload_json);
        let token = format!("{encoded_header}.{encoded_payload}.");

        let result = verifier.verify(&token).await;
        assert_eq!(result.unwrap_err(), FirebaseAuthError::ExpiredToken);
    }
}
