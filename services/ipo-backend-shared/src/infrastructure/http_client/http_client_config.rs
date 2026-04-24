use std::time::Duration;

/// Shared HTTP client configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientConfig {
    connect_timeout_ms: u64,
    read_timeout_ms: u64,
    max_idle_per_host: usize,
    keep_alive: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: 10_000,
            read_timeout_ms: 30_000,
            max_idle_per_host: 10,
            keep_alive: true,
        }
    }
}

impl HttpClientConfig {
    /// Creates a new HTTP client configuration.
    pub const fn new(
        connect_timeout_ms: u64,
        read_timeout_ms: u64,
        max_idle_per_host: usize,
        keep_alive: bool,
    ) -> Self {
        Self {
            connect_timeout_ms,
            read_timeout_ms,
            max_idle_per_host,
            keep_alive,
        }
    }

    /// Returns the connect timeout duration.
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_millis(self.connect_timeout_ms)
    }

    /// Returns the read timeout duration.
    pub fn read_timeout(&self) -> Duration {
        Duration::from_millis(self.read_timeout_ms)
    }

    /// Returns the max idle connections per host.
    pub fn max_idle_per_host(&self) -> usize {
        self.max_idle_per_host
    }

    /// Returns whether keep alive is enabled.
    pub fn keep_alive(&self) -> bool {
        self.keep_alive
    }
}
