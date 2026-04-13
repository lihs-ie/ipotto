import express from "express";
import { describe, expect, it } from "vitest";

import {
  createTemporaryBrowserSessionBaseDir,
  fixtureRootDirectory,
  InMemoryActiveSecuritiesAccountRepository,
  InMemoryExclusionRepository,
  InMemoryIpoStockRepository,
  InMemoryLotteryApplicationRepository,
  InMemoryNotificationEventPublisher,
  InMemoryOperationLogRepository,
  startExpressHttpServer,
  startFixtureHttpServer,
} from "./ipo-browser-test-support.js";

describe("ipo-browser-test-support", () => {
  it("provides in-memory repositories and publishers for tests", async () => {
    const accountRepository = new InMemoryActiveSecuritiesAccountRepository([
      {
        identifier: "account-1",
        securitiesCompany: "Rakuten",
        credential: {
          loginId: "login-id",
          loginPassword: "login-password",
          tradingPassword: "trading-password",
          mailCredential: {
            mailAddress: "user@example.com",
            mailPassword: "mail-password",
            imapHost: "imap.example.com",
            imapPort: 993,
          },
        },
      },
    ]);
    const stockRepository = new InMemoryIpoStockRepository([
      {
        identifier: "stock-1",
        companyName: "テストIPO",
        price: 600,
        shares: 100,
        bookBuildingStartDate: "2026-04-10",
        bookBuildingEndDate: "2026-04-12",
      },
    ]);
    const exclusionRepository = new InMemoryExclusionRepository([
      {
        identifier: "exclusion-1",
        companyName: "除外IPO",
        reason: "manual",
      },
    ]);
    const applicationRepository = new InMemoryLotteryApplicationRepository();
    const logRepository = new InMemoryOperationLogRepository();
    const publisher = new InMemoryNotificationEventPublisher();

    expect(await accountRepository.findActive()).toHaveLength(1);
    expect(await stockRepository.findInBookBuildingPeriod("2026-04-11")).toHaveLength(
      1,
    );
    expect(await exclusionRepository.findAll()).toHaveLength(1);

    await applicationRepository.save({
      identifier: "application-1",
      stock: "stock-1",
      securitiesAccount: "account-1",
      appliedOrder: {
        shares: 100,
        price: 600,
        orderedAt: "2026-04-13T00:00:00.000Z",
      },
      lotteryOutcome: null,
      status: "Applied",
      createdAt: "2026-04-13T00:00:00.000Z",
      updatedAt: "2026-04-13T00:00:00.000Z",
    });
    await logRepository.save({
      identifier: "log-1",
      application: null,
      eventType: "ApplyLottery",
      serviceName: "ipo-browser",
      status: "Success",
      message: "done",
      errorMessage: null,
      executedAt: "2026-04-13T00:00:00.000Z",
    });
    await publisher.publish({
      eventType: "OperationErrorOccurred",
      aggregateId: "log-1",
      aggregateType: "OperationLog",
      payload: {
        service_name: "ipo-browser",
        operation_type: "apply_lottery",
        error_message: "failed",
        occurred_at: "2026-04-13T00:00:00.000Z",
      },
    });

    expect(
      await applicationRepository.existsByStockAndAccount("stock-1", "account-1"),
    ).toBe(true);
    expect(logRepository.savedEntries()).toHaveLength(1);
    expect(publisher.publishedEvents()).toHaveLength(1);
  });

  it("serves local fixture files over HTTP", async () => {
    const server = await startFixtureHttpServer();

    try {
      const ok = await fetch(`${server.baseUrl}/ipo-stocks.json`);
      const notFound = await fetch(`${server.baseUrl}/missing.json`);

      expect(ok.status).toBe(200);
      expect(await ok.json()).toBeInstanceOf(Array);
      expect(notFound.status).toBe(404);
      expect(fixtureRootDirectory()).toContain("tests/fixtures/html");
    } finally {
      await server.close();
    }
  });

  it("starts an express app on an ephemeral port", async () => {
    const app = express();
    app.get("/health", (_request, response) => {
      response.json({ status: "ok" });
    });

    const server = await startExpressHttpServer(app);

    try {
      const response = await fetch(`${server.baseUrl}/health`);
      expect(response.status).toBe(200);
      expect(await response.json()).toEqual({ status: "ok" });
    } finally {
      await server.close();
    }
  });

  it("creates unique temporary browser-session base directories", () => {
    const first = createTemporaryBrowserSessionBaseDir();
    const second = createTemporaryBrowserSessionBaseDir();

    expect(first).not.toBe(second);
    expect(first).toContain("ipotto-ipo-browser-tests");
  });
});
