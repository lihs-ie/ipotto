//! Seed data for `notification_settings` (aggregate with channels).
//! Registers a LINE and Email channel with subscriptions to every
//! NotificationEventType, and enables notification dispatch so the
//! settings page shows realistic state.

use std::collections::BTreeMap;

use ipo_backend_shared::{
    domain::notification::{
        ChannelDestination, ChannelIdentifier, ChannelType, NotificationChannel,
        NotificationEventType, NotificationSetting, NotificationSettingIdentifier,
    },
    errors::DomainError,
};
use ulid::Ulid;

pub fn build_notification_setting() -> Result<NotificationSetting, DomainError> {
    let channels = vec![build_line_channel()?, build_email_channel()?];
    NotificationSetting::reconstruct(NotificationSettingIdentifier::default_id(), true, channels)
}

fn build_line_channel() -> Result<NotificationChannel, DomainError> {
    let mut destination_values = BTreeMap::new();
    destination_values.insert("token".to_string(), "seed-line-token-DEMO".to_string());
    let destination = ChannelDestination::new(ChannelType::Line, destination_values)?;

    NotificationChannel::reconstruct(
        channel_identifier(1)?,
        ChannelType::Line,
        destination,
        true,
        all_event_subscriptions(),
    )
}

fn build_email_channel() -> Result<NotificationChannel, DomainError> {
    let mut destination_values = BTreeMap::new();
    destination_values.insert("address".to_string(), "notify@example.com".to_string());
    let destination = ChannelDestination::new(ChannelType::Email, destination_values)?;

    NotificationChannel::reconstruct(
        channel_identifier(2)?,
        ChannelType::Email,
        destination,
        true,
        all_event_subscriptions(),
    )
}

fn all_event_subscriptions() -> BTreeMap<NotificationEventType, bool> {
    let mut map = BTreeMap::new();
    map.insert(NotificationEventType::ApplicationCompleted, true);
    map.insert(NotificationEventType::LotteryResultWon, true);
    map.insert(NotificationEventType::LotteryResultLost, true);
    map.insert(NotificationEventType::OperationError, true);
    map.insert(NotificationEventType::StockUpdated, false);
    map
}

fn channel_identifier(index: u8) -> Result<ChannelIdentifier, DomainError> {
    let ulid = Ulid::from_parts(1_700_000_030_000, u128::from(index));
    ChannelIdentifier::new(ulid.to_string())
}
