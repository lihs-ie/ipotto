import { promises as fs } from "node:fs";
import { join } from "node:path";

import { type BrowserContext, type Page } from "playwright";

import {
  allSelectors,
  loadSelectors,
  type SelectorDefinition,
} from "../config/selectors.js";

// Phase 3 Sprint 5.3 — Rakuten login flow. Uses selectors.yaml for
// primary + fallback selectors so minor markup rotations stay
// survivable. Sprint 6.4 wires in an optional twoFactorHandler that
// fires whenever the image-authentication container appears after
// form submission; the handler is a single injected closure so
// accounts.ts can compose it with IMAP / Gmail readers + the
// image-auth flow without this module having to know about either.

export type TwoFactorHandlerArgs = {
  readonly page: Page;
};

export type TwoFactorOutcome =
  | { readonly status: "authenticated"; readonly message?: string }
  | { readonly status: "bypassed"; readonly message?: string }
  | {
      readonly status: "failed";
      readonly reason: string;
      readonly screenshotPath: string | null;
    };

export type TwoFactorHandler = (
  args: TwoFactorHandlerArgs,
) => Promise<TwoFactorOutcome>;

export type LoginOptions = {
  readonly loginId: string;
  readonly password: string;
  readonly loginPageUrl: string;
  readonly twoFactorHandler?: TwoFactorHandler;
};

export type LoginSuccess = {
  readonly status: "success";
  readonly message: string;
};

export type LoginFailure = {
  readonly status: "failure";
  readonly reason: string;
  readonly screenshotPath: string | null;
};

export type LoginResult = LoginSuccess | LoginFailure;

const SCREENSHOT_DIR =
  process.env["BROWSER_SCREENSHOT_DIR"] ?? "/tmp/ipo-browser-screenshots";
const TWO_FACTOR_DETECTION_TIMEOUT_MS = 3_000;

export async function rakutenLogin(
  context: BrowserContext,
  options: LoginOptions,
): Promise<LoginResult> {
  const loginSelectors = loadSelectors().rakuten.login;
  const imageAuthSelectors = loadSelectors().rakuten.imageAuthentication;
  const page = await context.newPage();
  try {
    await page.goto(options.loginPageUrl, { waitUntil: "domcontentloaded" });

    const loginIdSelector = await firstMatchingSelector(
      page,
      loginSelectors.loginIdInput,
    );
    if (loginIdSelector === null) {
      return failure(
        page,
        "selectors_exhausted",
        "loginId",
        allSelectors(loginSelectors.loginIdInput),
      );
    }
    await page.fill(loginIdSelector, options.loginId);

    const passwordSelector = await firstMatchingSelector(
      page,
      loginSelectors.passwordInput,
    );
    if (passwordSelector === null) {
      return failure(
        page,
        "selectors_exhausted",
        "password",
        allSelectors(loginSelectors.passwordInput),
      );
    }
    await page.fill(passwordSelector, options.password);

    const submitSelector = await firstMatchingSelector(
      page,
      loginSelectors.submitButton,
    );
    if (submitSelector === null) {
      return failure(
        page,
        "selectors_exhausted",
        "submit",
        allSelectors(loginSelectors.submitButton),
      );
    }
    await page.click(submitSelector);

    await page
      .waitForLoadState("networkidle", { timeout: 10_000 })
      .catch(() => undefined);

    const twoFactorVisible = await detectImageAuthContainer(
      page,
      imageAuthSelectors.container,
      TWO_FACTOR_DETECTION_TIMEOUT_MS,
    );
    if (twoFactorVisible) {
      if (options.twoFactorHandler === undefined) {
        const screenshotPath = await tryScreenshot(
          page,
          "login-two-factor-no-handler",
        );
        return {
          status: "failure",
          reason:
            "rakuten presented the image-auth screen but no twoFactorHandler was configured",
          screenshotPath,
        };
      }
      const outcome = await options.twoFactorHandler({ page });
      switch (outcome.status) {
        case "authenticated":
          return {
            status: "success",
            message:
              outcome.message ?? "rakuten login + 2FA flow completed",
          };
        case "bypassed":
          return {
            status: "success",
            message:
              outcome.message ??
              "rakuten login completed (2FA bypassed by handler)",
          };
        case "failed":
          return {
            status: "failure",
            reason: outcome.reason,
            screenshotPath: outcome.screenshotPath,
          };
      }
    }

    return {
      status: "success",
      message: "rakuten login flow completed",
    };
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    const screenshotPath = await tryScreenshot(page, "login-exception");
    return {
      status: "failure",
      reason,
      screenshotPath,
    };
  } finally {
    await page.close().catch(() => undefined);
  }
}

async function firstMatchingSelector(
  page: Page,
  definition: SelectorDefinition,
): Promise<string | null> {
  for (const selector of allSelectors(definition)) {
    const count = await page
      .locator(selector)
      .count()
      .catch(() => 0);
    if (count > 0) {
      return selector;
    }
  }
  return null;
}

async function detectImageAuthContainer(
  page: Page,
  definition: SelectorDefinition,
  timeoutMs: number,
): Promise<boolean> {
  for (const selector of allSelectors(definition)) {
    const visible = await page
      .locator(selector)
      .first()
      .waitFor({ state: "visible", timeout: timeoutMs })
      .then(() => true)
      .catch(() => false);
    if (visible) {
      return true;
    }
  }
  return false;
}

async function failure(
  page: Page,
  code: "selectors_exhausted",
  role: string,
  candidates: readonly string[],
): Promise<LoginFailure> {
  const screenshotPath = await tryScreenshot(page, `login-${role}-not-found`);
  return {
    status: "failure",
    reason: `${code}: no matching selector for '${role}' (tried ${candidates.join(", ")})`,
    screenshotPath,
  };
}

async function tryScreenshot(
  page: Page,
  label: string,
): Promise<string | null> {
  try {
    await fs.mkdir(SCREENSHOT_DIR, { recursive: true });
    const path = join(SCREENSHOT_DIR, `${label}-${Date.now()}.png`);
    await page.screenshot({ path, fullPage: true });
    return path;
  } catch {
    return null;
  }
}
