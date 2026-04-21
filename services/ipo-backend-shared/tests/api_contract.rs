//! Contract tests pinning the serde representation of public-facing domain
//! enums, so `docs/04-api-specification/type-consistency-matrix.md` stays
//! accurate and any future divergence fails CI rather than silently reaching
//! the API surface.
//!
//! These tests intentionally duplicate the canonical string values. Changing
//! one of them here is a signal that the API specification has to change as
//! well (or that a `#[serde(rename...)]` has been introduced to preserve the
//! external contract).

use ipo_backend_shared::domain::{
    application::LotteryResult,
    notification::{ChannelType, NotificationEventType},
    operation_log::OperationEventType,
    stock::{FetchOrigin, Market, StockStatus},
};

fn serialize<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("serialization should not fail")
}

fn deserialize<T: serde::de::DeserializeOwned>(raw: &str) -> T {
    serde_json::from_str(raw).expect("deserialization should not fail")
}

#[test]
fn channel_type_serde_derive_uses_variant_names() {
    assert_eq!(serialize(&ChannelType::Line), "\"Line\"");
    assert_eq!(serialize(&ChannelType::Email), "\"Email\"");
    assert_eq!(serialize(&ChannelType::Slack), "\"Slack\"");
}

#[test]
fn channel_type_as_str_uses_api_expected_labels() {
    // API specification requires `"LINE"` uppercase for LINE Notify.
    assert_eq!(ChannelType::Line.as_str(), "LINE");
    assert_eq!(ChannelType::Email.as_str(), "Email");
    assert_eq!(ChannelType::Slack.as_str(), "Slack");
}

#[test]
fn channel_type_roundtrips_through_serde() {
    for channel in [ChannelType::Line, ChannelType::Email, ChannelType::Slack] {
        let raw = serialize(&channel);
        let restored: ChannelType = deserialize(&raw);
        assert_eq!(restored, channel);
    }
}

#[test]
fn market_serde_derive_matches_as_str() {
    for (market, expected) in [
        (Market::Prime, "Prime"),
        (Market::Standard, "Standard"),
        (Market::Growth, "Growth"),
    ] {
        assert_eq!(serialize(&market), format!("\"{expected}\""));
        assert_eq!(market.as_str(), expected);
    }
}

#[test]
fn market_new_accepts_english_and_japanese_labels() {
    for (input, expected) in [
        ("Prime", Market::Prime),
        ("プライム", Market::Prime),
        ("Standard", Market::Standard),
        ("スタンダード", Market::Standard),
        ("Growth", Market::Growth),
        ("グロース", Market::Growth),
    ] {
        let market = Market::new(input).expect("known market label");
        assert_eq!(market, expected);
    }
}

#[test]
fn stock_status_serde_derive_matches_as_str_and_exposes_all_variants() {
    let all = [
        StockStatus::Fetched,
        StockStatus::Eligible,
        StockStatus::Applied,
        StockStatus::Won,
        StockStatus::Lost,
        StockStatus::Alternate,
        StockStatus::Purchased,
        StockStatus::Declined,
        StockStatus::Sold,
        StockStatus::Excluded,
        StockStatus::Failed,
    ];

    assert_eq!(all.len(), 11, "StockStatus exposes 11 variants");

    for status in all {
        let raw = serialize(&status);
        assert_eq!(
            raw,
            format!("\"{}\"", status.as_str()),
            "serde derive must match as_str for StockStatus::{status:?}"
        );
        let restored: StockStatus = deserialize(&raw);
        assert_eq!(restored, status);
    }
}

#[test]
fn notification_event_type_serde_derive_uses_pascal_case() {
    for event in [
        NotificationEventType::ApplicationCompleted,
        NotificationEventType::LotteryResultWon,
        NotificationEventType::LotteryResultLost,
        NotificationEventType::OperationError,
        NotificationEventType::StockUpdated,
    ] {
        let raw = serialize(&event);
        let restored: NotificationEventType = deserialize(&raw);
        assert_eq!(restored, event);
    }

    assert_eq!(
        serialize(&NotificationEventType::ApplicationCompleted),
        "\"ApplicationCompleted\""
    );
    assert_eq!(
        serialize(&NotificationEventType::LotteryResultWon),
        "\"LotteryResultWon\""
    );
}

#[test]
fn operation_event_type_serde_and_as_str_agree_on_snake_case() {
    // Canonical form picked by
    // `fix(backend-shared): canonical serde form for OperationEventType`:
    // both serde derive (via `#[serde(rename_all = "snake_case")]`) and
    // `as_str()` expose the snake_case spelling that `GET /api/v1/logs`
    // returns and accepts as the `eventType` query parameter.
    for (variant, expected) in [
        (OperationEventType::FetchStocks, "fetch_stocks"),
        (OperationEventType::ApplyLottery, "apply_lottery"),
        (
            OperationEventType::CheckLotteryResult,
            "check_lottery_result",
        ),
        (
            OperationEventType::NotificationDispatch,
            "notification_dispatch",
        ),
        (OperationEventType::ConnectionTest, "connection_test"),
        (OperationEventType::Other, "other"),
    ] {
        assert_eq!(serialize(&variant), format!("\"{expected}\""));
        assert_eq!(variant.as_str(), expected);
        let restored: OperationEventType = deserialize(&format!("\"{expected}\""));
        assert_eq!(restored, variant);
    }
}

#[test]
fn fetch_origin_serde_derive_is_pascal_case() {
    for origin in [FetchOrigin::ExternalSite, FetchOrigin::SecuritiesSite] {
        let raw = serialize(&origin);
        let restored: FetchOrigin = deserialize(&raw);
        assert_eq!(restored, origin);
    }

    assert_eq!(
        serialize(&FetchOrigin::ExternalSite),
        "\"ExternalSite\"",
        "FetchOrigin::ExternalSite must serialize as \"ExternalSite\""
    );
    assert_eq!(
        serialize(&FetchOrigin::SecuritiesSite),
        "\"SecuritiesSite\"",
        "FetchOrigin::SecuritiesSite must serialize as \"SecuritiesSite\""
    );
}

#[test]
fn lottery_result_serde_matches_api_specification_labels() {
    for (result, expected) in [
        (LotteryResult::Won, "\"Won\""),
        (LotteryResult::Lost, "\"Lost\""),
        (LotteryResult::Alternate, "\"Alternate\""),
    ] {
        let raw = serialize(&result);
        assert_eq!(raw, expected);
        let restored: LotteryResult = deserialize(&raw);
        assert_eq!(restored, result);
    }
}
