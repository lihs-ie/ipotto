/// Polling configuration for authentication mail retrieval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailPollingConfig {
    polling_interval_seconds: u32,
    max_timeout_seconds: u32,
}

impl Default for MailPollingConfig {
    fn default() -> Self {
        Self {
            polling_interval_seconds: 3,
            max_timeout_seconds: 120,
        }
    }
}

impl MailPollingConfig {
    /// Creates a polling configuration.
    pub const fn new(polling_interval_seconds: u32, max_timeout_seconds: u32) -> Self {
        Self {
            polling_interval_seconds,
            max_timeout_seconds,
        }
    }

    pub fn polling_interval_seconds(&self) -> u32 {
        self.polling_interval_seconds
    }
    pub fn max_timeout_seconds(&self) -> u32 {
        self.max_timeout_seconds
    }
}
