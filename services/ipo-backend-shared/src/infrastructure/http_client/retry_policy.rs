/// Shared retry policy for outbound HTTP calls.
#[derive(Debug, Clone, PartialEq)]
pub struct RetryPolicy {
    max_retries: u32,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
    jitter_ratio: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1_000,
            max_backoff_ms: 30_000,
            jitter_ratio: 0.1,
        }
    }
}

impl RetryPolicy {
    /// Creates a retry policy.
    pub const fn new(
        max_retries: u32,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
        jitter_ratio: f64,
    ) -> Self {
        Self {
            max_retries,
            initial_backoff_ms,
            max_backoff_ms,
            jitter_ratio,
        }
    }

    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }
    pub fn initial_backoff_ms(&self) -> u64 {
        self.initial_backoff_ms
    }
    pub fn max_backoff_ms(&self) -> u64 {
        self.max_backoff_ms
    }
    pub fn jitter_ratio(&self) -> f64 {
        self.jitter_ratio
    }
}
