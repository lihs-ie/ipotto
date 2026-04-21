import { describe, expect, it } from "vitest";

import {
  FixtureRakutenNavigationTargetResolver,
  inferFixtureScenario,
  ProductionRakutenNavigationTargetResolver,
} from "./rakuten-navigation-target-resolver.js";

const STOCK = {
  identifier: "stock-1",
  companyName: "テストIPO",
  price: 600,
  shares: 100,
  bookBuildingStartDate: "2026-04-10",
  bookBuildingEndDate: "2026-04-12",
} as const;

describe("ProductionRakutenNavigationTargetResolver", () => {
  it("returns configured URLs", () => {
    const resolver = new ProductionRakutenNavigationTargetResolver(
      "https://example.com/login",
      "https://example.com/ipo",
      "https://example.com/apply",
    );

    expect(resolver.resolveLoginPageUrl()).toBe("https://example.com/login");
    expect(resolver.hasIpoListPageTarget()).toBe(true);
    expect(resolver.resolveIpoListPageUrl(STOCK)).toBe("https://example.com/ipo");
    expect(resolver.resolveApplicationPageUrl(STOCK)).toBe(
      "https://example.com/apply",
    );
  });

  it("throws when required production URLs are missing", () => {
    const resolver = new ProductionRakutenNavigationTargetResolver(null, null, null);

    expect(() => resolver.resolveLoginPageUrl()).toThrow(
      "RAKUTEN_LOGIN_PAGE_URL is not configured",
    );
    expect(() => resolver.resolveIpoListPageUrl(STOCK)).toThrow(
      "RAKUTEN_IPO_LIST_PAGE_URL is not configured",
    );
    expect(() => resolver.resolveApplicationPageUrl(STOCK)).toThrow(
      "RAKUTEN_APPLICATION_PAGE_URL is not configured",
    );
  });
});

describe("FixtureRakutenNavigationTargetResolver", () => {
  it("encodes fixture scenarios in navigation targets", () => {
    const resolver = new FixtureRakutenNavigationTargetResolver(
      "http://127.0.0.1:4010",
    );

    expect(resolver.resolveLoginPageUrl()).toBe(
      "http://127.0.0.1:4010/rakuten/login_page.html",
    );
    expect(resolver.hasIpoListPageTarget()).toBe(true);
    expect(resolver.resolveIpoListPageUrl(STOCK)).toContain("scenario=success");
    expect(resolver.resolveIpoListPageUrl(STOCK)).toContain(
      "companyName=%E3%83%86%E3%82%B9%E3%83%88IPO",
    );
    expect(resolver.resolveApplicationPageUrl(STOCK)).toContain(
      "scenario=success",
    );
  });

  it("infers fixture scenarios from company names", () => {
    expect(inferFixtureScenario({ ...STOCK, companyName: "申込済IPO" })).toBe(
      "already_applied",
    );
    expect(
      inferFixtureScenario({ ...STOCK, companyName: "残高不足IPO" }),
    ).toBe("insufficient_balance");
    expect(inferFixtureScenario({ ...STOCK, companyName: "失敗IPO" })).toBe(
      "failure",
    );
    expect(inferFixtureScenario(STOCK)).toBe("success");
  });
});
