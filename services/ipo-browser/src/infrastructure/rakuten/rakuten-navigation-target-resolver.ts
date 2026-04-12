import type { TargetIpoStock } from "../../domain/ipo-stock.js";

/**
 * Resolves navigation targets for Rakuten browser automation.
 */
export interface RakutenNavigationTargetResolver {
  /**
   * Returns the Rakuten login page URL.
   */
  resolveLoginPageUrl(): string;

  /**
   * Returns whether the IPO list route is available.
   */
  hasIpoListPageTarget(): boolean;

  /**
   * Returns the IPO list page URL for the given stock.
   */
  resolveIpoListPageUrl(stock: TargetIpoStock): string;

  /**
   * Returns the direct IPO application page URL for the given stock.
   */
  resolveApplicationPageUrl(stock: TargetIpoStock): string;
}

/**
 * Production resolver that navigates to real Rakuten URLs without fixture hints.
 */
export class ProductionRakutenNavigationTargetResolver
  implements RakutenNavigationTargetResolver
{
  /**
   * Creates the resolver.
   */
  public constructor(
    private readonly loginPageUrl: string | null,
    private readonly ipoListPageUrl: string | null,
    private readonly applicationPageUrl: string | null,
  ) {}

  /**
   * Returns the Rakuten login page URL.
   */
  public resolveLoginPageUrl(): string {
    return requireUrl(this.loginPageUrl, "RAKUTEN_LOGIN_PAGE_URL");
  }

  /**
   * Returns whether the IPO list route is available.
   */
  public hasIpoListPageTarget(): boolean {
    return this.ipoListPageUrl !== null;
  }

  /**
   * Returns the IPO list page URL.
   */
  public resolveIpoListPageUrl(stock: TargetIpoStock): string {
    void stock;
    return requireUrl(this.ipoListPageUrl, "RAKUTEN_IPO_LIST_PAGE_URL");
  }

  /**
   * Returns the direct application page URL.
   */
  public resolveApplicationPageUrl(stock: TargetIpoStock): string {
    void stock;
    return requireUrl(
      this.applicationPageUrl,
      "RAKUTEN_APPLICATION_PAGE_URL",
    );
  }
}

/**
 * Fixture resolver that encodes scenario hints for local HTML fixtures.
 */
export class FixtureRakutenNavigationTargetResolver
  implements RakutenNavigationTargetResolver
{
  /**
   * Creates the resolver.
   */
  public constructor(
    private readonly fixtureBaseUrl: string,
    private readonly applicationPagePath = "/rakuten/ipo_application_page.html",
  ) {}

  /**
   * Returns the fixture login page URL.
   */
  public resolveLoginPageUrl(): string {
    return buildUrl(this.fixtureBaseUrl, "/rakuten/login_page.html").toString();
  }

  /**
   * Returns whether the IPO list route is available.
   */
  public hasIpoListPageTarget(): boolean {
    return true;
  }

  /**
   * Returns the fixture IPO list page URL with scenario hints.
   */
  public resolveIpoListPageUrl(stock: TargetIpoStock): string {
    const url = buildUrl(this.fixtureBaseUrl, "/rakuten/ipo_list_page.html");
    url.searchParams.set("companyName", stock.companyName);
    url.searchParams.set("scenario", inferFixtureScenario(stock));
    return url.toString();
  }

  /**
   * Returns the fixture application page URL with scenario hints.
   */
  public resolveApplicationPageUrl(stock: TargetIpoStock): string {
    const url = buildUrl(this.fixtureBaseUrl, this.applicationPagePath);
    url.searchParams.set("scenario", inferFixtureScenario(stock));
    return url.toString();
  }
}

/**
 * Infers a local fixture scenario from the stock metadata.
 */
export function inferFixtureScenario(stock: TargetIpoStock): string {
  if (stock.companyName.includes("申込済")) {
    return "already_applied";
  }
  if (stock.companyName.includes("残高不足")) {
    return "insufficient_balance";
  }
  if (stock.companyName.includes("失敗")) {
    return "failure";
  }
  return "success";
}

/**
 * Requires a configured absolute URL value.
 */
function requireUrl(value: string | null, envName: string): string {
  if (value === null) {
    throw new Error(`${envName} is not configured`);
  }
  return value;
}

/**
 * Builds a fixture URL from the base server URL and path.
 */
function buildUrl(baseUrl: string, path: string): URL {
  return new URL(path, ensureTrailingSlash(baseUrl));
}

/**
 * Ensures the base URL can resolve absolute paths consistently.
 */
function ensureTrailingSlash(value: string): string {
  return value.endsWith("/") ? value : `${value}/`;
}
