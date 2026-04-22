use async_trait::async_trait;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};

use crate::{
    acl::scraping::{IpoStockScraperPort, ScrapedStock},
    errors::DomainError,
};

use super::{japanese_date_parser, japanese_shares_parser, japanese_yen_parser};

const SOURCE_LABEL: &str = "html_ipo_scraper";

/// Scraper adapter that downloads an IPO listing HTML page and extracts
/// rows from a known table structure. Acts as the middle tier of the
/// fallback chain (`JSON → HTML → Browser`), so the application still
/// gets data when the external JSON feed is down or missing entries.
///
/// The adapter targets a stable, simplified table format used by the
/// curated IPO listing mirror (see `tests/fixtures/html/ipo-listings.html`).
/// Real stock-exchange pages evolve frequently; treat this adapter as
/// one-of-many in the fallback chain rather than a sole source of truth.
#[derive(Debug, Clone)]
pub struct HtmlIpoScraperAdapter {
    client: Client,
    url: String,
    table_selector: String,
    row_selector: String,
    cell_selector: String,
}

impl HtmlIpoScraperAdapter {
    /// Creates an HTML scraper targeting the default table selectors
    /// (`table.ipo-listings`, `tbody tr`, `td`). Callers that need to
    /// scrape alternate sites can use [`Self::with_selectors`].
    pub fn new(client: Client, url: impl Into<String>) -> Self {
        Self {
            client,
            url: url.into(),
            table_selector: "table.ipo-listings".to_string(),
            row_selector: "tbody tr".to_string(),
            cell_selector: "td".to_string(),
        }
    }

    /// Creates an HTML scraper with caller-provided CSS selectors.
    pub fn with_selectors(
        client: Client,
        url: impl Into<String>,
        table_selector: impl Into<String>,
        row_selector: impl Into<String>,
        cell_selector: impl Into<String>,
    ) -> Self {
        Self {
            client,
            url: url.into(),
            table_selector: table_selector.into(),
            row_selector: row_selector.into(),
            cell_selector: cell_selector.into(),
        }
    }

    fn parse_document(&self, body: &str) -> Result<Vec<ScrapedStock>, DomainError> {
        let document = Html::parse_document(body);
        let table_selector =
            Selector::parse(&self.table_selector).map_err(|error| DomainError::ScrapingError {
                scraper_source: SOURCE_LABEL.to_string(),
                reason: format!("invalid table selector '{}': {error}", self.table_selector),
            })?;
        let row_selector =
            Selector::parse(&self.row_selector).map_err(|error| DomainError::ScrapingError {
                scraper_source: SOURCE_LABEL.to_string(),
                reason: format!("invalid row selector '{}': {error}", self.row_selector),
            })?;
        let cell_selector =
            Selector::parse(&self.cell_selector).map_err(|error| DomainError::ScrapingError {
                scraper_source: SOURCE_LABEL.to_string(),
                reason: format!("invalid cell selector '{}': {error}", self.cell_selector),
            })?;

        let table =
            document
                .select(&table_selector)
                .next()
                .ok_or_else(|| DomainError::ScrapingError {
                    scraper_source: SOURCE_LABEL.to_string(),
                    reason: format!(
                        "no element matches table selector '{}'",
                        self.table_selector
                    ),
                })?;

        let mut stocks = Vec::new();
        for row in table.select(&row_selector) {
            let cells: Vec<ElementRef<'_>> = row.select(&cell_selector).collect();
            if cells.len() < 12 {
                return Err(DomainError::ScrapingError {
                    scraper_source: SOURCE_LABEL.to_string(),
                    reason: format!(
                        "expected at least 12 cells per row, found {} for row '{}'",
                        cells.len(),
                        collapse_whitespace(&row.text().collect::<String>())
                    ),
                });
            }
            stocks.push(parse_row(&cells)?);
        }

        if stocks.is_empty() {
            return Err(DomainError::ScrapingError {
                scraper_source: SOURCE_LABEL.to_string(),
                reason: "HTML table contained no data rows".to_string(),
            });
        }
        Ok(stocks)
    }
}

#[async_trait]
impl IpoStockScraperPort for HtmlIpoScraperAdapter {
    async fn scrape(&self) -> Result<Vec<ScrapedStock>, DomainError> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .and_then(|response| response.error_for_status())
            .map_err(|error| DomainError::ScrapingError {
                scraper_source: SOURCE_LABEL.to_string(),
                reason: error.to_string(),
            })?;
        let body = response
            .text()
            .await
            .map_err(|error| DomainError::ScrapingError {
                scraper_source: SOURCE_LABEL.to_string(),
                reason: error.to_string(),
            })?;
        if body.trim().is_empty() {
            return Err(DomainError::ScrapingError {
                scraper_source: SOURCE_LABEL.to_string(),
                reason: "empty response body".to_string(),
            });
        }
        self.parse_document(&body)
    }
}

fn parse_row(cells: &[ElementRef<'_>]) -> Result<ScrapedStock, DomainError> {
    let company_name = cell_text(cells, 0)?;
    let ticker_raw = cell_text(cells, 1)?;
    let market = cell_text(cells, 2)?;
    let industry = cell_text(cells, 3)?;
    let book_building_start = japanese_date_parser::parse_japanese_date(&cell_text(cells, 4)?)?;
    let book_building_end = japanese_date_parser::parse_japanese_date(&cell_text(cells, 5)?)?;
    let lottery_date = japanese_date_parser::parse_japanese_date(&cell_text(cells, 6)?)?;
    let listing_date = japanese_date_parser::parse_japanese_date(&cell_text(cells, 7)?)?;
    let price_range_min = japanese_yen_parser::parse_japanese_yen(&cell_text(cells, 8)?)?;
    let price_range_max = japanese_yen_parser::parse_japanese_yen(&cell_text(cells, 9)?)?;
    let offer_price_cell = cell_text(cells, 10)?;
    let offer_price = if offer_price_cell.trim().is_empty() || offer_price_cell.trim() == "-" {
        None
    } else {
        Some(japanese_yen_parser::parse_japanese_yen(&offer_price_cell)?)
    };
    let lead_underwriter = cell_text(cells, 11)?;
    let shares_cell = cell_text(cells, 12).unwrap_or_else(|_| "0株".to_string());
    let number_of_offered_shares = japanese_shares_parser::parse_japanese_shares(&shares_cell)?;
    let ticker = normalize_ticker(&ticker_raw);
    Ok(ScrapedStock::new(
        company_name,
        ticker,
        market,
        industry,
        book_building_start,
        book_building_end,
        lottery_date,
        listing_date,
        price_range_min,
        price_range_max,
        offer_price,
        lead_underwriter,
        number_of_offered_shares,
    ))
}

fn cell_text(cells: &[ElementRef<'_>], index: usize) -> Result<String, DomainError> {
    let cell = cells.get(index).ok_or_else(|| DomainError::ScrapingError {
        scraper_source: SOURCE_LABEL.to_string(),
        reason: format!("missing cell at index {index}"),
    })?;
    Ok(collapse_whitespace(&cell.text().collect::<String>()))
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_ticker(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::HtmlIpoScraperAdapter;
    use crate::acl::scraping::IpoStockScraperPort;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    const FIXTURE: &str = include_str!("test_fixtures/ipo_listings.html");

    #[tokio::test]
    async fn scrapes_html_listings_fixture_into_structured_stocks() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ipo-listings.html"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(FIXTURE)
                    .insert_header("content-type", "text/html; charset=utf-8"),
            )
            .mount(&server)
            .await;

        let adapter = HtmlIpoScraperAdapter::new(
            reqwest::Client::new(),
            format!("{}/ipo-listings.html", server.uri()),
        );
        let stocks = adapter.scrape().await.expect("scrape succeeds");
        assert_eq!(stocks.len(), 2);

        let first = &stocks[0];
        assert_eq!(first.company_name(), "株式会社テスト工業");
        assert_eq!(first.ticker_symbol(), Some("1234"));
        assert_eq!(first.market(), "Growth");
        assert_eq!(first.industry(), "情報・通信業");
        assert_eq!(first.price_range_min(), 1200);
        assert_eq!(first.price_range_max(), 1500);
        assert_eq!(first.offer_price(), Some(1400));
        assert_eq!(first.lead_underwriter(), "楽天証券");
        assert_eq!(first.number_of_offered_shares(), 100_000);

        let second = &stocks[1];
        assert_eq!(second.company_name(), "テストホールディングス株式会社");
        assert_eq!(second.ticker_symbol(), None);
        assert_eq!(second.offer_price(), None);
    }

    #[tokio::test]
    async fn returns_error_when_table_is_missing() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing-table.html"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("<html><body>no table here</body></html>")
                    .insert_header("content-type", "text/html"),
            )
            .mount(&server)
            .await;

        let adapter = HtmlIpoScraperAdapter::new(
            reqwest::Client::new(),
            format!("{}/missing-table.html", server.uri()),
        );
        let error = adapter.scrape().await.expect_err("must error");
        let message = format!("{error}");
        assert!(
            message.contains("table.ipo-listings"),
            "error message surfaces missing table selector: {message}"
        );
    }

    #[tokio::test]
    async fn returns_error_when_upstream_returns_5xx() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/down.html"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;

        let adapter = HtmlIpoScraperAdapter::new(
            reqwest::Client::new(),
            format!("{}/down.html", server.uri()),
        );
        let error = adapter.scrape().await.expect_err("must error");
        assert!(format!("{error}").contains("503"));
    }

    #[test]
    fn parse_document_rejects_rows_with_too_few_cells() {
        let html = r#"<html><body><table class="ipo-listings"><tbody>
            <tr><td>A</td><td>B</td></tr>
        </tbody></table></body></html>"#;
        let adapter =
            HtmlIpoScraperAdapter::new(reqwest::Client::new(), "http://example".to_string());
        let error = adapter.parse_document(html).expect_err("must error");
        let message = format!("{error}");
        assert!(
            message.contains("expected at least 12 cells"),
            "error surfaces column shortage: {message}"
        );
    }

    #[test]
    fn normalize_ticker_returns_none_for_empty_or_dash() {
        use super::normalize_ticker;
        assert_eq!(normalize_ticker(""), None);
        assert_eq!(normalize_ticker("-"), None);
        assert_eq!(normalize_ticker("1234"), Some("1234".to_string()));
    }

    #[test]
    fn collapse_whitespace_flattens_newlines_and_tabs() {
        use super::collapse_whitespace;
        assert_eq!(collapse_whitespace("  a \n b \t c  "), "a b c");
    }

    fn _fixture_contains_expected_stocks() {
        assert!(FIXTURE.contains("株式会社テスト工業"));
    }
}
