import { afterEach, describe, expect, it, vi } from "vitest";

import { MockStockCatalogClient } from "./mock-stock-catalog-client.js";

describe("MockStockCatalogClient", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("returns an empty list when the catalog URL is unset", async () => {
    const client = new MockStockCatalogClient(null);

    await expect(client.fetchStocks()).resolves.toEqual([]);
  });

  it("fetches and returns scraped stocks", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: vi.fn().mockResolvedValue([
          {
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
          },
        ]),
      }),
    );

    const client = new MockStockCatalogClient("https://example.com/catalog.json");

    await expect(client.fetchStocks()).resolves.toHaveLength(1);
  });

  it("throws when the HTTP response is not ok or the payload is invalid", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn()
        .mockResolvedValueOnce({
          ok: false,
          status: 503,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: vi.fn().mockResolvedValue({ items: [] }),
        }),
    );

    const client = new MockStockCatalogClient("https://example.com/catalog.json");

    await expect(client.fetchStocks()).rejects.toThrow(
      "failed to fetch stock catalog: 503",
    );
    await expect(client.fetchStocks()).rejects.toThrow(
      "stock catalog payload must be an array",
    );
  });
});
