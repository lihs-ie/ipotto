// Test doubles exposed so downstream services can build integration
// tests without spinning up Firestore / Pub/Sub / Secret Manager
// clients. Production code paths must not import from this module.
pub use crate::infrastructure::{
    firestore::repositories::{
        FirestoreExclusionRepositoryInMemory, FirestoreIpoStockRepositoryInMemory,
        FirestoreLotteryApplicationRepositoryInMemory,
        FirestoreNotificationSettingRepositoryInMemory, FirestoreOperationLogRepositoryInMemory,
        FirestoreSecuritiesAccountRepositoryInMemory,
    },
    messaging::PubSubEventPublisher,
    secrets::InMemoryCredentialStore,
};
