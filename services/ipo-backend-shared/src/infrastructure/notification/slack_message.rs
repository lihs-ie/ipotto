use serde::{Deserialize, Serialize};

/// Simplified Slack webhook payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlackMessage {
    pub text: String,
}

impl SlackMessage {
    /// Creates a Slack message.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}
