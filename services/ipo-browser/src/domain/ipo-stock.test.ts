import { describe, expect, it } from "vitest";

import type {
  ExclusionEntry,
  ScrapedStock,
  TargetIpoStock,
} from "./ipo-stock.js";

describe("ipo-stock contracts", () => {
  it("keeps the target stock shape used by the apply workflow", () => {
    const stock: TargetIpoStock = {
      identifier: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      companyName: "テストIPO",
      price: 600,
      shares: 100,
      bookBuildingStartDate: "2026-04-10",
      bookBuildingEndDate: "2026-04-12",
    };

    expect(stock.price * stock.shares).toBe(60_000);
  });

  it("keeps exclusion and scraped stock payload shapes", () => {
    const exclusion: ExclusionEntry = {
      identifier: "01ARZ3NDEKTSV4RRFFQ69G5FAA",
      companyName: "除外IPO",
      reason: "manual",
    };
    const scrapedStock: ScrapedStock = {
      company_name: "テストIPO",
      ticker_symbol: "1234",
      market: "東証グロース",
      industry: "情報通信",
      book_building_start_date: "2026-04-10",
      book_building_end_date: "2026-04-12",
      lottery_date: "2026-04-15",
      listing_date: "2026-04-20",
      price_range_min: 500,
      price_range_max: 600,
      offer_price: 600,
      lead_underwriter: "楽天証券",
      number_of_offered_shares: 1000,
    };

    expect(exclusion.reason).toBe("manual");
    expect(scrapedStock.price_range_max).toBeGreaterThan(
      scrapedStock.price_range_min,
    );
  });
});
