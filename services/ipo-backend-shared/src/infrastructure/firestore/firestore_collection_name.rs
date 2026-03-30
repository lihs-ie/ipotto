/// Firestore collection names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirestoreCollectionName {
    IpoStocks,
    Exclusions,
    LotteryApplications,
    SecuritiesAccounts,
    NotificationSettings,
    OperationLogs,
}

impl FirestoreCollectionName {
    /// Returns the canonical collection name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IpoStocks => "ipo_stocks",
            Self::Exclusions => "exclusions",
            Self::LotteryApplications => "lottery_applications",
            Self::SecuritiesAccounts => "securities_accounts",
            Self::NotificationSettings => "notification_settings",
            Self::OperationLogs => "operation_logs",
        }
    }
}
