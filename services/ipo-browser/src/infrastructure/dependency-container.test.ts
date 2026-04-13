import { describe, expect, it, vi } from "vitest";

import { fromComponents } from "./dependency-container.js";
import { MockStockCatalogClient } from "./rakuten/mock-stock-catalog-client.js";

describe("fromComponents", () => {
  it("exposes the injected components through the runtime container", () => {
    const accountRepository = {
      findActive: vi.fn().mockResolvedValue([]),
    };
    const stockRepository = {
      findInBookBuildingPeriod: vi.fn().mockResolvedValue([]),
    };
    const exclusionRepository = {
      findAll: vi.fn().mockResolvedValue([]),
    };
    const applicationRepository = {
      existsByStockAndAccount: vi.fn().mockResolvedValue(false),
      save: vi.fn().mockResolvedValue(undefined),
    };
    const operationLogRepository = {
      save: vi.fn().mockResolvedValue(undefined),
    };
    const eventPublisher = {
      publish: vi.fn().mockResolvedValue(undefined),
    };
    const broker = {
      applyForIpo: vi.fn(),
      testConnection: vi.fn(),
      checkLotteryResult: vi.fn(),
      fetchIpoStocks: vi.fn(),
    };
    const stockCatalog = new MockStockCatalogClient(null);

    const container = fromComponents(
      accountRepository as never,
      stockRepository as never,
      exclusionRepository as never,
      applicationRepository as never,
      operationLogRepository as never,
      eventPublisher as never,
      broker as never,
      stockCatalog,
    );

    expect(typeof container.applyForLotteryUseCase().execute).toBe("function");
    expect(container.brokerPort()).toBe(broker);
    expect(container.stockCatalogClient()).toBe(stockCatalog);
  });
});
