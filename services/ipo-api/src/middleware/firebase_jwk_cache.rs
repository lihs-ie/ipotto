use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use jsonwebtoken::DecodingKey;
use tokio::sync::RwLock;

use super::FirebaseAuthError;

/// Raw JSON Web Key material fetched from the upstream endpoint (Google).
pub struct FetchedJwks {
    pub keys: HashMap<String, Vec<u8>>,
    pub ttl: Duration,
}

/// Abstraction over the network call that retrieves Firebase's public keys.
///
/// Exposed as a trait so tests can inject a deterministic fetcher without
/// touching the network.
#[async_trait]
pub trait JwksFetcher: Send + Sync {
    async fn fetch(&self) -> Result<FetchedJwks, FirebaseAuthError>;
}

/// Async-safe cache of Firebase-issued RSA public keys keyed by `kid`.
///
/// On the hot path reads go through a RwLock read guard and return the
/// already-parsed [`DecodingKey`]. On cache miss or TTL expiry the cache
/// is refreshed via the injected [`JwksFetcher`].
pub struct FirebaseJwkCache {
    fetcher: Arc<dyn JwksFetcher>,
    state: RwLock<Option<CachedState>>,
}

struct CachedState {
    keys: HashMap<String, DecodingKey>,
    expires_at: Instant,
}

impl FirebaseJwkCache {
    pub fn new(fetcher: Arc<dyn JwksFetcher>) -> Self {
        Self {
            fetcher,
            state: RwLock::new(None),
        }
    }

    /// Resolve the decoding key for the supplied `kid`, refreshing the
    /// cache from the upstream if either the cache is empty, has expired,
    /// or does not contain the requested `kid` (key rotation case).
    pub async fn decoding_key_for(&self, kid: &str) -> Result<DecodingKey, FirebaseAuthError> {
        if let Some(key) = self.lookup_live_key(kid).await {
            return Ok(key);
        }

        self.refresh().await?;

        self.lookup_live_key(kid)
            .await
            .ok_or(FirebaseAuthError::UnknownKeyId)
    }

    async fn lookup_live_key(&self, kid: &str) -> Option<DecodingKey> {
        let guard = self.state.read().await;
        let state = guard.as_ref()?;
        if state.expires_at <= Instant::now() {
            return None;
        }
        state.keys.get(kid).cloned()
    }

    async fn refresh(&self) -> Result<(), FirebaseAuthError> {
        let fetched = self.fetcher.fetch().await?;
        let mut parsed = HashMap::with_capacity(fetched.keys.len());
        for (kid, pem) in fetched.keys {
            let key = DecodingKey::from_rsa_pem(&pem).map_err(|error| {
                FirebaseAuthError::JwksFetchFailed(format!("cannot parse JWK for {kid}: {error}"))
            })?;
            parsed.insert(kid, key);
        }
        let mut guard = self.state.write().await;
        *guard = Some(CachedState {
            keys: parsed,
            expires_at: Instant::now() + fetched.ttl,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::time::Duration;

    use async_trait::async_trait;
    use tokio::sync::Mutex;

    use super::{FetchedJwks, FirebaseJwkCache, JwksFetcher};
    use crate::middleware::FirebaseAuthError;

    const TEST_PEM: &[u8] = include_bytes!("testdata/firebase_jwk_sample.pem");

    struct StubFetcher {
        calls: AtomicUsize,
        keys: Mutex<Vec<(String, Vec<u8>)>>,
        ttl: Duration,
    }

    impl StubFetcher {
        fn new(keys: Vec<(String, Vec<u8>)>, ttl: Duration) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                keys: Mutex::new(keys),
                ttl,
            }
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl JwksFetcher for StubFetcher {
        async fn fetch(&self) -> Result<FetchedJwks, FirebaseAuthError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let keys = self.keys.lock().await.clone();
            Ok(FetchedJwks {
                keys: keys.into_iter().collect(),
                ttl: self.ttl,
            })
        }
    }

    #[tokio::test]
    async fn fetches_once_per_ttl_and_returns_cached_key() {
        let fetcher = Arc::new(StubFetcher::new(
            vec![("kid-a".to_string(), TEST_PEM.to_vec())],
            Duration::from_secs(60),
        ));
        let cache = FirebaseJwkCache::new(fetcher.clone());

        cache
            .decoding_key_for("kid-a")
            .await
            .expect("key available");
        cache
            .decoding_key_for("kid-a")
            .await
            .expect("cached key available");

        assert_eq!(fetcher.call_count(), 1);
    }

    #[tokio::test]
    async fn refreshes_once_for_unknown_kid_and_surfaces_error_if_still_missing() {
        let fetcher = Arc::new(StubFetcher::new(
            vec![("kid-a".to_string(), TEST_PEM.to_vec())],
            Duration::from_secs(60),
        ));
        let cache = FirebaseJwkCache::new(fetcher.clone());

        let result = cache.decoding_key_for("kid-missing").await;

        assert_eq!(fetcher.call_count(), 1);
        assert!(
            matches!(result, Err(FirebaseAuthError::UnknownKeyId)),
            "expected UnknownKeyId error"
        );
    }

    struct FailingFetcher;

    #[async_trait]
    impl JwksFetcher for FailingFetcher {
        async fn fetch(&self) -> Result<FetchedJwks, FirebaseAuthError> {
            Err(FirebaseAuthError::JwksFetchFailed(
                "network down".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn surfaces_fetch_error() {
        let cache = FirebaseJwkCache::new(Arc::new(FailingFetcher));
        let result = cache.decoding_key_for("kid-a").await;

        assert!(matches!(result, Err(FirebaseAuthError::JwksFetchFailed(_))));
    }
}
