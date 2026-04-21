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

#[cfg(test)]
mod tests {
    use super::RetryPolicy;

    #[test]
    fn default_uses_exponential_backoff_values() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries(), 3);
        assert_eq!(policy.initial_backoff_ms(), 1_000);
        assert_eq!(policy.max_backoff_ms(), 30_000);
        assert!((policy.jitter_ratio() - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn new_creates_policy_with_supplied_values() {
        let policy = RetryPolicy::new(5, 500, 10_000, 0.25);
        assert_eq!(policy.max_retries(), 5);
        assert_eq!(policy.initial_backoff_ms(), 500);
        assert_eq!(policy.max_backoff_ms(), 10_000);
        assert!((policy.jitter_ratio() - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn clones_preserve_configuration() {
        let original = RetryPolicy::new(4, 250, 5_000, 0.2);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
