use crate::{
    acl::notification::NotificationEvent,
    infrastructure::notification::{EmailMessage, SlackMessage},
};

/// Formats notification events into channel-specific payloads.
#[derive(Debug, Default, Clone, Copy)]
pub struct NotificationMessageFormatter;

impl NotificationMessageFormatter {
    /// Formats an event for LINE.
    pub fn format_for_line(event: &NotificationEvent) -> String {
        Self::summary(event)
    }

    /// Formats an event for email.
    pub fn format_for_email(event: &NotificationEvent) -> EmailMessage {
        EmailMessage::new(
            format!("IPOtto notification: {}", event.event_name()),
            Self::summary(event),
        )
    }

    /// Formats an event for Slack.
    pub fn format_for_slack(event: &NotificationEvent) -> SlackMessage {
        SlackMessage::new(Self::summary(event))
    }

    fn summary(event: &NotificationEvent) -> String {
        match event {
            NotificationEvent::ApplicationCompleted(inner) => format!(
                "Application completed: stock={} shares={} price={}",
                inner.stock.value(),
                inner.applied_shares.value(),
                inner.applied_price.value()
            ),
            NotificationEvent::ApplicationFailed(inner) => format!(
                "Application failed: stock={} reason={}",
                inner.stock.value(),
                inner.error_message
            ),
            NotificationEvent::LotteryResultConfirmed(inner) => format!(
                "Lottery result confirmed: stock={} result={:?}",
                inner.stock.value(),
                inner.lottery_result
            ),
            NotificationEvent::IpoInfoUpdated(inner) => format!(
                "IPO stock updated: company={} listing={}",
                inner.company_name.value(),
                inner.listing_date
            ),
            NotificationEvent::OperationErrorOccurred(inner) => format!(
                "Operation error: service={} operation={} reason={}",
                inner.service_name, inner.operation_type, inner.error_message
            ),
            NotificationEvent::ImageAuthenticationFailed(inner) => format!(
                "Image authentication failed: account={} attempts={}",
                inner.securities_account.value(),
                inner.attempt_count
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::{
        acl::notification::NotificationEvent,
        domain::{
            account::SecuritiesAccountIdentifier,
            application::ApplicationIdentifier,
            stock::{Shares, StockIdentifier, Yen},
        },
        events::ApplicationCompleted,
        infrastructure::notification::NotificationMessageFormatter,
    };

    #[test]
    fn formats_application_completed_event() {
        let event = NotificationEvent::ApplicationCompleted(ApplicationCompleted {
            identifier: ApplicationIdentifier::generate(),
            stock: StockIdentifier::generate(),
            securities_account: SecuritiesAccountIdentifier::generate(),
            applied_shares: Shares::new(100).expect("shares"),
            applied_price: Yen::new(1_000).expect("price"),
            applied_at: Utc
                .with_ymd_and_hms(2026, 3, 29, 0, 0, 0)
                .single()
                .expect("timestamp"),
        });
        let line = NotificationMessageFormatter::format_for_line(&event);
        assert!(line.contains("Application completed"));
    }
}
