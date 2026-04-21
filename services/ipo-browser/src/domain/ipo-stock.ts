/**
 * IPO stock information required for DD-101.
 */
export interface TargetIpoStock {
  readonly identifier: string;
  readonly companyName: string;
  readonly price: number;
  readonly shares: number;
  readonly bookBuildingStartDate: string;
  readonly bookBuildingEndDate: string;
}

/**
 * Exclusion rule used during eligibility checks.
 */
export interface ExclusionEntry {
  readonly identifier: string;
  readonly companyName: string;
  readonly reason: string;
}

/**
 * Stock payload exposed from ipo-browser to Rust services.
 */
export interface ScrapedStock {
  readonly company_name: string;
  readonly ticker_symbol: string | null;
  readonly market: string;
  readonly industry: string;
  readonly book_building_start_date: string;
  readonly book_building_end_date: string;
  readonly lottery_date: string;
  readonly listing_date: string;
  readonly price_range_min: number;
  readonly price_range_max: number;
  readonly offer_price: number | null;
  readonly lead_underwriter: string;
  readonly number_of_offered_shares: number;
}
