//! Integration tests for the production gRPC-backed Firestore repositories.
//!
//! These tests require a Firestore emulator reachable on `FIRESTORE_EMULATOR_HOST`
//! (docker-compose + CI `rust-lint-test` job wire one up automatically). When the
//! env var is missing the tests become no-ops so they can still run in
//! environments without an emulator (e.g. `cargo test` on a developer laptop
//! without docker-compose running).

use std::sync::Arc;

use chrono::{NaiveDate, TimeZone, Utc};
use ipo_backend_shared::{
    acl::secrets::CredentialStorePort,
    domain::{
        account::{
            AccountCredential, ImapHost, ImapPort, LoginId, LoginPassword, MailAddress,
            MailCredential, MailPassword, SecuritiesAccount, SecuritiesAccountRepository,
            SecuritiesCompany, TradingPassword,
        },
        application::{
            ApplicationStatus, LotteryApplication, LotteryApplicationRepository, LotteryResult,
        },
        exclusion::{Exclusion, ExclusionReason, ExclusionRepository},
        notification::{
            ChannelDestination, ChannelType, NotificationChannel, NotificationEventType,
            NotificationSetting, NotificationSettingRepository,
        },
        operation_log::{
            OperationEventType, OperationLog, OperationLogPayload, OperationLogRepository,
            OperationStatus,
        },
        stock::{
            BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry, IpoOffering,
            IpoPricing, IpoSchedule, IpoStock, IpoStockRepository, LeadUnderwriter, Market,
            MetaSource, PriceRange, Shares, StockStatus, Yen,
        },
    },
    infrastructure::{
        crypto::{EncryptedCredentialStore, InMemoryKeyManagement},
        firestore::{
            build_firestore_client,
            repositories::{
                FirestoreExclusionRepository, FirestoreIpoStockRepository,
                FirestoreLotteryApplicationRepository, FirestoreNotificationSettingRepository,
                FirestoreOperationLogRepository, FirestoreSecuritiesAccountRepository,
            },
        },
        secrets::InMemoryCredentialStore,
    },
};

fn emulator_available() -> bool {
    std::env::var("FIRESTORE_EMULATOR_HOST").is_ok()
}

async fn new_db() -> Arc<firestore::FirestoreDb> {
    Arc::new(
        build_firestore_client("ipotto-integration")
            .await
            .expect("Firestore emulator client"),
    )
}

fn build_stock(status: StockStatus) -> IpoStock {
    IpoStock::create(
        CompanyProfile::new(
            CompanyName::new("統合テスト株式会社").expect("name"),
            None,
            Market::Growth,
            Industry::new("情報・通信業").expect("industry"),
        )
        .expect("profile"),
        IpoSchedule::new(
            BookBuildingPeriod::new(
                NaiveDate::from_ymd_opt(2026, 4, 1).expect("bb start"),
                NaiveDate::from_ymd_opt(2026, 4, 10).expect("bb end"),
            )
            .expect("period"),
            NaiveDate::from_ymd_opt(2026, 4, 15).expect("lottery"),
            NaiveDate::from_ymd_opt(2026, 4, 25).expect("listing"),
        )
        .expect("schedule"),
        IpoPricing::new(
            PriceRange::new(Yen::new(1200).expect("min"), Yen::new(1500).expect("max"))
                .expect("range"),
            Some(Yen::new(1400).expect("offer")),
        )
        .expect("pricing"),
        IpoOffering::new(
            LeadUnderwriter::new("楽天証券").expect("underwriter"),
            Shares::new(100_000).expect("shares"),
        )
        .expect("offering"),
        status,
        MetaSource::new(FetchOrigin::ExternalSite, Utc::now()),
    )
    .expect("stock")
}

fn build_account(active: bool) -> SecuritiesAccount {
    let mut account = SecuritiesAccount::create(
        SecuritiesCompany::Rakuten,
        AccountCredential::new(
            LoginId::new("login").expect("login id"),
            LoginPassword::new("password").expect("login password"),
            TradingPassword::new("1234").expect("trading password"),
            MailCredential::new(
                MailAddress::new("integ@example.com").expect("mail"),
                MailPassword::new("mail-password").expect("mail password"),
                ImapHost::new("imap.example.com").expect("host"),
                ImapPort::new(993).expect("port"),
            )
            .expect("mail credential"),
        )
        .expect("credential"),
    )
    .expect("account");
    if !active {
        account.deactivate();
    }
    account
}

#[tokio::test]
async fn firestore_ipo_stock_repository_roundtrips_stock() {
    if !emulator_available() {
        eprintln!("skipping: FIRESTORE_EMULATOR_HOST not set");
        return;
    }
    let db = new_db().await;
    let repository = FirestoreIpoStockRepository::new(db);
    let stock = build_stock(StockStatus::Eligible);

    repository.save(&stock).await.expect("save");
    let fetched = repository
        .find_by_id(stock.identifier())
        .await
        .expect("find")
        .expect("stock exists");
    assert_eq!(fetched.identifier(), stock.identifier());

    let by_status = repository
        .find_by_status(StockStatus::Eligible)
        .await
        .expect("find by status");
    assert!(by_status
        .iter()
        .any(|candidate| candidate.identifier() == stock.identifier()));

    let in_period = repository
        .find_in_book_building_period(NaiveDate::from_ymd_opt(2026, 4, 5).expect("date"))
        .await
        .expect("find in period");
    assert!(in_period
        .iter()
        .any(|candidate| candidate.identifier() == stock.identifier()));

    let all = repository.find_all().await.expect("find all");
    assert!(all
        .iter()
        .any(|candidate| candidate.identifier() == stock.identifier()));
}

#[tokio::test]
async fn firestore_lottery_application_repository_roundtrips_application() {
    if !emulator_available() {
        return;
    }
    let db = new_db().await;
    let repository = FirestoreLotteryApplicationRepository::new(db);
    let stock_identifier = build_stock(StockStatus::Applied).identifier().clone();
    let account_identifier = build_account(true).identifier().clone();
    let mut application = LotteryApplication::create_with_values(
        stock_identifier.clone(),
        account_identifier.clone(),
        Shares::new(100).expect("shares"),
        Yen::new(1400).expect("price"),
        Utc.with_ymd_and_hms(2026, 4, 5, 10, 0, 0)
            .single()
            .expect("ordered at"),
    )
    .expect("application");
    application.apply().expect("apply");
    application
        .record_outcome(LotteryResult::Won, Utc::now())
        .expect("record outcome");

    repository.save(&application).await.expect("save");
    let fetched = repository
        .find_by_id(application.identifier())
        .await
        .expect("find")
        .expect("application exists");
    assert_eq!(fetched.status(), ApplicationStatus::ResultChecked);

    let by_stock = repository
        .find_by_stock(&stock_identifier)
        .await
        .expect("find by stock");
    assert!(by_stock
        .iter()
        .any(|candidate| candidate.identifier() == application.identifier()));

    assert!(repository
        .exists_by_stock_and_account(&stock_identifier, &account_identifier)
        .await
        .expect("exists"));
}

#[tokio::test]
async fn firestore_securities_account_repository_roundtrips_account() {
    if !emulator_available() {
        return;
    }
    let db = new_db().await;
    let credential_store: Arc<dyn CredentialStorePort> = Arc::new(EncryptedCredentialStore::new(
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(InMemoryKeyManagement::random_for_test()),
    ));
    let repository = FirestoreSecuritiesAccountRepository::new(db, credential_store);
    let account = build_account(true);

    repository.save(&account).await.expect("save");
    let fetched = repository
        .find_by_id(account.identifier())
        .await
        .expect("find")
        .expect("account exists");
    assert_eq!(fetched.identifier(), account.identifier());
    assert_eq!(fetched.credential().login_id().value(), "login");

    let active = repository.find_all().await.expect("find all");
    assert!(active
        .iter()
        .any(|candidate| candidate.identifier() == account.identifier()));

    repository
        .delete(account.identifier())
        .await
        .expect("delete");
    assert!(repository
        .find_by_id(account.identifier())
        .await
        .expect("find after delete")
        .is_none());
}

#[tokio::test]
async fn firestore_exclusion_repository_roundtrips_exclusion() {
    if !emulator_available() {
        return;
    }
    let db = new_db().await;
    let repository = FirestoreExclusionRepository::new(db);
    let company_name = CompanyName::new("除外対象株式会社").expect("company");
    let exclusion = Exclusion::create(
        company_name.clone(),
        ExclusionReason::new("統合テスト").expect("reason"),
        Utc::now(),
    )
    .expect("exclusion");

    repository.save(&exclusion).await.expect("save");
    assert!(repository
        .exists_by_company_name(&company_name)
        .await
        .expect("exists"));
    let fetched = repository
        .find_by_id(exclusion.identifier())
        .await
        .expect("find")
        .expect("exclusion exists");
    assert_eq!(fetched.identifier(), exclusion.identifier());

    let all = repository.find_all().await.expect("find all");
    assert!(all
        .iter()
        .any(|candidate| candidate.identifier() == exclusion.identifier()));

    repository
        .delete(exclusion.identifier())
        .await
        .expect("delete");
}

#[tokio::test]
async fn firestore_operation_log_repository_roundtrips_log() {
    if !emulator_available() {
        return;
    }
    let db = new_db().await;
    let repository = FirestoreOperationLogRepository::new(db);
    let log = OperationLog::create(OperationLogPayload::new(
        None,
        OperationEventType::FetchStocks,
        "integration-test",
        OperationStatus::Succeeded,
        "integration log",
        None,
        Utc.with_ymd_and_hms(2026, 4, 5, 10, 0, 0)
            .single()
            .expect("executed at"),
    ))
    .expect("log");

    repository.save(&log).await.expect("save");
    let all = repository.find_all().await.expect("find all");
    assert!(all
        .iter()
        .any(|candidate| candidate.identifier() == log.identifier()));

    let by_event = repository
        .find_by_event_type(OperationEventType::FetchStocks)
        .await
        .expect("find by event type");
    assert!(by_event
        .iter()
        .any(|candidate| candidate.identifier() == log.identifier()));

    let by_range = repository
        .find_by_date_range(
            Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0)
                .single()
                .expect("start"),
            Utc.with_ymd_and_hms(2026, 4, 30, 0, 0, 0)
                .single()
                .expect("end"),
        )
        .await
        .expect("find by range");
    assert!(by_range
        .iter()
        .any(|candidate| candidate.identifier() == log.identifier()));
}

#[tokio::test]
async fn firestore_notification_setting_repository_roundtrips_setting() {
    if !emulator_available() {
        return;
    }
    let db = new_db().await;
    let repository = FirestoreNotificationSettingRepository::new(db);
    let mut destination = std::collections::BTreeMap::new();
    destination.insert("address".to_string(), "notify@example.com".to_string());
    let mut subscriptions = std::collections::BTreeMap::new();
    subscriptions.insert(NotificationEventType::ApplicationCompleted, true);
    let channel = NotificationChannel::create(
        ChannelType::Email,
        ChannelDestination::new(ChannelType::Email, destination).expect("destination"),
        true,
        subscriptions,
    )
    .expect("channel");
    let mut setting = NotificationSetting::create(vec![channel]).expect("setting");
    setting.enable().expect("enable");

    repository.save(&setting).await.expect("save");
    let fetched = repository
        .find_by_id(setting.identifier())
        .await
        .expect("find")
        .expect("setting exists");
    assert_eq!(fetched.channels().len(), 1);
    assert_eq!(fetched.channels()[0].channel_type(), ChannelType::Email);

    let default = repository.find_default().await.expect("default");
    assert_eq!(default.identifier(), setting.identifier());
}
