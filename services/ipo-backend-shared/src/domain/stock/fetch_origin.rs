use serde::{Deserialize, Serialize};

/// Source origin of fetched stock data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FetchOrigin {
    ExternalSite,
    SecuritiesSite,
}
