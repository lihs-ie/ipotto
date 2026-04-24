use serde::{Deserialize, Serialize};

/// Lottery result for an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LotteryResult {
    Won,
    Lost,
    Alternate,
}
