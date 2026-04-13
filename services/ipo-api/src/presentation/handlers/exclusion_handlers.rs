use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    application::use_cases::{
        ListExclusionsUseCase, RegisterExclusionInput, RegisterExclusionUseCase,
        RemoveExclusionUseCase,
    },
    error::ApiError,
    infrastructure::DependencyContainer,
};

pub async fn list_exclusions(
    State(container): State<DependencyContainer>,
) -> Result<Json<crate::application::use_cases::ListExclusionsOutput>, ApiError> {
    Ok(Json(
        ListExclusionsUseCase::new(container.exclusion_repository()).execute()?,
    ))
}

pub async fn register_exclusion(
    State(container): State<DependencyContainer>,
    Json(input): Json<RegisterExclusionInput>,
) -> Result<
    (
        StatusCode,
        Json<crate::application::use_cases::RegisterExclusionOutput>,
    ),
    ApiError,
> {
    let output = RegisterExclusionUseCase::new(container.exclusion_repository()).execute(input)?;
    Ok((StatusCode::CREATED, Json(output)))
}

pub async fn remove_exclusion(
    State(container): State<DependencyContainer>,
    Path(exclusion_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    RemoveExclusionUseCase::new(container.exclusion_repository()).execute(&exclusion_id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        Json,
    };
    use ipo_backend_shared::{
        acl::{browser::BrokerBrowserPort, notification::NotificationPort},
        domain::{account::AccountCredential, stock::IpoStock},
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

    use super::{list_exclusions, register_exclusion, remove_exclusion, RegisterExclusionInput};

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
        ) -> Result<ipo_backend_shared::domain::account::ConnectionTestResult, DomainError>
        {
            Ok(
                ipo_backend_shared::domain::account::ConnectionTestResult::new(
                    true,
                    "ok",
                    chrono::Utc::now(),
                ),
            )
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

    fn container() -> DependencyContainer {
        DependencyContainer::from_components(
            Arc::new(FirestoreIpoStockRepository::new()),
            Arc::new(FirestoreExclusionRepository::new()),
            Arc::new(FirestoreLotteryApplicationRepository::new()),
            Arc::new(FirestoreSecuritiesAccountRepository::new(
                InMemoryCredentialStore::new(),
            )),
            Arc::new(FirestoreNotificationSettingRepository::new()),
            Arc::new(FirestoreOperationLogRepository::new()),
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
    async fn registers_lists_and_removes_exclusions() {
        let container = container();
        let (status, Json(registered)) = register_exclusion(
            State(container.clone()),
            Json(RegisterExclusionInput {
                company_name: "テスト株式会社".to_string(),
                reason: "manual".to_string(),
            }),
        )
        .await
        .expect("register");
        assert_eq!(status, StatusCode::CREATED);

        let Json(listed) = list_exclusions(State(container.clone()))
            .await
            .expect("list");
        assert_eq!(listed.total_count, 1);

        let status = remove_exclusion(State(container), Path(registered.identifier))
            .await
            .expect("remove");
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
}
