use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use super::VerifiedToken;

const DEFAULT_MAX_REQUESTS: u32 = 100;
const DEFAULT_WINDOW_SECONDS: u64 = 60;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    max_requests: u32,
    window: Duration,
}

impl RateLimitConfig {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_REQUESTS,
            Duration::from_secs(DEFAULT_WINDOW_SECONDS),
        )
    }
}

pub struct RateLimitState {
    config: RateLimitConfig,
    buckets: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimitState {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    fn check(&self, key: &str) -> Result<(), Duration> {
        let now = Instant::now();
        let window_start = now - self.config.window;

        let mut buckets = self
            .buckets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let timestamps = buckets.entry(key.to_string()).or_default();

        timestamps.retain(|timestamp| *timestamp > window_start);

        if timestamps.len() as u32 >= self.config.max_requests {
            let oldest_in_window = timestamps[0];
            let retry_after = self.config.window - (now - oldest_in_window);
            return Err(retry_after);
        }

        timestamps.push(now);
        Ok(())
    }
}

#[derive(Serialize)]
struct RateLimitErrorBody {
    error: RateLimitErrorDetail,
}

#[derive(Serialize)]
struct RateLimitErrorDetail {
    code: &'static str,
    message: &'static str,
}

pub async fn rate_limit_middleware(
    State(state): State<Arc<RateLimitState>>,
    request: Request,
    next: Next,
) -> Response {
    let key = request
        .extensions()
        .get::<VerifiedToken>()
        .map(|token| token.uid.clone())
        .unwrap_or_default();

    match state.check(&key) {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            let retry_seconds = retry_after.as_secs().max(1);
            let body = RateLimitErrorBody {
                error: RateLimitErrorDetail {
                    code: "RATE_LIMIT_EXCEEDED",
                    message: "リクエスト数が上限を超えました。しばらく待ってから再試行してください",
                },
            };
            let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
            if let Ok(value) = HeaderValue::from_str(&retry_seconds.to_string()) {
                response.headers_mut().insert("retry-after", value);
            }
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use super::{RateLimitConfig, RateLimitState};

    #[test]
    fn allows_requests_within_limit() {
        let state = RateLimitState::new(RateLimitConfig::new(5, Duration::from_secs(60)));
        for _ in 0..5 {
            assert!(state.check("uid-1").is_ok());
        }
    }

    #[test]
    fn rejects_requests_exceeding_limit() {
        let state = RateLimitState::new(RateLimitConfig::new(3, Duration::from_secs(60)));
        for _ in 0..3 {
            assert!(state.check("uid-1").is_ok());
        }
        let result = state.check("uid-1");
        assert!(result.is_err());
    }

    #[test]
    fn tracks_separate_keys_independently() {
        let state = RateLimitState::new(RateLimitConfig::new(2, Duration::from_secs(60)));
        assert!(state.check("uid-a").is_ok());
        assert!(state.check("uid-a").is_ok());
        assert!(state.check("uid-a").is_err());

        assert!(state.check("uid-b").is_ok());
        assert!(state.check("uid-b").is_ok());
        assert!(state.check("uid-b").is_err());
    }

    #[test]
    fn returns_positive_retry_after_duration() {
        let state = RateLimitState::new(RateLimitConfig::new(1, Duration::from_secs(60)));
        assert!(state.check("uid-1").is_ok());
        let retry_after = state.check("uid-1").unwrap_err();
        assert!(retry_after.as_secs() > 0);
        assert!(retry_after.as_secs() <= 60);
    }

    #[test]
    fn default_config_allows_100_per_minute() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(60));
    }

    #[test]
    fn shared_state_is_thread_safe() {
        let state = Arc::new(RateLimitState::new(RateLimitConfig::new(
            10,
            Duration::from_secs(60),
        )));
        let state_clone = Arc::clone(&state);
        let handle = std::thread::spawn(move || {
            for _ in 0..5 {
                let _ = state_clone.check("uid-thread");
            }
        });
        for _ in 0..5 {
            let _ = state.check("uid-thread");
        }
        handle.join().expect("thread join");
        assert!(state.check("uid-thread").is_err());
    }
}
