use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    application::use_cases::{
        DeleteSecuritiesAccountUseCase, ListSecuritiesAccountsOutput,
        ListSecuritiesAccountsUseCase, RegisterSecuritiesAccountInput,
        RegisterSecuritiesAccountOutput, RegisterSecuritiesAccountUseCase,
        TestSecuritiesAccountConnectionUseCase, UpdateSecuritiesAccountInput,
        UpdateSecuritiesAccountOutput, UpdateSecuritiesAccountUseCase,
    },
    error::ApiError,
    infrastructure::DependencyContainer,
};

pub async fn list_accounts(
    State(container): State<DependencyContainer>,
) -> Result<Json<ListSecuritiesAccountsOutput>, ApiError> {
    Ok(Json(
        ListSecuritiesAccountsUseCase::new(container.account_repository()).execute()?,
    ))
}

pub async fn register_account(
    State(container): State<DependencyContainer>,
    Json(input): Json<RegisterSecuritiesAccountInput>,
) -> Result<(StatusCode, Json<RegisterSecuritiesAccountOutput>), ApiError> {
    let output =
        RegisterSecuritiesAccountUseCase::new(container.account_repository()).execute(input)?;
    Ok((StatusCode::CREATED, Json(output)))
}

pub async fn update_account(
    State(container): State<DependencyContainer>,
    Path(account_id): Path<String>,
    Json(mut input): Json<UpdateSecuritiesAccountInput>,
) -> Result<Json<UpdateSecuritiesAccountOutput>, ApiError> {
    input.account_identifier = account_id;
    let output =
        UpdateSecuritiesAccountUseCase::new(container.account_repository()).execute(input)?;
    Ok(Json(output))
}

pub async fn delete_account(
    State(container): State<DependencyContainer>,
    Path(account_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    DeleteSecuritiesAccountUseCase::new(container.account_repository()).execute(&account_id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_connection(
    State(container): State<DependencyContainer>,
    Path(account_id): Path<String>,
) -> Result<Json<crate::application::use_cases::ConnectionTestOutput>, ApiError> {
    let output = TestSecuritiesAccountConnectionUseCase::new(
        container.account_repository(),
        container.browser_port(),
    )
    .execute(&account_id)
    .await?;
    Ok(Json(output))
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
    use chrono::Utc;
    use ipo_backend_shared::{
        acl::{browser::BrokerBrowserPort, notification::NotificationPort},
        domain::{
            account::{AccountCredential, ConnectionTestResult},
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

    use super::{
        delete_account, list_accounts, register_account, test_connection,
        RegisterSecuritiesAccountInput,
    };

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

    fn input() -> RegisterSecuritiesAccountInput {
        RegisterSecuritiesAccountInput {
            securities_company: "Rakuten".to_string(),
            login_id: "login".to_string(),
            login_password: "password".to_string(),
            trading_password: "1234".to_string(),
            mail_address: "test@example.com".to_string(),
            mail_password: "mail-password".to_string(),
            imap_host: "imap.example.com".to_string(),
            imap_port: 993,
        }
    }

    #[tokio::test]
    async fn registers_lists_tests_and_deletes_accounts() {
        let container = container();
        let (status, Json(registered)) = register_account(State(container.clone()), Json(input()))
            .await
            .expect("register");
        assert_eq!(status, StatusCode::CREATED);

        let Json(listed) = list_accounts(State(container.clone())).await.expect("list");
        assert_eq!(listed.total_count, 1);

        let Json(connection) = test_connection(
            State(container.clone()),
            Path(registered.identifier.clone()),
        )
        .await
        .expect("test connection");
        assert!(connection.success);

        let status = delete_account(State(container), Path(registered.identifier))
            .await
            .expect("delete");
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
}
