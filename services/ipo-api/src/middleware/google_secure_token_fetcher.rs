use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;

use super::{
    firebase_jwk_cache::{FetchedJwks, JwksFetcher},
    FirebaseAuthError,
};

const GOOGLE_SECURE_TOKEN_KEYS_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";
const DEFAULT_JWKS_TTL_SECONDS: u64 = 3_600;

/// [`JwksFetcher`] implementation that calls the Google Secure Token public
/// keys endpoint over HTTPS. Shared Firebase ID token verification for all
/// production traffic.
pub struct GoogleSecureTokenFetcher {
    http: reqwest::Client,
    url: String,
}

impl GoogleSecureTokenFetcher {
    pub fn new(http: reqwest::Client) -> Self {
        Self {
            http,
            url: GOOGLE_SECURE_TOKEN_KEYS_URL.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_url(http: reqwest::Client, url: impl Into<String>) -> Self {
        Self {
            http,
            url: url.into(),
        }
    }
}

#[async_trait]
impl JwksFetcher for GoogleSecureTokenFetcher {
    async fn fetch(&self) -> Result<FetchedJwks, FirebaseAuthError> {
        let response = self
            .http
            .get(&self.url)
            .send()
            .await
            .map_err(|error| FirebaseAuthError::JwksFetchFailed(error.to_string()))?;

        let ttl = response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok())
            .and_then(parse_max_age)
            .unwrap_or(Duration::from_secs(DEFAULT_JWKS_TTL_SECONDS));

        let keys: HashMap<String, String> = response
            .json()
            .await
            .map_err(|error| FirebaseAuthError::JwksFetchFailed(error.to_string()))?;

        Ok(FetchedJwks {
            keys: keys
                .into_iter()
                .map(|(kid, pem)| (kid, pem.into_bytes()))
                .collect(),
            ttl,
        })
    }
}

fn parse_max_age(cache_control: &str) -> Option<Duration> {
    cache_control
        .split(',')
        .filter_map(|directive| {
            let trimmed = directive.trim();
            trimmed
                .strip_prefix("max-age=")
                .and_then(|value| value.parse::<u64>().ok())
        })
        .next()
        .map(Duration::from_secs)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{parse_max_age, FetchedJwks, GoogleSecureTokenFetcher, JwksFetcher};
    use crate::middleware::test_keypair::test_keypair;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn parses_max_age_directive() {
        assert_eq!(
            parse_max_age("public, max-age=18000, must-revalidate"),
            Some(Duration::from_secs(18_000))
        );
        assert_eq!(parse_max_age("no-cache"), None);
    }

    #[tokio::test]
    async fn fetches_keys_and_ttl_from_endpoint() {
        let server = MockServer::start().await;
        let public_pem = test_keypair().public_pem.clone();
        let pem_string = std::str::from_utf8(&public_pem)
            .expect("pem utf8")
            .to_string();
        let body = serde_json::json!({ "test-kid": pem_string });

        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("cache-control", "public, max-age=7200")
                    .set_body_json(body),
            )
            .mount(&server)
            .await;

        let fetcher = GoogleSecureTokenFetcher::with_url(
            reqwest::Client::new(),
            format!("{}/", server.uri()),
        );

        let FetchedJwks { keys, ttl } = fetcher.fetch().await.expect("fetch ok");
        assert_eq!(keys.get("test-kid").cloned(), Some(public_pem));
        assert_eq!(ttl, Duration::from_secs(7_200));
    }

    #[tokio::test]
    async fn surface_network_errors() {
        let fetcher = GoogleSecureTokenFetcher::with_url(
            reqwest::Client::new(),
            "http://127.0.0.1:1/", // guaranteed to fail
        );

        let result = fetcher.fetch().await;
        assert!(result.is_err());
    }
}
