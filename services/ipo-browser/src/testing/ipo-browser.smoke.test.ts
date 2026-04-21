import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { fromComponents } from "../infrastructure/dependency-container.js";
import { readAppConfig } from "../infrastructure/config/app-config.js";
import { MockStockCatalogClient } from "../infrastructure/rakuten/mock-stock-catalog-client.js";
import { PageFactory } from "../infrastructure/rakuten/page-factory.js";
import {
  FixtureRakutenNavigationTargetResolver,
} from "../infrastructure/rakuten/rakuten-navigation-target-resolver.js";
import {
  RakutenBrokerAdapter,
  StaticImageAuthenticationKeywordProvider,
} from "../infrastructure/rakuten/rakuten-broker-adapter.js";
import { BrowserSessionStorage } from "../infrastructure/session/browser-session-storage.js";
import { createApp } from "../server.js";
import {
  InMemoryActiveSecuritiesAccountRepository,
  InMemoryExclusionRepository,
  InMemoryIpoStockRepository,
  InMemoryLotteryApplicationRepository,
  InMemoryNotificationEventPublisher,
  InMemoryOperationLogRepository,
  startExpressHttpServer,
  startFixtureHttpServer,
} from "./ipo-browser-test-support.js";

describe("ipo-browser smoke", () => {
  const cleanupTargets: string[] = [];

  afterEach(async () => {
    while (cleanupTargets.length > 0) {
      const target = cleanupTargets.pop();
      if (target === undefined) {
        continue;
      }
      await rm(target, { recursive: true, force: true });
    }
  });

  it("boots with production-like env and serves health, stocks, and account-test routes", async () => {
    const fixtureServer = await startFixtureHttpServer();
    const browserSessionBaseDir = await mkdtemp(
      join(tmpdir(), "ipotto-ipo-browser-smoke-"),
    );
    cleanupTargets.push(browserSessionBaseDir);

    const config = readAppConfig({
      PORT: "8081",
      GCP_PROJECT_ID: "ipotto-smoke",
      IPO_NOTIFICATION_TOPIC: "ipo-notification",
      BROWSER_SESSION_BASE_DIR: browserSessionBaseDir,
      BROWSER_SESSION_RETENTION_HOURS: "24",
      MOCK_SERVER_URL: fixtureServer.baseUrl,
      RAKUTEN_IMAGE_AUTHENTICATION_KEYWORDS: "さくら,みかん",
      IPO_STOCK_CATALOG_URL: `${fixtureServer.baseUrl}/ipo-stocks.json`,
    });

    const broker = new RakutenBrokerAdapter(
      new PageFactory(),
      new BrowserSessionStorage(config.browserSessionBaseDir),
      new StaticImageAuthenticationKeywordProvider(
        config.rakutenImageAuthenticationKeywords,
      ),
      new FixtureRakutenNavigationTargetResolver(config.mockServerUrl ?? fixtureServer.baseUrl),
      { mockLotteryResult: "Won" },
    );
    const app = createApp(
      fromComponents(
        new InMemoryActiveSecuritiesAccountRepository([]),
        new InMemoryIpoStockRepository([]),
        new InMemoryExclusionRepository(),
        new InMemoryLotteryApplicationRepository(),
        new InMemoryOperationLogRepository(),
        new InMemoryNotificationEventPublisher(),
        broker,
        new MockStockCatalogClient(config.stockCatalogUrl),
      ),
    );
    const appServer = await startExpressHttpServer(app);

    try {
      const healthResponse = await fetch(`${appServer.baseUrl}/health`);
      expect(healthResponse.status).toBe(200);
      await expect(healthResponse.json()).resolves.toEqual({ status: "ok" });

      const stocksResponse = await fetch(`${appServer.baseUrl}/internal/stocks`);
      expect(stocksResponse.status).toBe(200);
      await expect(stocksResponse.json()).resolves.toMatchObject([
        {
          company_name: "CIテスト株式会社",
          ticker_symbol: "4321",
        },
      ]);

      const connectionResponse = await fetch(
        `${appServer.baseUrl}/internal/accounts/test`,
        {
          method: "POST",
          headers: {
            "content-type": "application/json",
          },
          body: JSON.stringify({
            loginId: "smoke-login",
            loginPassword: "smoke-password",
            tradingPassword: "1234",
            mailCredential: {
              mailAddress: "smoke@example.com",
              mailPassword: "smoke-mail-password",
              imapHost: "imap.example.com",
              imapPort: 993,
            },
          }),
        },
      );
      expect(connectionResponse.status).toBe(200);
      await expect(connectionResponse.json()).resolves.toMatchObject({
        success: true,
        message: "connected",
      });
    } finally {
      await appServer.close();
      await fixtureServer.close();
    }
  }, 60_000);
});
