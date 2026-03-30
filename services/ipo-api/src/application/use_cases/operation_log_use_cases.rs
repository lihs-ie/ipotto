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

    pub fn execute(
        &self,
        input: ListOperationLogsInput,
    ) -> Result<ListOperationLogsOutput, DomainError> {
        let mut logs = self.repository.find_all()?;
        if let Some(start_date) = input.start_date {
            let start = parse_start_date(&start_date)?;
            logs.retain(|log| log.executed_at() >= start);
        }
        if let Some(end_date) = input.end_date {
            let end = parse_end_date(&end_date)?;
            logs.retain(|log| log.executed_at() <= end);
        }
        if let Some(event_type) = input.event_type {
            let event_type = parse_event_type(&event_type)?;
            logs.retain(|log| log.event_type() == event_type);
        }

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
