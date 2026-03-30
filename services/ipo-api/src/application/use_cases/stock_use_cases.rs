use std::{collections::BTreeMap, sync::Arc};

use chrono::{Duration, Utc};
use ipo_backend_shared::{
    domain::{
        account::SecuritiesAccountRepository,
        application::{LotteryApplicationRepository, LotteryResult},
        stock::{IpoStock, IpoStockRepository, StockIdentifier, StockStatus},
    },
    errors::DomainError,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIpoStocksInput {
    pub status_filter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpoStockSummaryOutput {
    pub identifier: String,
    pub company_name: String,
    pub ticker_symbol: Option<String>,
    pub market: String,
    pub industry: String,
    pub book_building_start_date: String,
    pub book_building_end_date: String,
    pub lottery_date: String,
    pub listing_date: String,
    pub price_range_min: i64,
    pub price_range_max: i64,
    pub offer_price: Option<i64>,
    pub lead_underwriter: String,
    pub number_of_offered_shares: u32,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIpoStocksOutput {
    pub items: Vec<IpoStockSummaryOutput>,
    pub total_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetIpoStockInput {
    pub stock_identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfileOutput {
    pub company_name: String,
    pub ticker_symbol: Option<String>,
    pub market: String,
    pub industry: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleOutput {
    pub book_building_start_date: String,
    pub book_building_end_date: String,
    pub lottery_date: String,
    pub listing_date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingOutput {
    pub price_range_min: i64,
    pub price_range_max: i64,
    pub offer_price: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferingOutput {
    pub lead_underwriter: String,
    pub number_of_offered_shares: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaSourceOutput {
    pub source: String,
    pub fetched_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockApplicationOutput {
    pub identifier: String,
    pub securities_company: String,
    pub applied_shares: u32,
    pub applied_price: i64,
    pub applied_at: String,
    pub lottery_outcome: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIpoStockOutput {
    pub identifier: String,
    pub company_profile: CompanyProfileOutput,
    pub schedule: ScheduleOutput,
    pub pricing: PricingOutput,
    pub offering: OfferingOutput,
    pub status: String,
    pub meta_source: MetaSourceOutput,
    pub applications: Vec<StockApplicationOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardRecentActivityOutput {
    pub stock: String,
    pub company_name: String,
    pub securities_company: String,
    pub event_type: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardUpcomingStockOutput {
    pub stock: String,
    pub company_name: String,
    pub book_building_start_date: String,
    pub book_building_end_date: String,
    pub lottery_date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardAccountStatusOutput {
    pub securities_company: String,
    pub connection_status: String,
    pub last_tested_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSystemStatusOutput {
    pub next_job_scheduled_at: String,
    pub accounts: Vec<DashboardAccountStatusOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummaryOutput {
    pub status_counts: BTreeMap<String, u32>,
    pub recent_activities: Vec<DashboardRecentActivityOutput>,
    pub upcoming_stocks: Vec<DashboardUpcomingStockOutput>,
    pub system_status: DashboardSystemStatusOutput,
}

pub struct ListIpoStocksUseCase {
    repository: Arc<dyn IpoStockRepository + Send + Sync>,
}

impl ListIpoStocksUseCase {
    pub fn new(repository: Arc<dyn IpoStockRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub fn execute(&self, input: ListIpoStocksInput) -> Result<ListIpoStocksOutput, DomainError> {
        let stocks = match input.status_filter {
            Some(status_filter) => self
                .repository
                .find_by_status(parse_stock_status(&status_filter)?)?,
            None => self.repository.find_all()?,
        };

        let items = stocks.iter().map(stock_to_summary).collect::<Vec<_>>();
        Ok(ListIpoStocksOutput {
            total_count: items.len(),
            items,
        })
    }
}

pub struct GetIpoStockUseCase {
    stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
    application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
    account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
}

impl GetIpoStockUseCase {
    pub fn new(
        stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
        application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
        account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
    ) -> Self {
        Self {
            stock_repository,
            application_repository,
            account_repository,
        }
    }

    pub fn execute(&self, input: GetIpoStockInput) -> Result<GetIpoStockOutput, DomainError> {
        let stock_identifier = StockIdentifier::new(input.stock_identifier)?;
        let stock = self
            .stock_repository
            .find_by_id(&stock_identifier)?
            .ok_or_else(|| DomainError::FirestoreMappingError {
                reason: format!("stock not found: {}", stock_identifier.value()),
            })?;

        let applications = self
            .application_repository
            .find_by_stock(stock.identifier())?
            .into_iter()
            .map(|application| {
                let securities_company = self
                    .account_repository
                    .find_by_id(application.securities_account())?
                    .map(|account| account.securities_company().as_str().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                Ok(StockApplicationOutput {
                    identifier: application.identifier().value().to_string(),
                    securities_company,
                    applied_shares: application.applied_order().shares().value(),
                    applied_price: application.applied_order().price().value(),
                    applied_at: application.applied_order().ordered_at().to_rfc3339(),
                    lottery_outcome: application
                        .lottery_outcome()
                        .map(|outcome| lottery_result_to_string(outcome.result())),
                    status: application.status().as_str().to_string(),
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?;

        Ok(GetIpoStockOutput {
            identifier: stock.identifier().value().to_string(),
            company_profile: CompanyProfileOutput {
                company_name: stock.company_profile().company_name().value().to_string(),
                ticker_symbol: stock
                    .company_profile()
                    .ticker_symbol()
                    .map(|ticker| ticker.value().to_string()),
                market: stock.company_profile().market().as_str().to_string(),
                industry: stock.company_profile().industry().value().to_string(),
            },
            schedule: ScheduleOutput {
                book_building_start_date: stock
                    .schedule()
                    .book_building_period()
                    .start_date()
                    .to_string(),
                book_building_end_date: stock
                    .schedule()
                    .book_building_period()
                    .end_date()
                    .to_string(),
                lottery_date: stock.schedule().lottery_date().to_string(),
                listing_date: stock.schedule().listing_date().to_string(),
            },
            pricing: PricingOutput {
                price_range_min: stock.pricing().price_range().minimum_price().value(),
                price_range_max: stock.pricing().price_range().maximum_price().value(),
                offer_price: stock.pricing().offer_price().map(|value| value.value()),
            },
            offering: OfferingOutput {
                lead_underwriter: stock.offering().lead_underwriter().value().to_string(),
                number_of_offered_shares: stock.offering().number_of_offered_shares().value(),
            },
            status: stock.status().as_str().to_string(),
            meta_source: MetaSourceOutput {
                source: match stock.meta_source().source() {
                    ipo_backend_shared::domain::stock::FetchOrigin::ExternalSite => {
                        "ExternalSite".to_string()
                    }
                    ipo_backend_shared::domain::stock::FetchOrigin::SecuritiesSite => {
                        "SecuritiesSite".to_string()
                    }
                },
                fetched_at: stock.meta_source().fetched_at().to_rfc3339(),
            },
            applications,
        })
    }
}

pub struct GetDashboardSummaryUseCase {
    stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
    application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
    account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
}

impl GetDashboardSummaryUseCase {
    pub fn new(
        stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
        application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
        account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
    ) -> Self {
        Self {
            stock_repository,
            application_repository,
            account_repository,
        }
    }

    pub fn execute(&self) -> Result<DashboardSummaryOutput, DomainError> {
        let stocks = self.stock_repository.find_all()?;
        let mut status_counts = BTreeMap::new();
        for stock in &stocks {
            *status_counts
                .entry(stock.status().as_str().to_string())
                .or_insert(0) += 1;
        }

        let mut recent_activities = Vec::new();
        for stock in &stocks {
            for application in self
                .application_repository
                .find_by_stock(stock.identifier())?
            {
                let occurred_at = application
                    .lottery_outcome()
                    .map(|outcome| outcome.confirmed_at())
                    .unwrap_or_else(|| application.applied_order().ordered_at());
                let event_type = application
                    .lottery_outcome()
                    .map(|outcome| match outcome.result() {
                        LotteryResult::Won => "LotteryResultWon".to_string(),
                        LotteryResult::Lost => "LotteryResultLost".to_string(),
                        LotteryResult::Alternate => "LotteryResultAlternate".to_string(),
                    })
                    .unwrap_or_else(|| "ApplicationCompleted".to_string());
                let securities_company = self
                    .account_repository
                    .find_by_id(application.securities_account())?
                    .map(|account| account.securities_company().as_str().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                recent_activities.push((
                    occurred_at,
                    DashboardRecentActivityOutput {
                        stock: stock.identifier().value().to_string(),
                        company_name: stock.company_profile().company_name().value().to_string(),
                        securities_company,
                        event_type,
                        occurred_at: occurred_at.to_rfc3339(),
                    },
                ));
            }
        }
        recent_activities.sort_by_key(|(occurred_at, _)| std::cmp::Reverse(*occurred_at));

        let today = Utc::now().date_naive();
        let mut upcoming_stocks = stocks
            .iter()
            .filter(|stock| stock.schedule().book_building_period().start_date() >= today)
            .map(|stock| DashboardUpcomingStockOutput {
                stock: stock.identifier().value().to_string(),
                company_name: stock.company_profile().company_name().value().to_string(),
                book_building_start_date: stock
                    .schedule()
                    .book_building_period()
                    .start_date()
                    .to_string(),
                book_building_end_date: stock
                    .schedule()
                    .book_building_period()
                    .end_date()
                    .to_string(),
                lottery_date: stock.schedule().lottery_date().to_string(),
            })
            .collect::<Vec<_>>();
        upcoming_stocks.sort_by_key(|stock| stock.book_building_start_date.clone());
        upcoming_stocks.truncate(5);

        let accounts = self
            .account_repository
            .find_all()?
            .into_iter()
            .map(|account| DashboardAccountStatusOutput {
                securities_company: account.securities_company().as_str().to_string(),
                connection_status: account
                    .connection_test()
                    .map(|result| {
                        if result.success() {
                            "healthy"
                        } else {
                            "failed"
                        }
                    })
                    .unwrap_or("unknown")
                    .to_string(),
                last_tested_at: account
                    .connection_test()
                    .map(|result| result.tested_at().to_rfc3339()),
            })
            .collect::<Vec<_>>();

        Ok(DashboardSummaryOutput {
            status_counts,
            recent_activities: recent_activities
                .into_iter()
                .map(|(_, activity)| activity)
                .take(5)
                .collect(),
            upcoming_stocks,
            system_status: DashboardSystemStatusOutput {
                next_job_scheduled_at: (Utc::now() + Duration::hours(24)).to_rfc3339(),
                accounts,
            },
        })
    }
}

fn stock_to_summary(stock: &IpoStock) -> IpoStockSummaryOutput {
    IpoStockSummaryOutput {
        identifier: stock.identifier().value().to_string(),
        company_name: stock.company_profile().company_name().value().to_string(),
        ticker_symbol: stock
            .company_profile()
            .ticker_symbol()
            .map(|ticker| ticker.value().to_string()),
        market: stock.company_profile().market().as_str().to_string(),
        industry: stock.company_profile().industry().value().to_string(),
        book_building_start_date: stock
            .schedule()
            .book_building_period()
            .start_date()
            .to_string(),
        book_building_end_date: stock
            .schedule()
            .book_building_period()
            .end_date()
            .to_string(),
        lottery_date: stock.schedule().lottery_date().to_string(),
        listing_date: stock.schedule().listing_date().to_string(),
        price_range_min: stock.pricing().price_range().minimum_price().value(),
        price_range_max: stock.pricing().price_range().maximum_price().value(),
        offer_price: stock.pricing().offer_price().map(|value| value.value()),
        lead_underwriter: stock.offering().lead_underwriter().value().to_string(),
        number_of_offered_shares: stock.offering().number_of_offered_shares().value(),
        status: stock.status().as_str().to_string(),
    }
}

fn parse_stock_status(value: &str) -> Result<StockStatus, DomainError> {
    match value {
        "Fetched" => Ok(StockStatus::Fetched),
        "Eligible" => Ok(StockStatus::Eligible),
        "Applied" => Ok(StockStatus::Applied),
        "Won" => Ok(StockStatus::Won),
        "Lost" => Ok(StockStatus::Lost),
        "Alternate" => Ok(StockStatus::Alternate),
        "Purchased" => Ok(StockStatus::Purchased),
        "Declined" => Ok(StockStatus::Declined),
        "Sold" => Ok(StockStatus::Sold),
        "Excluded" => Ok(StockStatus::Excluded),
        "Failed" => Ok(StockStatus::Failed),
        other => Err(DomainError::InvalidStatusTransition {
            from: other.to_string(),
            to: "StockStatus".to_string(),
        }),
    }
}

fn lottery_result_to_string(value: LotteryResult) -> String {
    match value {
        LotteryResult::Won => "Won".to_string(),
        LotteryResult::Lost => "Lost".to_string(),
        LotteryResult::Alternate => "Alternate".to_string(),
    }
}
