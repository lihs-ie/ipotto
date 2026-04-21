use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::{
    application::use_cases::{
        ListOperationLogsInput, ListOperationLogsOutput, ListOperationLogsUseCase,
    },
    infrastructure::DependencyContainer,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOperationLogsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub event_type: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

pub async fn list_logs(
    State(container): State<DependencyContainer>,
    Query(query): Query<ListOperationLogsQuery>,
) -> Result<Json<ListOperationLogsOutput>, crate::error::ApiError> {
    Ok(Json(
        ListOperationLogsUseCase::new(container.operation_log_repository()).execute(
            ListOperationLogsInput {
                start_date: query.start_date,
                end_date: query.end_date,
                event_type: query.event_type,
                cursor: query.cursor,
                limit: query.limit,
            },
        )?,
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        extract::{Query, State},
        Json,
    };
    use chrono::{TimeZone, Utc};
    use ipo_backend_shared::{
        acl::{browser::BrokerBrowserPort, notification::NotificationPort},
        domain::{
            account::{AccountCredential, ConnectionTestResult},
            operation_log::{
                OperationEventType, OperationLog, OperationLogPayload, OperationLogRepository,
                OperationStatus,
            },
            stock::IpoStock,
        },
        errors::DomainError,
        infrastructure::{
            firestore::repositories::{
                FirestoreExclusionRepository, FirestoreIpoStockRepository,
                FirestoreLotteryApplicationRepository, FirestoreNotificationSettingRepository,
                FirestoreOperationLogRepository, FirestoreSecuritiesAccountRepository,
            },
            notification::SlackNotificationAdapter,
            secrets::InMemoryCredentialStore,
        },
    };
    use reqwest::Client;

    use crate::infrastructure::{DependencyContainer, NotificationPortRegistry};

    use super::{list_logs, ListOperationLogsQuery};

    #[derive(Debug)]
    struct DummyBrowserPort;

    #[async_trait]
    impl BrokerBrowserPort for DummyBrowserPort {
        async fn fetch_ipo_stocks(
            &self,
        ) -> Result<Vec<ipo_backend_shared::acl::scraping::ScrapedStock>, DomainError> {
            Ok(Vec::new())
        }

        async fn test_connection(
            &self,
            _credential: &AccountCredential,
        ) -> Result<ConnectionTestResult, DomainError> {
            Ok(ConnectionTestResult::new(true, "ok", Utc::now()))
        }

        async fn check_lottery_result(
            &self,
            _credential: &AccountCredential,
            _stock: &IpoStock,
        ) -> Result<Option<ipo_backend_shared::domain::application::LotteryResult>, DomainError>
        {
            Ok(None)
        }
    }

    fn container(repository: Arc<dyn OperationLogRepository + Send + Sync>) -> DependencyContainer {
        DependencyContainer::from_components(
            Arc::new(FirestoreIpoStockRepository::new()),
            Arc::new(FirestoreExclusionRepository::new()),
            Arc::new(FirestoreLotteryApplicationRepository::new()),
            Arc::new(FirestoreSecuritiesAccountRepository::new(
                InMemoryCredentialStore::new(),
            )),
            Arc::new(FirestoreNotificationSettingRepository::new()),
            repository,
            Arc::new(DummyBrowserPort),
            Arc::new(NotificationPortRegistry::new(
                Arc::new(SlackNotificationAdapter::new(Client::new()))
                    as Arc<dyn NotificationPort + Send + Sync>,
                Arc::new(SlackNotificationAdapter::new(Client::new()))
                    as Arc<dyn NotificationPort + Send + Sync>,
                Arc::new(SlackNotificationAdapter::new(Client::new()))
                    as Arc<dyn NotificationPort + Send + Sync>,
            )),
        )
        .expect("container")
    }

    #[tokio::test]
    async fn lists_logs_from_query_parameters() {
        let repository = Arc::new(FirestoreOperationLogRepository::new())
            as Arc<dyn OperationLogRepository + Send + Sync>;
        repository
            .save(
                &OperationLog::create(OperationLogPayload::new(
                    None,
                    OperationEventType::ApplyLottery,
                    "ipo-browser",
                    OperationStatus::Succeeded,
                    "ok",
                    None,
                    Utc.with_ymd_and_hms(2026, 4, 13, 9, 0, 0)
                        .single()
                        .expect("executed"),
                ))
                .expect("log"),
            )
            .expect("save log");

        let Json(output) = list_logs(
            State(container(repository)),
            Query(ListOperationLogsQuery {
                start_date: Some("2026-04-13".to_string()),
                end_date: Some("2026-04-13".to_string()),
                event_type: Some("apply_lottery".to_string()),
                cursor: None,
                limit: Some(10),
            }),
        )
        .await
        .expect("list");

        assert_eq!(output.items.len(), 1);
        assert_eq!(output.items[0].event_type, "apply_lottery");
    }
}
