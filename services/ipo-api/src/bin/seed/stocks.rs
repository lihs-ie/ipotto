//! Seed data for `ipo_stocks`. Produces 12 stocks covering every
//! [`StockStatus`] variant so dashboard summary counts show real numbers
//! and filter views have at least one example per state.

use chrono::{Duration, Utc};
use ipo_backend_shared::{
    domain::stock::{
        BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry, IpoOffering,
        IpoPricing, IpoSchedule, IpoStock, LeadUnderwriter, Market, MetaSource, PriceRange, Shares,
        StockIdentifier, StockStatus, TickerSymbol, Yen,
    },
    errors::DomainError,
};
use ulid::Ulid;

pub fn build_stocks() -> Result<Vec<IpoStock>, DomainError> {
    let today = Utc::now().date_naive();
    let seeds: [StockSeed; 12] = [
        StockSeed {
            index: 1,
            company_name: "株式会社テックスター",
            ticker: Some("4001"),
            market: Market::Growth,
            industry: "情報・通信業",
            book_building_offset: 3,
            book_building_duration: 5,
            lottery_offset: 12,
            listing_offset: 20,
            min_price: 1_200,
            max_price: 1_500,
            offer_price: None,
            lead_underwriter: "SBI証券",
            offered_shares: 500_000,
            status: StockStatus::Fetched,
        },
        StockSeed {
            index: 2,
            company_name: "グリーンエナジー株式会社",
            ticker: Some("4002"),
            market: Market::Prime,
            industry: "電気・ガス業",
            book_building_offset: 7,
            book_building_duration: 5,
            lottery_offset: 15,
            listing_offset: 23,
            min_price: 2_100,
            max_price: 2_500,
            offer_price: None,
            lead_underwriter: "野村證券",
            offered_shares: 1_200_000,
            status: StockStatus::Eligible,
        },
        StockSeed {
            index: 3,
            company_name: "メディカルバイオ株式会社",
            ticker: Some("4003"),
            market: Market::Growth,
            industry: "医薬品",
            book_building_offset: -8,
            book_building_duration: 4,
            lottery_offset: -1,
            listing_offset: 9,
            min_price: 1_800,
            max_price: 2_200,
            offer_price: Some(2_100),
            lead_underwriter: "大和証券",
            offered_shares: 300_000,
            status: StockStatus::Applied,
        },
        StockSeed {
            index: 4,
            company_name: "アーバンロジスティクス株式会社",
            ticker: Some("4004"),
            market: Market::Standard,
            industry: "陸運業",
            book_building_offset: -22,
            book_building_duration: 5,
            lottery_offset: -12,
            listing_offset: -2,
            min_price: 1_600,
            max_price: 1_900,
            offer_price: Some(1_850),
            lead_underwriter: "SMBC日興証券",
            offered_shares: 450_000,
            status: StockStatus::Won,
        },
        StockSeed {
            index: 5,
            company_name: "クリーンフードテック株式会社",
            ticker: Some("4005"),
            market: Market::Growth,
            industry: "食料品",
            book_building_offset: -28,
            book_building_duration: 5,
            lottery_offset: -15,
            listing_offset: -5,
            min_price: 1_050,
            max_price: 1_300,
            offer_price: Some(1_200),
            lead_underwriter: "みずほ証券",
            offered_shares: 280_000,
            status: StockStatus::Lost,
        },
        StockSeed {
            index: 6,
            company_name: "サイバーセキュアNetworks株式会社",
            ticker: Some("4006"),
            market: Market::Growth,
            industry: "情報・通信業",
            book_building_offset: -30,
            book_building_duration: 5,
            lottery_offset: -18,
            listing_offset: -8,
            min_price: 2_800,
            max_price: 3_200,
            offer_price: Some(3_100),
            lead_underwriter: "楽天証券",
            offered_shares: 200_000,
            status: StockStatus::Alternate,
        },
        StockSeed {
            index: 7,
            company_name: "リテールAI株式会社",
            ticker: Some("4007"),
            market: Market::Prime,
            industry: "小売業",
            book_building_offset: -40,
            book_building_duration: 5,
            lottery_offset: -25,
            listing_offset: -15,
            min_price: 3_500,
            max_price: 4_000,
            offer_price: Some(3_900),
            lead_underwriter: "SBI証券",
            offered_shares: 800_000,
            status: StockStatus::Purchased,
        },
        StockSeed {
            index: 8,
            company_name: "ネクストステージゲームス株式会社",
            ticker: Some("4008"),
            market: Market::Growth,
            industry: "情報・通信業",
            book_building_offset: -50,
            book_building_duration: 5,
            lottery_offset: -35,
            listing_offset: -25,
            min_price: 1_700,
            max_price: 2_000,
            offer_price: Some(1_950),
            lead_underwriter: "野村證券",
            offered_shares: 400_000,
            status: StockStatus::Declined,
        },
        StockSeed {
            index: 9,
            company_name: "アーキテクトビルド株式会社",
            ticker: Some("4009"),
            market: Market::Standard,
            industry: "建設業",
            book_building_offset: -80,
            book_building_duration: 5,
            lottery_offset: -60,
            listing_offset: -45,
            min_price: 2_200,
            max_price: 2_600,
            offer_price: Some(2_500),
            lead_underwriter: "大和証券",
            offered_shares: 600_000,
            status: StockStatus::Sold,
        },
        StockSeed {
            index: 10,
            company_name: "怪しいバイブコイン株式会社",
            ticker: Some("4010"),
            market: Market::Growth,
            industry: "金融業",
            book_building_offset: -5,
            book_building_duration: 5,
            lottery_offset: 3,
            listing_offset: 13,
            min_price: 500,
            max_price: 800,
            offer_price: None,
            lead_underwriter: "新興証券",
            offered_shares: 100_000,
            status: StockStatus::Excluded,
        },
        StockSeed {
            index: 11,
            company_name: "フェイクニュース株式会社",
            ticker: Some("4011"),
            market: Market::Growth,
            industry: "サービス業",
            book_building_offset: -3,
            book_building_duration: 5,
            lottery_offset: 5,
            listing_offset: 15,
            min_price: 900,
            max_price: 1_100,
            offer_price: None,
            lead_underwriter: "新興証券",
            offered_shares: 150_000,
            status: StockStatus::Failed,
        },
        StockSeed {
            index: 12,
            company_name: "スマートモビリティ株式会社",
            ticker: Some("4012"),
            market: Market::Prime,
            industry: "輸送用機器",
            book_building_offset: 14,
            book_building_duration: 5,
            lottery_offset: 22,
            listing_offset: 30,
            min_price: 2_600,
            max_price: 3_000,
            offer_price: None,
            lead_underwriter: "SMBC日興証券",
            offered_shares: 2_000_000,
            status: StockStatus::Fetched,
        },
    ];

    seeds
        .into_iter()
        .map(|seed| seed.into_stock(today))
        .collect()
}

pub fn stock_identifier(index: u8) -> Result<StockIdentifier, DomainError> {
    let ulid = Ulid::from_parts(1_700_000_000_000, u128::from(index));
    StockIdentifier::new(ulid.to_string())
}

struct StockSeed {
    index: u8,
    company_name: &'static str,
    ticker: Option<&'static str>,
    market: Market,
    industry: &'static str,
    book_building_offset: i64,
    book_building_duration: i64,
    lottery_offset: i64,
    listing_offset: i64,
    min_price: i64,
    max_price: i64,
    offer_price: Option<i64>,
    lead_underwriter: &'static str,
    offered_shares: u32,
    status: StockStatus,
}

impl StockSeed {
    fn into_stock(self, today: chrono::NaiveDate) -> Result<IpoStock, DomainError> {
        let start = today + Duration::days(self.book_building_offset);
        let end = start + Duration::days(self.book_building_duration);
        let lottery = today + Duration::days(self.lottery_offset);
        let listing = today + Duration::days(self.listing_offset);

        let ticker = match self.ticker {
            Some(value) => Some(TickerSymbol::new(value)?),
            None => None,
        };
        let profile = CompanyProfile::new(
            CompanyName::new(self.company_name)?,
            ticker,
            self.market,
            Industry::new(self.industry)?,
        )?;
        let schedule = IpoSchedule::new(BookBuildingPeriod::new(start, end)?, lottery, listing)?;
        let offer_price = match self.offer_price {
            Some(value) => Some(Yen::new(value)?),
            None => None,
        };
        let pricing = IpoPricing::new(
            PriceRange::new(Yen::new(self.min_price)?, Yen::new(self.max_price)?)?,
            offer_price,
        )?;
        let offering = IpoOffering::new(
            LeadUnderwriter::new(self.lead_underwriter)?,
            Shares::new(self.offered_shares)?,
        )?;
        let fetched_at = Utc::now() - chrono::Duration::hours(2);
        let meta = MetaSource::new(FetchOrigin::ExternalSite, fetched_at);

        IpoStock::reconstruct(
            stock_identifier(self.index)?,
            profile,
            schedule,
            pricing,
            offering,
            self.status,
            meta,
        )
    }
}
