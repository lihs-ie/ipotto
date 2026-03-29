use serde::{Deserialize, Serialize};

/// Type of notification channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChannelType {
    Line,
    Email,
    Slack,
}

impl ChannelType {
    /// Returns the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Line => "LINE",
            Self::Email => "Email",
            Self::Slack => "Slack",
        }
    }
}
