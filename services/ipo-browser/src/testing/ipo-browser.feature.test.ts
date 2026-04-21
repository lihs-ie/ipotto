import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { fromComponents } from "../infrastructure/dependency-container.js";
import { MockStockCatalogClient } from "../infrastructure/rakuten/mock-stock-catalog-client.js";
import { PageFactory } from "../infrastructure/rakuten/page-factory.js";
import type { RakutenNavigationTargetResolver } from "../infrastructure/rakuten/rakuten-navigation-target-resolver.js";
import {
  FixtureRakutenNavigationTargetResolver,
} from "../infrastructure/rakuten/rakuten-navigation-target-resolver.js";
import {
  RakutenBrokerAdapter,
  StaticImageAuthenticationKeywordProvider,
} from "../infrastructure/rakuten/rakuten-broker-adapter.js";
import { BrowserSessionStorage } from "../infrastructure/session/browser-session-storage.js";
import { createApp } from "../server.js";
import type { TargetIpoStock } from "../domain/ipo-stock.js";
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

class ImageAuthenticationFixtureResolver
  implements RakutenNavigationTargetResolver
{
  private readonly delegate: FixtureRakutenNavigationTargetResolver;

  /**
   * Creates the resolver.
   */
  public constructor(private readonly baseUrl: string) {
    this.delegate = new FixtureRakutenNavigationTargetResolver(baseUrl);
  }

  /**
   * Returns the login page URL with image-authentication enabled.
   */
  public resolveLoginPageUrl(): string {
    return new URL("/rakuten/login_page.html?auth=image", this.baseUrl).toString();
  }

  /**
   * Returns whether the IPO list route is available.
   */
  public hasIpoListPageTarget(): boolean {
    return this.delegate.hasIpoListPageTarget();
  }

  /**
   * Returns the fixture IPO list page URL.
   */
  public resolveIpoListPageUrl(stock: TargetIpoStock): string {
    return this.delegate.resolveIpoListPageUrl(stock);
  }

  /**
   * Returns the fixture application page URL.
   */
  public resolveApplicationPageUrl(stock: TargetIpoStock): string {
    return this.delegate.resolveApplicationPageUrl(stock);
  }
}

describe("ipo-browser feature", () => {
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

  it(
    "applies for a lottery through the real app, real adapter, and fixture HTML flow",
    async () => {
      const fixtureServer = await startFixtureHttpServer();
      const browserSessionBaseDir = await mkdtemp(
        join(tmpdir(), "ipotto-ipo-browser-feature-"),
      );
      cleanupTargets.push(browserSessionBaseDir);

      const accountRepository = new InMemoryActiveSecuritiesAccountRepository([
        {
          identifier: "01JS8H9F7Y7P5X1F6PWBJ0A0A1",
          securitiesCompany: "Rakuten",
          credential: {
            loginId: "feature-login",
            loginPassword: "feature-password",
            tradingPassword: "1234",
            mailCredential: {
              mailAddress: "feature@example.com",
              mailPassword: "feature-mail-password",
              imapHost: "imap.example.com",
              imapPort: 993,
            },
          },
        },
      ]);
      const stockRepository = new InMemoryIpoStockRepository([
        {
          identifier: "01JS8H9F7Y7P5X1F6PWBJ0A0B2",
          companyName: "テスト株式会社",
          price: 1400,
          shares: 100,
          bookBuildingStartDate: "2026-04-01",
          bookBuildingEndDate: "2026-04-10",
        },
      ]);
      const applicationRepository = new InMemoryLotteryApplicationRepository();
      const operationLogRepository = new InMemoryOperationLogRepository();
      const eventPublisher = new InMemoryNotificationEventPublisher();
      const broker = new RakutenBrokerAdapter(
        new PageFactory(),
        new BrowserSessionStorage(browserSessionBaseDir),
        new StaticImageAuthenticationKeywordProvider(["さくら", "みかん"]),
        new ImageAuthenticationFixtureResolver(fixtureServer.baseUrl),
        { mockLotteryResult: "Won" },
      );
      const app = createApp(
        fromComponents(
          accountRepository,
          stockRepository,
          new InMemoryExclusionRepository(),
          applicationRepository,
          operationLogRepository,
          eventPublisher,
          broker,
          new MockStockCatalogClient(`${fixtureServer.baseUrl}/ipo-stocks.json`),
        ),
      );
      const appServer = await startExpressHttpServer(app);

      try {
        const response = await fetch(`${appServer.baseUrl}/internal/pubsub/apply`, {
          method: "POST",
          headers: {
            "content-type": "application/json",
          },
          body: JSON.stringify({
            targetDate: "2026-04-10",
          }),
        });

        expect(response.status).toBe(200);
        await expect(response.json()).resolves.toMatchObject({
          appliedCount: 1,
          skippedCount: 0,
          failedCount: 0,
          results: [
            {
              accountId: "01JS8H9F7Y7P5X1F6PWBJ0A0A1",
              stockId: "01JS8H9F7Y7P5X1F6PWBJ0A0B2",
              result: "success",
              reason: null,
              failureCategory: null,
            },
          ],
        });
        expect(applicationRepository.savedRecords()).toHaveLength(1);
        expect(operationLogRepository.savedEntries()).toHaveLength(1);
        expect(operationLogRepository.savedEntries()[0]).toMatchObject({
          status: "Success",
          serviceName: "ipo-browser",
        });
        expect(eventPublisher.publishedEvents()).toHaveLength(1);
        expect(eventPublisher.publishedEvents()[0]).toMatchObject({
          eventType: "ApplicationCompleted",
          aggregateType: "LotteryApplication",
          payload: {
            stock: "01JS8H9F7Y7P5X1F6PWBJ0A0B2",
            securities_account: "01JS8H9F7Y7P5X1F6PWBJ0A0A1",
            applied_shares: 100,
            applied_price: 1400,
          },
        });
      } finally {
        await appServer.close();
        await fixtureServer.close();
      }
    },
    60_000,
  );
});
