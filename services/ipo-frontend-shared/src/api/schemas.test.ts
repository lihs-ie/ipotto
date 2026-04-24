import { describe, expect, it } from "vitest";

import { apiErrorSchema } from "./error-response";
import {
  getIpoStockResponseSchema,
  listIpoStocksQuerySchema,
  listIpoStocksResponseSchema,
} from "./stocks";
import { dashboardSummaryResponseSchema } from "./dashboard";
import {
  createExclusionRequestSchema,
  listExclusionsResponseSchema,
} from "./exclusions";
import {
  getNotificationSettingResponseSchema,
  updateNotificationSettingRequestSchema,
} from "./notifications";
import {
  createSecuritiesAccountRequestSchema,
  listSecuritiesAccountsResponseSchema,
  testSecuritiesAccountConnectionResponseSchema,
  updateSecuritiesAccountRequestSchema,
} from "./accounts";
import {
  listOperationLogsQuerySchema,
  listOperationLogsResponseSchema,
} from "./operation-logs";

describe("apiErrorSchema", () => {
  it("accepts error with details array", () => {
    const payload = {
      error: {
        code: "VALIDATION_ERROR",
        message: "入力値が不正です",
        details: [{ field: "mailAddress", message: "形式が不正です" }],
      },
    };
    expect(apiErrorSchema.parse(payload)).toEqual(payload);
  });

  it("accepts error without details", () => {
    const payload = {
      error: { code: "STOCK_NOT_FOUND", message: "銘柄が見つかりません" },
    };
    expect(apiErrorSchema.parse(payload)).toEqual(payload);
  });

  it("rejects missing code", () => {
    expect(() =>
      apiErrorSchema.parse({ error: { message: "missing code" } }),
    ).toThrow();
  });
});

describe("API-001 listIpoStocks", () => {
  it("parses a valid response", () => {
    const payload = {
      items: [
        {
          identifier: "stock_abc123",
          companyName: "○○株式会社",
          tickerSymbol: "1234",
          market: "Growth",
          industry: "情報・通信業",
          bookBuildingStartDate: "2026-04-01",
          bookBuildingEndDate: "2026-04-10",
          lotteryDate: "2026-04-15",
          listingDate: "2026-04-25",
          priceRangeMin: 1200,
          priceRangeMax: 1500,
          offerPrice: null,
          leadUnderwriter: "楽天証券",
          numberOfOfferedShares: 100_000,
          status: "Eligible",
        },
      ],
      totalCount: 17,
    };
    const parsed = listIpoStocksResponseSchema.parse(payload);
    expect(parsed.items).toHaveLength(1);
    expect(parsed.items[0]?.status).toBe("Eligible");
  });

  it("accepts query without status filter", () => {
    expect(listIpoStocksQuerySchema.parse({})).toEqual({});
  });

  it("rejects invalid status", () => {
    expect(() =>
      listIpoStocksQuerySchema.parse({ status: "Pending" }),
    ).toThrow();
  });
});

describe("API-002 getIpoStock", () => {
  it("parses response with applications", () => {
    const payload = {
      identifier: "stock_abc123",
      companyProfile: {
        companyName: "○○株式会社",
        tickerSymbol: "1234",
        market: "Growth",
        industry: "情報・通信業",
      },
      schedule: {
        bookBuildingStartDate: "2026-04-01",
        bookBuildingEndDate: "2026-04-10",
        lotteryDate: "2026-04-15",
        listingDate: "2026-04-25",
      },
      pricing: {
        priceRangeMin: 1200,
        priceRangeMax: 1500,
        offerPrice: 1400,
      },
      offering: { leadUnderwriter: "楽天証券", numberOfOfferedShares: 100_000 },
      status: "Applied",
      metaSource: {
        source: "ExternalSite",
        fetchedAt: "2026-03-25T09:30:00Z",
      },
      applications: [
        {
          identifier: "app_xyz789",
          securitiesCompany: "楽天証券",
          appliedShares: 100,
          appliedPrice: 1400,
          appliedAt: "2026-04-05T10:00:00Z",
          lotteryOutcome: null,
          status: "Applied",
        },
      ],
    };
    const parsed = getIpoStockResponseSchema.parse(payload);
    expect(parsed.applications[0]?.status).toBe("Applied");
  });
});

describe("API-003 dashboard", () => {
  it("parses status counts and recent activities", () => {
    const payload = {
      statusCounts: { Fetched: 2, Eligible: 5, Applied: 3 },
      recentActivities: [
        {
          stock: "stock_abc",
          companyName: "A社",
          securitiesCompany: "楽天証券",
          eventType: "ApplicationCompleted",
          occurredAt: "2026-03-25T10:00:00Z",
        },
      ],
      upcomingStocks: [],
      systemStatus: {
        nextJobScheduledAt: "2026-04-01T09:00:00Z",
        accounts: [
          {
            securitiesCompany: "楽天証券",
            connectionStatus: "healthy",
            lastTestedAt: "2026-03-25T10:00:00Z",
          },
        ],
      },
    };
    const parsed = dashboardSummaryResponseSchema.parse(payload);
    expect(parsed.statusCounts.Fetched).toBe(2);
  });
});

describe("API-004/005 exclusions", () => {
  it("parses list response", () => {
    const payload = {
      items: [
        {
          identifier: "excl_abc",
          companyName: "除外株式会社",
          reason: "テスト",
          registeredAt: "2026-03-25T10:00:00Z",
        },
      ],
      totalCount: 1,
    };
    expect(listExclusionsResponseSchema.parse(payload).totalCount).toBe(1);
  });

  it("validates create request", () => {
    expect(
      createExclusionRequestSchema.parse({
        companyName: "除外株式会社",
        reason: "テスト理由",
      }),
    ).toEqual({ companyName: "除外株式会社", reason: "テスト理由" });
  });
});

describe("API-007/008 notification settings", () => {
  const base = {
    identifier: "default",
    enabled: true,
    channels: [
      {
        identifier: "ch_line_001",
        channelType: "LINE",
        destination: { accessToken: "xxx" },
        enabled: true,
        subscriptions: { ApplicationCompleted: true, LotteryResultWon: true },
      },
    ],
  };

  it("parses get response", () => {
    expect(getNotificationSettingResponseSchema.parse(base).enabled).toBe(true);
  });

  it("validates update request (full replace)", () => {
    const payload = {
      enabled: true,
      channels: [
        {
          channelType: "Email",
          destination: { address: "ops@example.com" },
          enabled: true,
          subscriptions: { OperationError: true },
        },
      ],
    };
    expect(updateNotificationSettingRequestSchema.parse(payload).enabled).toBe(
      true,
    );
  });
});

describe("API-009/010/011/013 accounts", () => {
  it("parses list response with masked credentials", () => {
    const payload = {
      items: [
        {
          identifier: "acct_abc",
          securitiesCompany: "Rakuten",
          loginIdMasked: "r****n",
          mailAddressMasked: "u***@example.com",
          imapHost: "imap.example.com",
          imapPort: 993,
          active: true,
          registeredAt: "2026-03-25T10:00:00Z",
          lastTestedAt: null,
          lastTestedStatus: null,
        },
      ],
      totalCount: 1,
    };
    expect(
      listSecuritiesAccountsResponseSchema.parse(payload).items[0]?.active,
    ).toBe(true);
  });

  it("validates create request", () => {
    expect(() =>
      createSecuritiesAccountRequestSchema.parse({
        securitiesCompany: "Rakuten",
        loginId: "login",
        loginPassword: "password",
        tradingPassword: "1234",
        mailAddress: "user@example.com",
        mailPassword: "mail-password",
        imapHost: "imap.example.com",
        imapPort: 993,
      }),
    ).not.toThrow();
  });

  it("accepts empty update (no-op)", () => {
    expect(updateSecuritiesAccountRequestSchema.parse({})).toEqual({});
  });

  it("parses connection test response", () => {
    expect(
      testSecuritiesAccountConnectionResponseSchema.parse({
        success: false,
        message: "invalid credentials",
        testedAt: "2026-03-25T10:00:00Z",
      }).success,
    ).toBe(false);
  });
});

describe("API-014 operation logs", () => {
  it("parses list response with cursor + hasMore", () => {
    const payload = {
      items: [
        {
          identifier: "log_001",
          stock: null,
          eventType: "fetch_stocks",
          serviceName: "ipo-info-fetcher",
          status: "succeeded",
          message: "ok",
          errorDetails: null,
          executedAt: "2026-03-25T10:00:00Z",
        },
      ],
      nextCursor: "opaque-cursor",
      hasMore: true,
    };
    const parsed = listOperationLogsResponseSchema.parse(payload);
    expect(parsed.items).toHaveLength(1);
    expect(parsed.hasMore).toBe(true);
  });

  it("rejects response missing hasMore field", () => {
    expect(() =>
      listOperationLogsResponseSchema.parse({
        items: [],
        nextCursor: null,
      }),
    ).toThrow();
  });

  it("validates query pagination", () => {
    expect(
      listOperationLogsQuerySchema.parse({ limit: 50, cursor: "abc" }),
    ).toEqual({ limit: 50, cursor: "abc" });
  });

  it("rejects limit over 200", () => {
    expect(() =>
      listOperationLogsQuerySchema.parse({ limit: 500 }),
    ).toThrow();
  });
});
