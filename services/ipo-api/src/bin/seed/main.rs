//! Local development seed binary.
//!
//! Populates the Firestore emulator with realistic demo data so the
//! dashboard and other authenticated pages show meaningful content
//! without hitting the production ingestion pipeline. Invoke via
//! `make seed` / `make seed-reset`. Refuses to run unless
//! `FIRESTORE_EMULATOR_HOST` is set, to avoid touching real Firestore.
//!
//! Reset semantics: this binary always writes with fixed identifiers so
//! re-running it is a safe upsert. The `make seed-reset` target
//! additionally calls the emulator REST API
//! (`DELETE /emulator/v1/projects/<id>/databases/(default)/documents`)
//! to wipe every collection before seeding, in case previous runs left
//! documents under different identifiers.

use std::env;
use std::sync::Arc;

use ipo_backend_shared::{
    domain::{
        application::LotteryApplicationRepository, exclusion::ExclusionRepository,
        notification::NotificationSettingRepository, operation_log::OperationLogRepository,
        stock::IpoStockRepository,
    },
    errors::DomainError,
    infrastructure::firestore::{
        build_firestore_client,
        repositories::{
            FirestoreExclusionRepository, FirestoreIpoStockRepository,
            FirestoreLotteryApplicationRepository, FirestoreNotificationSettingRepository,
            FirestoreOperationLogRepository,
        },
    },
};

mod accounts;
mod applications;
mod channels;
mod exclusions;
mod logs;
mod stocks;

const PROJECT_ID: &str = "ipotto-local";

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("seed failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), DomainError> {
    if env::var("FIRESTORE_EMULATOR_HOST").is_err() {
        return Err(DomainError::FirestoreMappingError {
            reason: "FIRESTORE_EMULATOR_HOST is not set — refusing to seed non-emulator Firestore"
                .to_string(),
        });
    }

    let project_id = env::var("FIREBASE_PROJECT_ID").unwrap_or_else(|_| PROJECT_ID.to_string());
    let db = Arc::new(build_firestore_client(&project_id).await?);

    let stock_repository = Arc::new(FirestoreIpoStockRepository::new(db.clone()));
    let exclusion_repository = Arc::new(FirestoreExclusionRepository::new(db.clone()));
    let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new(db.clone()));
    let notification_setting_repository =
        Arc::new(FirestoreNotificationSettingRepository::new(db.clone()));
    let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new(db));

    println!("-- seeding stocks");
    let stocks = stocks::build_stocks()?;
    for stock in &stocks {
        stock_repository.save(stock).await?;
    }
    println!("   wrote {} stocks", stocks.len());

    println!("-- seeding exclusions");
    let exclusions_data = exclusions::build_exclusions()?;
    for exclusion in &exclusions_data {
        exclusion_repository.save(exclusion).await?;
    }
    println!("   wrote {} exclusions", exclusions_data.len());

    println!("-- skipping securities accounts (credential store cannot be shared across processes)");
    println!("   register Rakuten account from the UI to exercise connection tests");

    println!("-- seeding notification setting (with channels)");
    let setting = channels::build_notification_setting()?;
    notification_setting_repository.save(&setting).await?;
    println!("   wrote {} channels", setting.channels().len());

    println!("-- seeding lottery applications");
    let applications_data =
        applications::build_applications(&accounts::placeholder_account_identifier()?, &stocks)?;
    for application in &applications_data {
        application_repository.save(application).await?;
    }
    println!("   wrote {} applications", applications_data.len());

    println!("-- seeding operation logs");
    let logs_data = logs::build_logs(
        &applications_data
            .iter()
            .map(|application| application.identifier().clone())
            .collect::<Vec<_>>(),
    )?;
    for log in &logs_data {
        operation_log_repository.save(log).await?;
    }
    println!("   wrote {} logs", logs_data.len());

    println!("seed complete");
    Ok(())
}
