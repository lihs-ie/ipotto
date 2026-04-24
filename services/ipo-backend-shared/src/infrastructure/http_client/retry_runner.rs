use std::{future::Future, time::Duration};

use rand::Rng;
use tokio::time::sleep;
use tracing::debug;

use super::RetryPolicy;

/// Runs an asynchronous fallible operation under the supplied
/// [`RetryPolicy`], retrying only when `classify_retryable(&error)`
/// returns `true`. Backs off exponentially between attempts with
/// uniformly distributed jitter capped at `policy.jitter_ratio()` of
/// the current delay, and ceilinged at `policy.max_backoff_ms()`.
///
/// * `policy.max_retries()` counts additional attempts *after* the
///   initial one. `max_retries = 3` therefore implies up to 4
///   invocations of the operation in the worst case.
/// * Non-retryable errors short-circuit and are returned immediately.
/// * Successful results are returned without sleeping.
///
/// The operation closure is `Fn() -> Future` so each attempt gets a
/// freshly constructed future — essential for `reqwest` request
/// builders which consume `self` on `.send()`.
pub async fn retry_with_policy<T, E, F, Fut>(
    policy: &RetryPolicy,
    classify_retryable: impl Fn(&E) -> bool,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let max_retries = policy.max_retries();
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if attempt == max_retries || !classify_retryable(&error) {
                    return Err(error);
                }
                let delay_ms = compute_backoff_ms(policy, attempt);
                debug!(
                    attempt = attempt + 1,
                    next_delay_ms = delay_ms,
                    "retry_with_policy: operation failed with a retryable error, sleeping before next attempt"
                );
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }
    unreachable!("retry_with_policy loop must return on the final attempt")
}

fn compute_backoff_ms(policy: &RetryPolicy, attempt_index: u32) -> u64 {
    let base = policy
        .initial_backoff_ms()
        .saturating_mul(2u64.saturating_pow(attempt_index));
    let base = base.min(policy.max_backoff_ms());
    if policy.jitter_ratio() <= f64::EPSILON || base == 0 {
        return base;
    }
    let jitter_range = (base as f64 * policy.jitter_ratio()).round() as i64;
    let mut rng = rand::thread_rng();
    let offset = rng.gen_range(-jitter_range..=jitter_range);
    let adjusted = base as i64 + offset;
    adjusted.max(0) as u64
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum FakeError {
        Transient,
        Permanent,
    }

    fn is_transient(error: &FakeError) -> bool {
        matches!(error, FakeError::Transient)
    }

    fn fast_policy() -> RetryPolicy {
        RetryPolicy::new(3, 1, 2, 0.0)
    }

    #[tokio::test]
    async fn returns_immediately_when_first_attempt_succeeds() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let result: Result<u32, FakeError> =
            retry_with_policy(&fast_policy(), is_transient, move || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(42)
                }
            })
            .await;
        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retries_on_transient_error_until_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let result: Result<u32, FakeError> =
            retry_with_policy(&fast_policy(), is_transient, move || {
                let counter = counter_clone.clone();
                async move {
                    let attempt = counter.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err(FakeError::Transient)
                    } else {
                        Ok(7)
                    }
                }
            })
            .await;
        assert_eq!(result, Ok(7));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn stops_retrying_when_attempts_exhausted() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let result: Result<u32, FakeError> =
            retry_with_policy(&fast_policy(), is_transient, move || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err(FakeError::Transient)
                }
            })
            .await;
        assert_eq!(result, Err(FakeError::Transient));
        assert_eq!(counter.load(Ordering::SeqCst), 4, "1 initial + 3 retries");
    }

    #[tokio::test]
    async fn does_not_retry_when_classify_returns_false() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let result: Result<u32, FakeError> =
            retry_with_policy(&fast_policy(), is_transient, move || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err(FakeError::Permanent)
                }
            })
            .await;
        assert_eq!(result, Err(FakeError::Permanent));
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "non-retryable error must short-circuit"
        );
    }

    #[test]
    fn compute_backoff_is_capped_by_max_backoff() {
        let policy = RetryPolicy::new(5, 500, 1_000, 0.0);
        assert_eq!(compute_backoff_ms(&policy, 0), 500);
        assert_eq!(compute_backoff_ms(&policy, 1), 1_000);
        assert_eq!(compute_backoff_ms(&policy, 2), 1_000, "capped at max");
        assert_eq!(
            compute_backoff_ms(&policy, 10),
            1_000,
            "saturating pow capped"
        );
    }

    #[test]
    fn compute_backoff_applies_jitter_within_bounds() {
        let policy = RetryPolicy::new(3, 1_000, 30_000, 0.5);
        for _ in 0..100 {
            let value = compute_backoff_ms(&policy, 0);
            assert!(value <= 1_500, "jitter should stay within +50%");
        }
    }
}
