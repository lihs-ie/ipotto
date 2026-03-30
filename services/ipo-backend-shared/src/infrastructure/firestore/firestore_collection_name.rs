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

#[cfg(test)]
mod tests {
    use super::FirestoreCollectionName;

    #[test]
    fn returns_expected_collection_names() {
        assert_eq!(FirestoreCollectionName::IpoStocks.as_str(), "ipo_stocks");
        assert_eq!(FirestoreCollectionName::Exclusions.as_str(), "exclusions");
        assert_eq!(
            FirestoreCollectionName::LotteryApplications.as_str(),
            "lottery_applications"
        );
        assert_eq!(
            FirestoreCollectionName::SecuritiesAccounts.as_str(),
            "securities_accounts"
        );
        assert_eq!(
            FirestoreCollectionName::NotificationSettings.as_str(),
            "notification_settings"
        );
        assert_eq!(
            FirestoreCollectionName::OperationLogs.as_str(),
            "operation_logs"
        );
    }
}
