use serde::{Deserialize, Serialize};

/// Simplified email message payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailMessage {
    pub subject: String,
    pub body: String,
}

impl EmailMessage {
    /// Creates an email message.
    pub fn new(subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            body: body.into(),
        }
    }
}
