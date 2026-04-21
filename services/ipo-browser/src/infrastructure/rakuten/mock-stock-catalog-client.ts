import type { ScrapedStock } from "../../domain/ipo-stock.js";

/**
 * Fetches stock fixtures for the browser service.
 */
export class MockStockCatalogClient {
  /**
   * Creates the stock catalog client.
   */
  public constructor(private readonly stockCatalogUrl: string | null) {}

  /**
   * Fetches available IPO stocks.
   */
  public async fetchStocks(): Promise<readonly ScrapedStock[]> {
    if (this.stockCatalogUrl === null) {
      return [];
    }

    const response = await fetch(this.stockCatalogUrl);
    if (!response.ok) {
      throw new Error(`failed to fetch stock catalog: ${response.status}`);
    }

    const payload = (await response.json()) as unknown;
    if (!Array.isArray(payload)) {
      throw new Error("stock catalog payload must be an array");
    }

    return payload.map((item) => item as ScrapedStock);
  }
}
