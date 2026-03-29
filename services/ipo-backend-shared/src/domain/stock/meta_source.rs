use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::FetchOrigin;

/// Metadata about where the stock data came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaSource {
    source: FetchOrigin,
    fetched_at: DateTime<Utc>,
}

impl MetaSource {
    /// Creates metadata about the source of stock data.
    pub fn new(source: FetchOrigin, fetched_at: DateTime<Utc>) -> Self {
        Self { source, fetched_at }
    }

    /// Returns the source type.
    pub fn source(&self) -> FetchOrigin {
        self.source
    }

    /// Returns when the source was fetched.
    pub fn fetched_at(&self) -> DateTime<Utc> {
        self.fetched_at
    }
}
