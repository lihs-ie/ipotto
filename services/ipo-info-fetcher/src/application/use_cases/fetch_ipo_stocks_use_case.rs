use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use ipo_backend_shared::{
    acl::{
        messaging::EventPublisherPort,
        scraping::{IpoStockScraperPort, ScrapedStock},
    },
    domain::{
        operation_log::{
            OperationEventType, OperationLog, OperationLogPayload, OperationLogRepository,
            OperationStatus,
        },
        stock::{
            BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry, IpoOffering,
            IpoPricing, IpoSchedule, IpoStock, IpoStockRepository, IpoStockUpdate, LeadUnderwriter,
            Market, MetaSource, PriceRange, Shares, StockStatus, TickerSymbol, Yen,
        },
    },
    errors::DomainError,
    events::IpoInfoUpdated,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchIpoStocksOutput {
    pub fetched_count: u32,
    pub updated_count: u32,
    pub errors: Vec<String>,
}

pub struct FetchIpoStocksUseCase {
    stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
    scraper: Arc<dyn IpoStockScraperPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
    operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
}

impl FetchIpoStocksUseCase {
    pub fn new(
        stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
        scraper: Arc<dyn IpoStockScraperPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
        operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
    ) -> Self {
        Self {
            stock_repository,
            scraper,
            event_publisher,
            operation_log_repository,
        }
    }

    pub async fn execute(&self) -> Result<FetchIpoStocksOutput, DomainError> {
        let scraped_stocks = self.scraper.scrape().await?;
        let mut existing_stocks = self
            .stock_repository
            .find_all()?
            .into_iter()
            .map(|stock| {
                (
                    stock.company_profile().company_name().value().to_string(),
                    stock,
                )
            })
            .collect::<HashMap<_, _>>();
        let mut fetched_count = 0_u32;
        let mut updated_count = 0_u32;
        let mut errors = Vec::new();

        for scraped_stock in scraped_stocks {
            if let Err(error) = self
                .process_scraped_stock(
                    &scraped_stock,
                    &mut existing_stocks,
                    &mut fetched_count,
                    &mut updated_count,
                )
                .await
            {
                errors.push(error.to_string());
                self.operation_log_repository.save(&OperationLog::create(
                    OperationLogPayload::new(
                        None,
                        OperationEventType::FetchStocks,
                        "ipo-info-fetcher",
                        OperationStatus::Failed,
                        format!("failed to process {}", scraped_stock.company_name()),
                        Some(error.to_string()),
                        Utc::now(),
                    ),
                )?)?;
            }
        }

        Ok(FetchIpoStocksOutput {
            fetched_count,
            updated_count,
            errors,
        })
    }

    async fn process_scraped_stock(
        &self,
        scraped_stock: &ScrapedStock,
        existing_stocks: &mut HashMap<String, IpoStock>,
        fetched_count: &mut u32,
        updated_count: &mut u32,
    ) -> Result<(), DomainError> {
        let existing = existing_stocks
            .get(scraped_stock.company_name())
            .cloned();
        let update = build_stock_update(scraped_stock)?;

        if let Some(mut stock) = existing {
            stock.update_from_source(update)?;
            self.stock_repository.save(&stock)?;
            existing_stocks.insert(
                stock.company_profile().company_name().value().to_string(),
                stock.clone(),
            );
            *updated_count += 1;
            self.operation_log_repository.save(&OperationLog::create(
                OperationLogPayload::new(
                    None,
                    OperationEventType::FetchStocks,
                    "ipo-info-fetcher",
                    OperationStatus::Succeeded,
                    format!("updated {}", stock.company_profile().company_name().value()),
                    None,
                    Utc::now(),
                ),
            )?)?;
            return Ok(());
        }

        let stock = build_new_stock(scraped_stock)?;
        self.stock_repository.save(&stock)?;
        existing_stocks.insert(
            stock.company_profile().company_name().value().to_string(),
            stock.clone(),
        );
        self.event_publisher
            .publish(
                "ipo-info-updated",
                stock.identifier().value(),
                "IpoStock",
                serde_json::to_value(IpoInfoUpdated {
                    identifier: stock.identifier().clone(),
                    company_name: stock.company_profile().company_name().clone(),
                    book_building_period: stock.schedule().book_building_period().clone(),
                    lottery_date: stock.schedule().lottery_date(),
                    listing_date: stock.schedule().listing_date(),
                    updated_at: Utc::now(),
                })
                .map_err(|error| DomainError::PubSubPublishError {
                    reason: error.to_string(),
                })?,
                None,
            )
            .await?;
        self.operation_log_repository
            .save(&OperationLog::create(OperationLogPayload::new(
                None,
                OperationEventType::FetchStocks,
                "ipo-info-fetcher",
                OperationStatus::Succeeded,
                format!("saved {}", stock.company_profile().company_name().value()),
                None,
                Utc::now(),
            ))?)?;
        *fetched_count += 1;
        Ok(())
    }
}

fn build_new_stock(scraped_stock: &ScrapedStock) -> Result<IpoStock, DomainError> {
    let company_profile = CompanyProfile::new(
        CompanyName::new(scraped_stock.company_name())?,
        scraped_stock
            .ticker_symbol()
            .map(TickerSymbol::new)
            .transpose()?,
        Market::new(scraped_stock.market())?,
        Industry::new(scraped_stock.industry())?,
    )?;
    let schedule = IpoSchedule::new(
        BookBuildingPeriod::new(
            scraped_stock.book_building_start_date(),
            scraped_stock.book_building_end_date(),
        )?,
        scraped_stock.lottery_date(),
        scraped_stock.listing_date(),
    )?;
    let pricing = IpoPricing::new(
        PriceRange::new(
            Yen::new(scraped_stock.price_range_min())?,
            Yen::new(scraped_stock.price_range_max())?,
        )?,
        scraped_stock.offer_price().map(Yen::new).transpose()?,
    )?;
    let offering = IpoOffering::new(
        LeadUnderwriter::new(scraped_stock.lead_underwriter())?,
        Shares::new(scraped_stock.number_of_offered_shares())?,
    )?;
    IpoStock::create(
        company_profile,
        schedule,
        pricing,
        offering,
        StockStatus::Fetched,
        MetaSource::new(FetchOrigin::ExternalSite, Utc::now()),
    )
}

fn build_stock_update(scraped_stock: &ScrapedStock) -> Result<IpoStockUpdate, DomainError> {
    let company_profile = CompanyProfile::new(
        CompanyName::new(scraped_stock.company_name())?,
        scraped_stock
            .ticker_symbol()
            .map(TickerSymbol::new)
            .transpose()?,
        Market::new(scraped_stock.market())?,
        Industry::new(scraped_stock.industry())?,
    )?;
    let schedule = IpoSchedule::new(
        BookBuildingPeriod::new(
            scraped_stock.book_building_start_date(),
            scraped_stock.book_building_end_date(),
        )?,
        scraped_stock.lottery_date(),
        scraped_stock.listing_date(),
    )?;
    let pricing = IpoPricing::new(
        PriceRange::new(
            Yen::new(scraped_stock.price_range_min())?,
            Yen::new(scraped_stock.price_range_max())?,
        )?,
        scraped_stock.offer_price().map(Yen::new).transpose()?,
    )?;
    let offering = IpoOffering::new(
        LeadUnderwriter::new(scraped_stock.lead_underwriter())?,
        Shares::new(scraped_stock.number_of_offered_shares())?,
    )?;

    Ok(IpoStockUpdate::new(
        company_profile,
        schedule,
        pricing,
        offering,
        StockStatus::Fetched,
        MetaSource::new(FetchOrigin::ExternalSite, Utc::now()),
    ))
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
    };

    use async_trait::async_trait;
    use chrono::NaiveDate;
    use ipo_backend_shared::{
        acl::scraping::{IpoStockScraperPort, ScrapedStock},
        domain::stock::IpoStockRepository,
        infrastructure::messaging::PubSubEventEnvelope,
        testing::{
            FirestoreIpoStockRepository, FirestoreOperationLogRepository, PubSubEventPublisher,
        },
    };

    use super::FetchIpoStocksUseCase;

    #[derive(Debug)]
    struct StaticScraper {
        stocks: Vec<ScrapedStock>,
    }

    #[async_trait]
    impl IpoStockScraperPort for StaticScraper {
        async fn scrape(
            &self,
        ) -> Result<Vec<ScrapedStock>, ipo_backend_shared::errors::DomainError> {
            Ok(self.stocks.clone())
        }
    }

    #[derive(Debug, Default)]
    struct CountingStockRepository {
        inner: FirestoreIpoStockRepository,
        find_all_calls: AtomicUsize,
    }

    impl CountingStockRepository {
        fn new() -> Self {
            Self::default()
        }
    }

    impl IpoStockRepository for CountingStockRepository {
        fn find_by_id(
            &self,
            identifier: &ipo_backend_shared::domain::stock::StockIdentifier,
        ) -> Result<Option<ipo_backend_shared::domain::stock::IpoStock>, ipo_backend_shared::errors::DomainError>
        {
            self.inner.find_by_id(identifier)
        }

        fn save(
            &self,
            stock: &ipo_backend_shared::domain::stock::IpoStock,
        ) -> Result<(), ipo_backend_shared::errors::DomainError> {
            self.inner.save(stock)
        }

        fn find_all(
            &self,
        ) -> Result<Vec<ipo_backend_shared::domain::stock::IpoStock>, ipo_backend_shared::errors::DomainError>
        {
            self.find_all_calls.fetch_add(1, Ordering::SeqCst);
            self.inner.find_all()
        }

        fn find_by_status(
            &self,
            status: ipo_backend_shared::domain::stock::StockStatus,
        ) -> Result<Vec<ipo_backend_shared::domain::stock::IpoStock>, ipo_backend_shared::errors::DomainError>
        {
            self.inner.find_by_status(status)
        }

        fn find_in_book_building_period(
            &self,
            date: NaiveDate,
        ) -> Result<Vec<ipo_backend_shared::domain::stock::IpoStock>, ipo_backend_shared::errors::DomainError>
        {
            self.inner.find_in_book_building_period(date)
        }
    }

    #[tokio::test]
    async fn saves_new_stock_and_publishes_event() {
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        let event_publisher = Arc::new(PubSubEventPublisher::new("ipo-info-fetcher"));
        let use_case = FetchIpoStocksUseCase::new(
            stock_repository.clone(),
            Arc::new(StaticScraper {
                stocks: vec![ScrapedStock::new(
                    "テスト株式会社",
                    Some("1234".to_string()),
                    "Growth",
                    "情報・通信業",
                    NaiveDate::from_ymd_opt(2026, 4, 1).expect("bb start"),
                    NaiveDate::from_ymd_opt(2026, 4, 10).expect("bb end"),
                    NaiveDate::from_ymd_opt(2026, 4, 15).expect("lottery"),
                    NaiveDate::from_ymd_opt(2026, 4, 25).expect("listing"),
                    1200,
                    1500,
                    Some(1400),
                    "楽天証券",
                    100000,
                )],
            }),
            event_publisher.clone(),
            Arc::new(FirestoreOperationLogRepository::new()),
        );

        let output = use_case.execute().await.expect("execute");

        assert_eq!(output.fetched_count, 1);
        assert_eq!(output.updated_count, 0);
        assert_eq!(stock_repository.find_all().expect("find all").len(), 1);
        assert_eq!(
            event_publisher
                .published_messages()
                .await
                .expect("messages")
                .len(),
            1
        );
        let messages = event_publisher
            .published_messages()
            .await
            .expect("messages");
        let envelope: PubSubEventEnvelope<serde_json::Value> =
            serde_json::from_str(&messages[0]).expect("envelope");
        assert_eq!(envelope.event_type, "ipo-info-updated");
        assert_eq!(envelope.aggregate_type, "IpoStock");
        assert_eq!(envelope.metadata.service_name, "ipo-info-fetcher");
        assert_eq!(envelope.payload["company_name"], "テスト株式会社");
    }

    #[tokio::test]
    async fn loads_existing_stocks_only_once_per_execution() {
        let stock_repository = Arc::new(CountingStockRepository::new());
        let use_case = FetchIpoStocksUseCase::new(
            stock_repository.clone(),
            Arc::new(StaticScraper {
                stocks: vec![
                    ScrapedStock::new(
                        "テスト株式会社",
                        Some("1234".to_string()),
                        "Growth",
                        "情報・通信業",
                        NaiveDate::from_ymd_opt(2026, 4, 1).expect("bb start"),
                        NaiveDate::from_ymd_opt(2026, 4, 10).expect("bb end"),
                        NaiveDate::from_ymd_opt(2026, 4, 15).expect("lottery"),
                        NaiveDate::from_ymd_opt(2026, 4, 25).expect("listing"),
                        1200,
                        1500,
                        Some(1400),
                        "楽天証券",
                        100000,
                    ),
                    ScrapedStock::new(
                        "別の株式会社",
                        Some("5678".to_string()),
                        "Growth",
                        "情報・通信業",
                        NaiveDate::from_ymd_opt(2026, 4, 2).expect("bb start"),
                        NaiveDate::from_ymd_opt(2026, 4, 11).expect("bb end"),
                        NaiveDate::from_ymd_opt(2026, 4, 16).expect("lottery"),
                        NaiveDate::from_ymd_opt(2026, 4, 26).expect("listing"),
                        1300,
                        1600,
                        Some(1500),
                        "楽天証券",
                        200000,
                    ),
                ],
            }),
            Arc::new(PubSubEventPublisher::new("ipo-info-fetcher")),
            Arc::new(FirestoreOperationLogRepository::new()),
        );

        let output = use_case.execute().await.expect("execute");

        assert_eq!(output.fetched_count, 2);
        assert_eq!(stock_repository.find_all_calls.load(Ordering::SeqCst), 1);
    }
}
