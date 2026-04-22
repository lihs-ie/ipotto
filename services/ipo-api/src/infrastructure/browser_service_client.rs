use async_trait::async_trait;
use chrono::Utc;
use ipo_backend_shared::{
    acl::{
        browser::{ApplicationResult, BrokerBrowserPort},
        scraping::ScrapedStock,
    },
    domain::{
        account::{AccountCredential, ConnectionTestResult},
        application::{AppliedOrder, LotteryResult},
        stock::IpoStock,
    },
    errors::DomainError,
    infrastructure::http_client::{get_json_with_retry, post_json_with_retry, RetryPolicy},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct BrowserServiceClient {
    client: Client,
    base_url: String,
    retry_policy: RetryPolicy,
}

impl BrowserServiceClient {
    pub fn new(client: Client, base_url: impl Into<String>) -> Self {
        Self::with_retry_policy(client, base_url, RetryPolicy::default())
    }

    pub fn with_retry_policy(
        client: Client,
        base_url: impl Into<String>,
        retry_policy: RetryPolicy,
    ) -> Self {
        Self {
            client,
            base_url: base_url.into(),
            retry_policy,
        }
    }
}

#[async_trait]
impl BrokerBrowserPort for BrowserServiceClient {
    async fn fetch_ipo_stocks(&self) -> Result<Vec<ScrapedStock>, DomainError> {
        let url = format!("{}/internal/stocks", self.base_url);
        get_json_with_retry::<Vec<ScrapedStock>>(&self.retry_policy, &self.client, &url).await
    }

    async fn test_connection(
        &self,
        credential: &AccountCredential,
    ) -> Result<ConnectionTestResult, DomainError> {
        let url = format!("{}/internal/accounts/test", self.base_url);
        let request = BrowserCredentialRequest::from_credential(credential);
        let response: BrowserConnectionTestResponse =
            post_json_with_retry(&self.retry_policy, &self.client, &url, &request).await?;
        Ok(ConnectionTestResult::new(
            response.success,
            response.message,
            response.tested_at.unwrap_or_else(Utc::now),
        ))
    }

    async fn check_lottery_result(
        &self,
        credential: &AccountCredential,
        stock: &IpoStock,
    ) -> Result<Option<LotteryResult>, DomainError> {
        let url = format!("{}/internal/lottery-results/check", self.base_url);
        let request = BrowserLotteryResultRequest::new(credential, stock);
        let response: BrowserLotteryResultResponse =
            post_json_with_retry(&self.retry_policy, &self.client, &url, &request).await?;
        response.result.map(parse_lottery_result).transpose()
    }

    async fn apply_for_ipo(
        &self,
        credential: &AccountCredential,
        stock: &IpoStock,
        applied_order: &AppliedOrder,
    ) -> Result<ApplicationResult, DomainError> {
        let url = format!("{}/internal/lottery-applications/submit", self.base_url);
        let request = BrowserApplyRequest::new(credential, stock, applied_order);
        post_json_with_retry::<_, ApplicationResult>(
            &self.retry_policy,
            &self.client,
            &url,
            &request,
        )
        .await
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserCredentialRequest {
    login_id: String,
    login_password: String,
    trading_password: String,
    mail_address: String,
    mail_password: String,
    imap_host: String,
    imap_port: u16,
}

impl BrowserCredentialRequest {
    fn from_credential(credential: &AccountCredential) -> Self {
        Self {
            login_id: credential.login_id().value().to_string(),
            login_password: credential.login_password().value().to_string(),
            trading_password: credential.trading_password().value().to_string(),
            mail_address: credential
                .mail_credential()
                .mail_address()
                .value()
                .to_string(),
            mail_password: credential
                .mail_credential()
                .mail_password()
                .value()
                .to_string(),
            imap_host: credential.mail_credential().imap_host().value().to_string(),
            imap_port: credential.mail_credential().imap_port().value(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserLotteryResultRequest {
    credential: BrowserCredentialRequest,
    stock_identifier: String,
}

impl BrowserLotteryResultRequest {
    fn new(credential: &AccountCredential, stock: &IpoStock) -> Self {
        Self {
            credential: BrowserCredentialRequest::from_credential(credential),
            stock_identifier: stock.identifier().value().to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserApplyRequest {
    credential: BrowserCredentialRequest,
    stock_identifier: String,
    company_name: String,
    shares: u32,
    price: i64,
}

impl BrowserApplyRequest {
    fn new(credential: &AccountCredential, stock: &IpoStock, applied_order: &AppliedOrder) -> Self {
        Self {
            credential: BrowserCredentialRequest::from_credential(credential),
            stock_identifier: stock.identifier().value().to_string(),
            company_name: stock.company_profile().company_name().value().to_string(),
            shares: applied_order.shares().value(),
            price: applied_order.price().value(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowserConnectionTestResponse {
    success: bool,
    message: String,
    tested_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowserLotteryResultResponse {
    result: Option<String>,
}

fn parse_lottery_result(value: String) -> Result<LotteryResult, DomainError> {
    match value.as_str() {
        "Won" => Ok(LotteryResult::Won),
        "Lost" => Ok(LotteryResult::Lost),
        "Alternate" => Ok(LotteryResult::Alternate),
        other => Err(DomainError::HttpClientError {
            reason: format!("unsupported lottery result: {other}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, Utc};
    use ipo_backend_shared::{
        acl::browser::{ApplicationResult, BrokerBrowserPort},
        domain::{
            account::{
                AccountCredential, ImapHost, ImapPort, LoginId, LoginPassword, MailAddress,
                MailCredential, MailPassword, TradingPassword,
            },
            application::AppliedOrder,
            stock::{
                BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry,
                IpoOffering, IpoPricing, IpoSchedule, IpoStock, LeadUnderwriter, Market,
                MetaSource, PriceRange, Shares, StockStatus, Yen,
            },
        },
    };
    use serde_json::json;
    use wiremock::{
        matchers::{body_partial_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::BrowserServiceClient;

    fn build_credential() -> AccountCredential {
        AccountCredential::new(
            LoginId::new("login").expect("login id"),
            LoginPassword::new("password").expect("login password"),
            TradingPassword::new("1234").expect("trading password"),
            MailCredential::new(
                MailAddress::new("test@example.com").expect("mail"),
                MailPassword::new("mail-password").expect("mail password"),
                ImapHost::new("imap.example.com").expect("host"),
                ImapPort::new(993).expect("port"),
            )
            .expect("mail credential"),
        )
        .expect("credential")
    }

    #[tokio::test]
    async fn tests_connection_via_browser_service() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/internal/accounts/test"))
            .and(body_partial_json(json!({
                "loginId": "login",
                "loginPassword": "password",
                "tradingPassword": "1234",
                "mailAddress": "test@example.com",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "message": "connected"
            })))
            .mount(&server)
            .await;

        let client = BrowserServiceClient::new(reqwest::Client::new(), server.uri());
        let result = ipo_backend_shared::acl::browser::BrokerBrowserPort::test_connection(
            &client,
            &build_credential(),
        )
        .await
        .expect("test connection");

        assert!(result.success());
        assert_eq!(result.message(), "connected");
    }

    fn build_stock() -> IpoStock {
        IpoStock::create(
            CompanyProfile::new(
                CompanyName::new("テスト第一株式会社").expect("name"),
                None,
                Market::Growth,
                Industry::new("情報・通信業").expect("industry"),
            )
            .expect("profile"),
            IpoSchedule::new(
                BookBuildingPeriod::new(
                    NaiveDate::from_ymd_opt(2026, 4, 20).expect("start"),
                    NaiveDate::from_ymd_opt(2026, 4, 24).expect("end"),
                )
                .expect("period"),
                NaiveDate::from_ymd_opt(2026, 4, 27).expect("lottery"),
                NaiveDate::from_ymd_opt(2026, 4, 30).expect("listing"),
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
            StockStatus::Eligible,
            MetaSource::new(FetchOrigin::ExternalSite, Utc::now()),
        )
        .expect("stock")
    }

    fn build_applied_order() -> AppliedOrder {
        AppliedOrder::new(
            Shares::new(100).expect("shares"),
            Yen::new(1400).expect("price"),
            Utc::now(),
        )
        .expect("applied order")
    }

    #[tokio::test]
    async fn applies_for_ipo_translates_success_response() {
        let server = MockServer::start().await;
        let stock = build_stock();

        Mock::given(method("POST"))
            .and(path("/internal/lottery-applications/submit"))
            .and(body_partial_json(json!({
                "stockIdentifier": stock.identifier().value(),
                "companyName": "テスト第一株式会社",
                "shares": 100,
                "price": 1400,
                "credential": {
                    "loginId": "login",
                    "tradingPassword": "1234"
                }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "success"
            })))
            .mount(&server)
            .await;

        let client = BrowserServiceClient::new(reqwest::Client::new(), server.uri());
        let result = client
            .apply_for_ipo(&build_credential(), &stock, &build_applied_order())
            .await
            .expect("apply should succeed");

        assert_eq!(result, ApplicationResult::Success);
    }

    #[tokio::test]
    async fn applies_for_ipo_decodes_failure_reason() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/internal/lottery-applications/submit"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "failure",
                "reason": "broker timeout"
            })))
            .mount(&server)
            .await;

        let client = BrowserServiceClient::new(reqwest::Client::new(), server.uri());
        let result = client
            .apply_for_ipo(&build_credential(), &build_stock(), &build_applied_order())
            .await
            .expect("apply call should resolve");

        assert_eq!(
            result,
            ApplicationResult::Failure {
                reason: "broker timeout".to_string()
            }
        );
    }

    #[tokio::test]
    async fn applies_for_ipo_decodes_already_applied_variant() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/internal/lottery-applications/submit"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "already_applied"
            })))
            .mount(&server)
            .await;

        let client = BrowserServiceClient::new(reqwest::Client::new(), server.uri());
        let result = client
            .apply_for_ipo(&build_credential(), &build_stock(), &build_applied_order())
            .await
            .expect("apply call should resolve");

        assert_eq!(result, ApplicationResult::AlreadyApplied);
    }

    #[tokio::test]
    async fn applies_for_ipo_decodes_insufficient_balance_variant() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/internal/lottery-applications/submit"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "insufficient_balance"
            })))
            .mount(&server)
            .await;

        let client = BrowserServiceClient::new(reqwest::Client::new(), server.uri());
        let result = client
            .apply_for_ipo(&build_credential(), &build_stock(), &build_applied_order())
            .await
            .expect("apply call should resolve");

        assert_eq!(result, ApplicationResult::InsufficientBalance);
    }
}
