use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result of testing a securities account connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    success: bool,
    message: String,
    tested_at: DateTime<Utc>,
}

impl ConnectionTestResult {
    /// Creates a connection test result.
    pub fn new(success: bool, message: impl Into<String>, tested_at: DateTime<Utc>) -> Self {
        Self {
            success,
            message: message.into(),
            tested_at,
        }
    }

    /// Returns whether the connection test succeeded.
    pub fn success(&self) -> bool {
        self.success
    }

    /// Returns the message associated with the test result.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns when the test was executed.
    pub fn tested_at(&self) -> DateTime<Utc> {
        self.tested_at
    }
}
