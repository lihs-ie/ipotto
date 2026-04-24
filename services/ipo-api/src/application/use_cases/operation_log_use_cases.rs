use std::{cmp::Reverse, sync::Arc};

use chrono::{NaiveDate, TimeZone, Utc};
use ipo_backend_shared::{
    domain::operation_log::{OperationEventType, OperationLogRepository},
    errors::DomainError,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOperationLogsInput {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub event_type: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationLogOutput {
    pub identifier: String,
    pub event_type: String,
    pub service_name: String,
    pub status: String,
    pub message: String,
    pub error_message: Option<String>,
    pub executed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOperationLogsOutput {
    pub items: Vec<OperationLogOutput>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

pub struct ListOperationLogsUseCase {
    repository: Arc<dyn OperationLogRepository + Send + Sync>,
}

impl ListOperationLogsUseCase {
    pub fn new(repository: Arc<dyn OperationLogRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: ListOperationLogsInput,
    ) -> Result<ListOperationLogsOutput, DomainError> {
        let start = input
            .start_date
            .as_deref()
            .map(parse_start_date)
            .transpose()?;
        let end = input.end_date.as_deref().map(parse_end_date).transpose()?;
        let event_type = input
            .event_type
            .as_deref()
            .map(parse_event_type)
            .transpose()?;

        let mut logs = match (start, end, event_type) {
            (Some(start), Some(end), Some(event_type)) => {
                let mut logs = self.repository.find_by_date_range(start, end).await?;
                logs.retain(|log| log.event_type() == event_type);
                logs
            }
            (Some(start), Some(end), None) => {
                self.repository.find_by_date_range(start, end).await?
            }
            (None, None, Some(event_type)) => {
                self.repository.find_by_event_type(event_type).await?
            }
            (start, end, event_type) => {
                let mut logs = self.repository.find_all().await?;
                if let Some(start) = start {
                    logs.retain(|log| log.executed_at() >= start);
                }
                if let Some(end) = end {
                    logs.retain(|log| log.executed_at() <= end);
                }
                if let Some(event_type) = event_type {
                    logs.retain(|log| log.event_type() == event_type);
                }
                logs
            }
        };

        logs.sort_by_key(|log| Reverse(log.executed_at()));
        if let Some(cursor) = input.cursor {
            if let Some(position) = logs
                .iter()
                .position(|log| log.identifier().value() == cursor)
            {
                logs = logs.into_iter().skip(position + 1).collect();
            }
        }

        let limit = input.limit.unwrap_or(20).min(100);
        let has_more = logs.len() > limit;
        let items = logs
            .into_iter()
            .take(limit)
            .map(|log| OperationLogOutput {
                identifier: log.identifier().value().to_string(),
                event_type: log.event_type().as_str().to_string(),
                service_name: log.service_name().to_string(),
                status: log.status().as_str().to_string(),
                message: log.message().to_string(),
                error_message: log.error_message().map(ToString::to_string),
                executed_at: log.executed_at().to_rfc3339(),
            })
            .collect::<Vec<_>>();

        let next_cursor = if has_more {
            items.last().map(|item| item.identifier.clone())
        } else {
            None
        };

        Ok(ListOperationLogsOutput {
            has_more,
            next_cursor,
            items,
        })
    }
}

fn parse_start_date(value: &str) -> Result<chrono::DateTime<Utc>, DomainError> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|error| {
        DomainError::OperationLogValidationError {
            reason: error.to_string(),
        }
    })?;
    let naive =
        date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| DomainError::OperationLogValidationError {
                reason: "invalid start date".to_string(),
            })?;
    Ok(Utc.from_utc_datetime(&naive))
}

fn parse_end_date(value: &str) -> Result<chrono::DateTime<Utc>, DomainError> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|error| {
        DomainError::OperationLogValidationError {
            reason: error.to_string(),
        }
    })?;
    let naive =
        date.and_hms_opt(23, 59, 59)
            .ok_or_else(|| DomainError::OperationLogValidationError {
                reason: "invalid end date".to_string(),
            })?;
    Ok(Utc.from_utc_datetime(&naive))
}

fn parse_event_type(value: &str) -> Result<OperationEventType, DomainError> {
    match value {
        "fetch_stocks" => Ok(OperationEventType::FetchStocks),
        "apply_lottery" => Ok(OperationEventType::ApplyLottery),
        "check_lottery_result" => Ok(OperationEventType::CheckLotteryResult),
        "notification_dispatch" => Ok(OperationEventType::NotificationDispatch),
        "connection_test" => Ok(OperationEventType::ConnectionTest),
        "other" => Ok(OperationEventType::Other),
        other => Err(DomainError::OperationLogValidationError {
            reason: format!("unsupported event type: {other}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use chrono::{TimeZone, Utc};
    use ipo_backend_shared::domain::operation_log::{
        OperationEventType, OperationLog, OperationLogPayload, OperationLogRepository,
        OperationStatus,
    };

    use super::{ListOperationLogsInput, ListOperationLogsUseCase};

    #[derive(Debug, Default)]
    struct CountingRepository {
        logs: Vec<OperationLog>,
        find_all_calls: AtomicUsize,
        find_by_date_range_calls: AtomicUsize,
        find_by_event_type_calls: AtomicUsize,
    }

    impl CountingRepository {
        fn new(logs: Vec<OperationLog>) -> Self {
            Self {
                logs,
                ..Self::default()
            }
        }
    }

    #[async_trait::async_trait]
    impl OperationLogRepository for CountingRepository {
        async fn save(
            &self,
            _log: &OperationLog,
        ) -> Result<(), ipo_backend_shared::errors::DomainError> {
            Ok(())
        }

        async fn find_all(
            &self,
        ) -> Result<Vec<OperationLog>, ipo_backend_shared::errors::DomainError> {
            self.find_all_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.logs.clone())
        }

        async fn find_by_date_range(
            &self,
            start: chrono::DateTime<Utc>,
            end: chrono::DateTime<Utc>,
        ) -> Result<Vec<OperationLog>, ipo_backend_shared::errors::DomainError> {
            self.find_by_date_range_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self
                .logs
                .iter()
                .filter(|log| log.executed_at() >= start && log.executed_at() <= end)
                .cloned()
                .collect())
        }

        async fn find_by_event_type(
            &self,
            event_type: OperationEventType,
        ) -> Result<Vec<OperationLog>, ipo_backend_shared::errors::DomainError> {
            self.find_by_event_type_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self
                .logs
                .iter()
                .filter(|log| log.event_type() == event_type)
                .cloned()
                .collect())
        }
    }

    fn build_log(event_type: OperationEventType, day: u32) -> OperationLog {
        OperationLog::create(OperationLogPayload::new(
            None,
            event_type,
            "ipo-api",
            OperationStatus::Succeeded,
            "ok",
            None,
            Utc.with_ymd_and_hms(2026, 4, day, 10, 0, 0)
                .single()
                .expect("timestamp"),
        ))
        .expect("log")
    }

    #[tokio::test]
    async fn uses_repository_level_filtering_before_in_memory_filtering() {
        let repository = Arc::new(CountingRepository::new(vec![
            build_log(OperationEventType::FetchStocks, 1),
            build_log(OperationEventType::ConnectionTest, 2),
        ]));
        let use_case = ListOperationLogsUseCase::new(repository.clone());

        let output = use_case
            .execute(ListOperationLogsInput {
                start_date: Some("2026-04-01".to_string()),
                end_date: Some("2026-04-30".to_string()),
                event_type: Some("fetch_stocks".to_string()),
                cursor: None,
                limit: None,
            })
            .await
            .expect("execute");

        assert_eq!(output.items.len(), 1);
        assert_eq!(repository.find_all_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            repository.find_by_date_range_calls.load(Ordering::SeqCst),
            1
        );
        assert_eq!(
            repository.find_by_event_type_calls.load(Ordering::SeqCst),
            0
        );
    }
}
