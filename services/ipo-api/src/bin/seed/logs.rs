//! Seed data for `operation_logs`. Produces 12 entries over the last
//! seven days covering every `OperationEventType`, with a mix of
//! Succeeded and Failed statuses so the logs page shows an active
//! history.

use chrono::{Duration, Utc};
use ipo_backend_shared::{
    domain::{
        application::ApplicationIdentifier,
        operation_log::{
            OperationEventType, OperationLog, OperationLogIdentifier, OperationLogPayload,
            OperationStatus,
        },
    },
    errors::DomainError,
};
use ulid::Ulid;

pub fn build_logs(
    applications: &[ApplicationIdentifier],
) -> Result<Vec<OperationLog>, DomainError> {
    let now = Utc::now();

    let entries: [LogSeed; 12] = [
        LogSeed {
            index: 1,
            application_index: None,
            event_type: OperationEventType::FetchStocks,
            service_name: "ipo-info-fetcher",
            status: OperationStatus::Succeeded,
            message: "6 件の銘柄情報を取得しました",
            error_message: None,
            offset_hours: -3,
        },
        LogSeed {
            index: 2,
            application_index: None,
            event_type: OperationEventType::FetchStocks,
            service_name: "ipo-info-fetcher",
            status: OperationStatus::Succeeded,
            message: "2 件の銘柄情報を更新しました",
            error_message: None,
            offset_hours: -27,
        },
        LogSeed {
            index: 3,
            application_index: Some(0),
            event_type: OperationEventType::ApplyLottery,
            service_name: "ipo-applier",
            status: OperationStatus::Succeeded,
            message: "楽天証券で申込を完了しました",
            error_message: None,
            offset_hours: -30,
        },
        LogSeed {
            index: 4,
            application_index: Some(1),
            event_type: OperationEventType::CheckLotteryResult,
            service_name: "ipo-result-checker",
            status: OperationStatus::Succeeded,
            message: "当選結果を確認しました (Won)",
            error_message: None,
            offset_hours: -48,
        },
        LogSeed {
            index: 5,
            application_index: Some(2),
            event_type: OperationEventType::CheckLotteryResult,
            service_name: "ipo-result-checker",
            status: OperationStatus::Succeeded,
            message: "抽選結果を確認しました (Lost)",
            error_message: None,
            offset_hours: -50,
        },
        LogSeed {
            index: 6,
            application_index: None,
            event_type: OperationEventType::ConnectionTest,
            service_name: "ipo-browser",
            status: OperationStatus::Succeeded,
            message: "楽天証券への接続テストに成功しました",
            error_message: None,
            offset_hours: -70,
        },
        LogSeed {
            index: 7,
            application_index: None,
            event_type: OperationEventType::NotificationDispatch,
            service_name: "ipo-api",
            status: OperationStatus::Succeeded,
            message: "LINE 通知を配信しました (ApplicationCompleted)",
            error_message: None,
            offset_hours: -72,
        },
        LogSeed {
            index: 8,
            application_index: None,
            event_type: OperationEventType::NotificationDispatch,
            service_name: "ipo-api",
            status: OperationStatus::Failed,
            message: "Email 通知の送信に失敗しました",
            error_message: Some("SendGrid 429 Too Many Requests".to_string()),
            offset_hours: -80,
        },
        LogSeed {
            index: 9,
            application_index: Some(3),
            event_type: OperationEventType::ApplyLottery,
            service_name: "ipo-applier",
            status: OperationStatus::Failed,
            message: "申込処理中にエラーが発生しました",
            error_message: Some("navigation timeout: 30000ms exceeded".to_string()),
            offset_hours: -96,
        },
        LogSeed {
            index: 10,
            application_index: None,
            event_type: OperationEventType::FetchStocks,
            service_name: "ipo-info-fetcher",
            status: OperationStatus::Failed,
            message: "外部サイトからの情報取得に失敗しました",
            error_message: Some("upstream returned 503 Service Unavailable".to_string()),
            offset_hours: -120,
        },
        LogSeed {
            index: 11,
            application_index: None,
            event_type: OperationEventType::Other,
            service_name: "ipo-api",
            status: OperationStatus::Succeeded,
            message: "メンテナンスバッチが正常に完了しました",
            error_message: None,
            offset_hours: -144,
        },
        LogSeed {
            index: 12,
            application_index: None,
            event_type: OperationEventType::ConnectionTest,
            service_name: "ipo-browser",
            status: OperationStatus::Failed,
            message: "SBI証券への接続テストに失敗しました",
            error_message: Some("login form captcha challenge".to_string()),
            offset_hours: -160,
        },
    ];

    entries
        .into_iter()
        .map(|seed| seed.into_log(now, applications))
        .collect()
}

struct LogSeed {
    index: u8,
    application_index: Option<usize>,
    event_type: OperationEventType,
    service_name: &'static str,
    status: OperationStatus,
    message: &'static str,
    error_message: Option<String>,
    offset_hours: i64,
}

impl LogSeed {
    fn into_log(
        self,
        now: chrono::DateTime<Utc>,
        applications: &[ApplicationIdentifier],
    ) -> Result<OperationLog, DomainError> {
        let application = self
            .application_index
            .and_then(|index| applications.get(index).cloned());
        let payload = OperationLogPayload::new(
            application,
            self.event_type,
            self.service_name,
            self.status,
            self.message,
            self.error_message,
            now + Duration::hours(self.offset_hours),
        );
        OperationLog::reconstruct(operation_log_identifier(self.index)?, payload)
    }
}

fn operation_log_identifier(index: u8) -> Result<OperationLogIdentifier, DomainError> {
    let ulid = Ulid::from_parts(1_700_000_050_000, u128::from(index));
    OperationLogIdentifier::new(ulid.to_string())
}
