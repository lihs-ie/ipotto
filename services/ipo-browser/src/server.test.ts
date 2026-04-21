import { afterEach, describe, expect, it, vi } from "vitest";

import { createApp } from "./server.js";
import { startExpressHttpServer } from "./testing/ipo-browser-test-support.js";

describe("createApp", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("responds from health, stock, account-test, lottery-result, and apply routes", async () => {
    const fetchStocks = vi.fn().mockResolvedValue([{ company_name: "テストIPO" }]);
    const testConnection = vi.fn().mockResolvedValue({
      success: true,
      message: "connected",
      testedAt: "2026-04-13T00:00:00.000Z",
    });
    const checkLotteryResult = vi.fn().mockResolvedValue("Won");
    const execute = vi.fn().mockResolvedValue({
      appliedCount: 1,
      skippedCount: 0,
      failedCount: 0,
      results: [],
      accountSummaries: [],
      stockSummaries: [],
    });
    const app = createApp({
      stockCatalogClient: () => ({ fetchStocks }),
      brokerPort: () => ({ testConnection, checkLotteryResult }),
      applyForLotteryUseCase: () => ({ execute }),
    } as never);
    const server = await startExpressHttpServer(app);

    try {
      const health = await fetch(`${server.baseUrl}/health`);
      const stocks = await fetch(`${server.baseUrl}/internal/stocks`);
      const accountTest = await fetch(`${server.baseUrl}/internal/accounts/test`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          loginId: "login-id",
          loginPassword: "login-password",
          tradingPassword: "trading-password",
          mailAddress: "user@example.com",
          mailPassword: "mail-password",
          imapHost: "imap.example.com",
          imapPort: 993,
        }),
      });
      const lotteryResult = await fetch(
        `${server.baseUrl}/internal/lottery-results/check`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            credential: {
              loginId: "login-id",
              loginPassword: "login-password",
              tradingPassword: "trading-password",
              mailAddress: "user@example.com",
              mailPassword: "mail-password",
              imapHost: "imap.example.com",
              imapPort: 993,
            },
            stockIdentifier: "stock-1",
          }),
        },
      );
      const apply = await fetch(`${server.baseUrl}/internal/pubsub/apply`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          targetDate: "2026-04-10",
        }),
      });

      expect(health.status).toBe(200);
      expect(await health.json()).toEqual({ status: "ok" });
      expect(stocks.status).toBe(200);
      expect(await stocks.json()).toEqual([{ company_name: "テストIPO" }]);
      expect(accountTest.status).toBe(200);
      expect(await accountTest.json()).toEqual({
        success: true,
        message: "connected",
        testedAt: "2026-04-13T00:00:00.000Z",
      });
      expect(lotteryResult.status).toBe(200);
      expect(await lotteryResult.json()).toEqual({ result: "Won" });
      expect(apply.status).toBe(200);
      expect(testConnection).toHaveBeenCalledTimes(1);
      expect(checkLotteryResult).toHaveBeenCalledWith(
        expect.objectContaining({
          loginId: "login-id",
        }),
        "stock-1",
      );
      expect(execute).toHaveBeenCalledWith({ targetDate: "2026-04-10" });
    } finally {
      await server.close();
    }
  });

  it("returns a 500 response when request validation fails", async () => {
    const app = createApp({
      stockCatalogClient: () => ({ fetchStocks: vi.fn() }),
      brokerPort: () => ({ testConnection: vi.fn(), checkLotteryResult: vi.fn() }),
      applyForLotteryUseCase: () => ({ execute: vi.fn() }),
    } as never);
    const server = await startExpressHttpServer(app);

    try {
      const response = await fetch(`${server.baseUrl}/internal/accounts/test`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ loginId: "" }),
      });

      expect(response.status).toBe(500);
      expect(await response.json()).toEqual({
        code: "INTERNAL_SERVER_ERROR",
        message: "loginId is required",
      });
    } finally {
      await server.close();
    }
  });
});
