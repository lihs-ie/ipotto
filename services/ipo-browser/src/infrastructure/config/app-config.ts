/**
 * Runtime configuration for ipo-browser.
 */
export interface AppConfig {
  readonly port: number;
  readonly gcpProjectId: string;
  readonly notificationTopic: string;
  readonly browserSessionBaseDir: string;
  readonly browserSessionRetentionHours: number;
  readonly mockServerUrl: string | null;
  readonly rakutenLoginPageUrl: string | null;
  readonly rakutenIpoListPageUrl: string | null;
  readonly rakutenApplicationPageUrl: string | null;
  readonly rakutenImageAuthenticationKeywords:
    | readonly [string, string]
    | null;
  readonly mockLotteryResult: "Won" | "Lost" | "Alternate" | null;
  readonly stockCatalogUrl: string | null;
}

/**
 * Reads runtime configuration from environment variables.
 */
export function readAppConfig(
  env: NodeJS.ProcessEnv = process.env,
): AppConfig {
  const port = parseInteger(env["PORT"], 8081);
  const mockServerUrl = normalizeOptionalString(env["MOCK_SERVER_URL"]);
  return {
    port,
    gcpProjectId:
      normalizeOptionalString(env["GCP_PROJECT_ID"]) ??
      normalizeOptionalString(env["GOOGLE_CLOUD_PROJECT"]) ??
      normalizeOptionalString(env["GCLOUD_PROJECT"]) ??
      "ipotto-local",
    notificationTopic:
      normalizeOptionalString(env["IPO_NOTIFICATION_TOPIC"]) ??
      "ipo-notification",
    browserSessionBaseDir:
      normalizeOptionalString(env["BROWSER_SESSION_BASE_DIR"]) ??
      "/tmp/browser-sessions",
    browserSessionRetentionHours: parseInteger(
      env["BROWSER_SESSION_RETENTION_HOURS"],
      24,
    ),
    mockServerUrl,
    rakutenLoginPageUrl:
      normalizeOptionalString(env["RAKUTEN_LOGIN_PAGE_URL"]) ??
      (mockServerUrl === null ? null : `${mockServerUrl}/rakuten/login_page.html`),
    rakutenIpoListPageUrl:
      normalizeOptionalString(env["RAKUTEN_IPO_LIST_PAGE_URL"]) ??
      (mockServerUrl === null ? null : `${mockServerUrl}/rakuten/ipo_list_page.html`),
    rakutenApplicationPageUrl:
      normalizeOptionalString(env["RAKUTEN_APPLICATION_PAGE_URL"]) ??
      (mockServerUrl === null
        ? null
        : `${mockServerUrl}/rakuten/ipo_application_page.html`),
    rakutenImageAuthenticationKeywords: parseKeywordPair(
      env["RAKUTEN_IMAGE_AUTHENTICATION_KEYWORDS"],
      mockServerUrl === null ? null : "さくら,みかん",
    ),
    mockLotteryResult: parseLotteryResult(env["RAKUTEN_MOCK_LOTTERY_RESULT"]),
    stockCatalogUrl:
      normalizeOptionalString(env["IPO_STOCK_CATALOG_URL"]) ??
      (mockServerUrl === null ? null : `${mockServerUrl}/ipo-stocks.json`),
  };
}

/**
 * Parses a keyword pair from a comma-separated string.
 */
function parseKeywordPair(
  value: string | undefined,
  fallback: string | null,
): readonly [string, string] | null {
  const normalized = normalizeOptionalString(value) ?? fallback;
  if (normalized === null) {
    return null;
  }
  const parts = normalized
    .split(",")
    .map((part) => part.trim())
    .filter((part) => part !== "");
  if (parts.length !== 2) {
    throw new Error(
      "RAKUTEN_IMAGE_AUTHENTICATION_KEYWORDS must contain exactly two comma-separated values",
    );
  }
  const first = parts[0];
  const second = parts[1];
  if (first === undefined || second === undefined) {
    throw new Error("image authentication keywords are incomplete");
  }
  return [first, second];
}

/**
 * Parses a positive integer with a fallback.
 */
function parseInteger(
  value: string | undefined,
  fallback: number,
): number {
  if (value === undefined || value.trim() === "") {
    return fallback;
  }
  const parsed = Number.parseInt(value, 10);
  if (Number.isNaN(parsed)) {
    throw new Error(`invalid integer value: ${value}`);
  }
  return parsed;
}

/**
 * Normalizes an optional string.
 */
function normalizeOptionalString(value: string | undefined): string | null {
  if (value === undefined) {
    return null;
  }
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}

/**
 * Parses a mock lottery result value.
 */
function parseLotteryResult(
  value: string | undefined,
): "Won" | "Lost" | "Alternate" | null {
  switch (normalizeOptionalString(value)) {
    case "Won":
    case "Lost":
    case "Alternate":
      return value as "Won" | "Lost" | "Alternate";
    case null:
      return null;
    default:
      throw new Error(`unsupported RAKUTEN_MOCK_LOTTERY_RESULT: ${value}`);
  }
}
