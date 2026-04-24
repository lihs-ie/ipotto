use serde::{Deserialize, Serialize};

/// Activation flag for a securities account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountActivation {
    is_active: bool,
}

impl AccountActivation {
    /// Creates an activation value.
    pub fn new(is_active: bool) -> Self {
        Self { is_active }
    }

    /// Returns whether the account is active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
