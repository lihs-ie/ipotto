import { promises as fs } from "node:fs";
import { join } from "node:path";

import { type BrowserContext, type Page } from "playwright";

import {
  allSelectors,
  loadSelectors,
  type SelectorDefinition,
} from "../config/selectors.js";

// Phase 3 Sprint 5.3 — Rakuten login flow. Uses the selectors.yaml
// definitions (primary + fallbacks) so markup rotations don't
// immediately break the flow. Phase 3 Sprint 5.4 adds a screenshot on
// total selector failure so operators can diff against a baseline.

export type LoginOptions = {
  loginId: string;
  password: string;
  loginPageUrl: string;
};

export type LoginSuccess = {
  status: "success";
  message: string;
};

export type LoginFailure = {
  status: "failure";
  reason: string;
  screenshotPath: string | null;
};

export type LoginResult = LoginSuccess | LoginFailure;

const SCREENSHOT_DIR =
  process.env["BROWSER_SCREENSHOT_DIR"] ?? "/tmp/ipo-browser-screenshots";

export async function rakutenLogin(
  context: BrowserContext,
  options: LoginOptions,
): Promise<LoginResult> {
  const selectors = loadSelectors().rakuten.login;
  const page = await context.newPage();
  try {
    await page.goto(options.loginPageUrl, { waitUntil: "domcontentloaded" });

    const loginIdSelector = await firstMatchingSelector(
      page,
      selectors.loginIdInput,
    );
    if (loginIdSelector === null) {
      return failure(
        page,
        "selectors_exhausted",
        "loginId",
        allSelectors(selectors.loginIdInput),
      );
    }
    await page.fill(loginIdSelector, options.loginId);

    const passwordSelector = await firstMatchingSelector(
      page,
      selectors.passwordInput,
    );
    if (passwordSelector === null) {
      return failure(
        page,
        "selectors_exhausted",
        "password",
        allSelectors(selectors.passwordInput),
      );
    }
    await page.fill(passwordSelector, options.password);

    const submitSelector = await firstMatchingSelector(
      page,
      selectors.submitButton,
    );
    if (submitSelector === null) {
      return failure(
        page,
        "selectors_exhausted",
        "submit",
        allSelectors(selectors.submitButton),
      );
    }
    await page.click(submitSelector);

    // Wait for the subsequent navigation or form-level update to settle.
    // The HTML mock server fixture stays on the same URL, so we poll on
    // domcontentloaded with a short timeout rather than requiring a full
    // URL change.
    await page
      .waitForLoadState("networkidle", { timeout: 10_000 })
      .catch(() => undefined);

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
