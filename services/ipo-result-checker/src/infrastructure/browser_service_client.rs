use async_trait::async_trait;
use ipo_backend_shared::{
    acl::{browser::BrokerBrowserPort, scraping::ScrapedStock},
    domain::{
        account::{AccountCredential, ConnectionTestResult},
        application::LotteryResult,
        stock::IpoStock,
    },
    errors::DomainError,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct BrowserServiceClient {
    client: Client,
    base_url: String,
}

impl BrowserServiceClient {
    pub fn new(client: Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl BrokerBrowserPort for BrowserServiceClient {
    async fn fetch_ipo_stocks(&self) -> Result<Vec<ScrapedStock>, DomainError> {
        Err(DomainError::HttpClientError {
            reason: "fetch_ipo_stocks is not used by ipo-result-checker".to_string(),
        })
    }

    async fn test_connection(
        &self,
        _credential: &AccountCredential,
    ) -> Result<ConnectionTestResult, DomainError> {
        Err(DomainError::HttpClientError {
            reason: "test_connection is not used by ipo-result-checker".to_string(),
        })
    }

    async fn check_lottery_result(
        &self,
        credential: &AccountCredential,
        stock: &IpoStock,
    ) -> Result<Option<LotteryResult>, DomainError> {
        let response = self
            .client
            .post(format!("{}/internal/lottery-results/check", self.base_url))
            .json(&BrowserLotteryResultRequest::new(credential, stock))
            .send()
            .await
            .map_err(|error| DomainError::HttpClientError {
                reason: error.to_string(),
            })?
            .error_for_status()
            .map_err(|error| DomainError::HttpClientError {
                reason: error.to_string(),
            })?
            .json::<BrowserLotteryResultResponse>()
            .await
            .map_err(|error| DomainError::HttpClientError {
                reason: error.to_string(),
            })?;
        response.result.map(parse_lottery_result).transpose()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserCredentialRequest {
    login_id: String,
    login_password: String,
    trading_password: String,
}

impl BrowserCredentialRequest {
    fn from_credential(credential: &AccountCredential) -> Self {
        Self {
            login_id: credential.login_id().value().to_string(),
            login_password: credential.login_password().value().to_string(),
            trading_password: credential.trading_password().value().to_string(),
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
    use ipo_backend_shared::domain::{
        account::{
            AccountCredential, ImapHost, ImapPort, LoginId, LoginPassword, MailAddress,
            MailCredential, MailPassword, TradingPassword,
        },
        application::LotteryResult,
        stock::{
            BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry, IpoOffering,
            IpoPricing, IpoSchedule, IpoStock, LeadUnderwriter, Market, MetaSource, PriceRange,
            Shares, StockStatus, Yen,
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

    fn build_stock() -> IpoStock {
        IpoStock::create(
            CompanyProfile::new(
                CompanyName::new("テスト株式会社").expect("name"),
                None,
                Market::Growth,
                Industry::new("情報・通信業").expect("industry"),
            )
            .expect("profile"),
            IpoSchedule::new(
                BookBuildingPeriod::new(
                    NaiveDate::from_ymd_opt(2026, 4, 1).expect("start"),
                    NaiveDate::from_ymd_opt(2026, 4, 10).expect("end"),
                )
                .expect("period"),
                NaiveDate::from_ymd_opt(2026, 4, 15).expect("lottery"),
                NaiveDate::from_ymd_opt(2026, 4, 25).expect("listing"),
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
                Shares::new(100000).expect("shares"),
            )
            .expect("offering"),
            StockStatus::Applied,
            MetaSource::new(FetchOrigin::ExternalSite, Utc::now()),
        )
        .expect("stock")
    }

    #[tokio::test]
    async fn checks_lottery_result_via_browser_service() {
        let server = MockServer::start().await;
        let credential = build_credential();
        let stock = build_stock();

        Mock::given(method("POST"))
            .and(path("/internal/lottery-results/check"))
            .and(body_partial_json(json!({
                "stockIdentifier": stock.identifier().value(),
                "credential": {
                    "loginId": "login",
                    "loginPassword": "password",
                    "tradingPassword": "1234"
                }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": "Won"
            })))
            .mount(&server)
            .await;

        let client = BrowserServiceClient::new(reqwest::Client::new(), server.uri());
        let result = ipo_backend_shared::acl::browser::BrokerBrowserPort::check_lottery_result(
            &client,
            &credential,
            &stock,
        )
        .await
        .expect("check result");

        assert_eq!(result, Some(LotteryResult::Won));
    }
}
