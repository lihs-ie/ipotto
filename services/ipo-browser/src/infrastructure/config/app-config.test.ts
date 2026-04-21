import { describe, expect, it } from "vitest";

import { readAppConfig } from "./app-config.js";

describe("readAppConfig", () => {
  it("returns defaults without environment variables", () => {
    expect(readAppConfig({})).toEqual({
      port: 8081,
      gcpProjectId: "ipotto-local",
      notificationTopic: "ipo-notification",
      browserSessionBaseDir: "/tmp/browser-sessions",
      browserSessionRetentionHours: 24,
      mockServerUrl: null,
      rakutenLoginPageUrl: null,
      rakutenIpoListPageUrl: null,
      rakutenApplicationPageUrl: null,
      rakutenImageAuthenticationKeywords: null,
      mockLotteryResult: null,
      stockCatalogUrl: null,
    });
  });

  it("derives fixture URLs and keyword defaults from MOCK_SERVER_URL", () => {
    expect(
      readAppConfig({
        MOCK_SERVER_URL: "http://127.0.0.1:4010",
      }),
    ).toEqual(
      expect.objectContaining({
        mockServerUrl: "http://127.0.0.1:4010",
        rakutenLoginPageUrl: "http://127.0.0.1:4010/rakuten/login_page.html",
        rakutenIpoListPageUrl: "http://127.0.0.1:4010/rakuten/ipo_list_page.html",
        rakutenApplicationPageUrl:
          "http://127.0.0.1:4010/rakuten/ipo_application_page.html",
        rakutenImageAuthenticationKeywords: ["さくら", "みかん"],
        stockCatalogUrl: "http://127.0.0.1:4010/ipo-stocks.json",
      }),
    );
  });

  it("prefers explicit configuration values", () => {
    const config = readAppConfig({
      PORT: "9000",
      GCP_PROJECT_ID: "project-1",
      IPO_NOTIFICATION_TOPIC: "custom-topic",
      BROWSER_SESSION_BASE_DIR: "/var/tmp/ipo-browser",
      BROWSER_SESSION_RETENTION_HOURS: "12",
      RAKUTEN_LOGIN_PAGE_URL: "https://example.com/login",
      RAKUTEN_IPO_LIST_PAGE_URL: "https://example.com/ipo",
      RAKUTEN_APPLICATION_PAGE_URL: "https://example.com/apply",
      RAKUTEN_IMAGE_AUTHENTICATION_KEYWORDS: "ぶどう,りんご",
      RAKUTEN_MOCK_LOTTERY_RESULT: "Won",
      IPO_STOCK_CATALOG_URL: "https://example.com/catalog.json",
    });

    expect(config).toEqual({
      port: 9000,
      gcpProjectId: "project-1",
      notificationTopic: "custom-topic",
      browserSessionBaseDir: "/var/tmp/ipo-browser",
      browserSessionRetentionHours: 12,
      mockServerUrl: null,
      rakutenLoginPageUrl: "https://example.com/login",
      rakutenIpoListPageUrl: "https://example.com/ipo",
      rakutenApplicationPageUrl: "https://example.com/apply",
      rakutenImageAuthenticationKeywords: ["ぶどう", "りんご"],
      mockLotteryResult: "Won",
      stockCatalogUrl: "https://example.com/catalog.json",
    });
  });

  it("throws on invalid integers and unsupported lottery result values", () => {
    expect(() => readAppConfig({ PORT: "NaN" })).toThrow(
      "invalid integer value: NaN",
    );
    expect(() =>
      readAppConfig({ RAKUTEN_MOCK_LOTTERY_RESULT: "Invalid" }),
    ).toThrow("unsupported RAKUTEN_MOCK_LOTTERY_RESULT: Invalid");
  });

  it("throws when the keyword pair is malformed", () => {
    expect(() =>
      readAppConfig({
        RAKUTEN_IMAGE_AUTHENTICATION_KEYWORDS: "さくら",
      }),
    ).toThrow(
      "RAKUTEN_IMAGE_AUTHENTICATION_KEYWORDS must contain exactly two comma-separated values",
    );
  });
});
