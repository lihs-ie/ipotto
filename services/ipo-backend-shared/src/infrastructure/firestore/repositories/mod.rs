// In-memory repository implementations are kept behind the `test-support`
// feature flag (or the crate's own `cfg(test)` builds) so production
// builds of downstream services cannot accidentally fall back to the
// in-memory doubles. The real Firestore-backed implementations live in
// sibling modules and are always available.
#[cfg(any(test, feature = "test-support"))]
pub mod firestore_exclusion_repository_in_memory;
#[cfg(any(test, feature = "test-support"))]
pub mod firestore_ipo_stock_repository_in_memory;
#[cfg(any(test, feature = "test-support"))]
pub mod firestore_lottery_application_repository_in_memory;
#[cfg(any(test, feature = "test-support"))]
pub mod firestore_notification_setting_repository_in_memory;
#[cfg(any(test, feature = "test-support"))]
pub mod firestore_operation_log_repository_in_memory;
#[cfg(any(test, feature = "test-support"))]
pub mod firestore_securities_account_repository_in_memory;

#[cfg(any(test, feature = "test-support"))]
pub use firestore_exclusion_repository_in_memory::FirestoreExclusionRepositoryInMemory;
#[cfg(any(test, feature = "test-support"))]
pub use firestore_ipo_stock_repository_in_memory::FirestoreIpoStockRepositoryInMemory;
#[cfg(any(test, feature = "test-support"))]
pub use firestore_lottery_application_repository_in_memory::FirestoreLotteryApplicationRepositoryInMemory;
#[cfg(any(test, feature = "test-support"))]
pub use firestore_notification_setting_repository_in_memory::FirestoreNotificationSettingRepositoryInMemory;
#[cfg(any(test, feature = "test-support"))]
pub use firestore_operation_log_repository_in_memory::FirestoreOperationLogRepositoryInMemory;
#[cfg(any(test, feature = "test-support"))]
pub use firestore_securities_account_repository_in_memory::FirestoreSecuritiesAccountRepositoryInMemory;

pub mod firestore_exclusion_repository;
pub mod firestore_ipo_stock_repository;
pub mod firestore_lottery_application_repository;
pub mod firestore_notification_setting_repository;
pub mod firestore_operation_log_repository;
pub mod firestore_securities_account_repository;

pub use firestore_exclusion_repository::FirestoreExclusionRepository;
pub use firestore_ipo_stock_repository::FirestoreIpoStockRepository;
pub use firestore_lottery_application_repository::FirestoreLotteryApplicationRepository;
pub use firestore_notification_setting_repository::FirestoreNotificationSettingRepository;
pub use firestore_operation_log_repository::FirestoreOperationLogRepository;
pub use firestore_securities_account_repository::FirestoreSecuritiesAccountRepository;
