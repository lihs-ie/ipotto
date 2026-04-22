import { promises as fs } from "node:fs";
import { join } from "node:path";

import type { Page } from "playwright";

import {
  allSelectors,
  loadSelectors,
  type SelectorDefinition,
} from "../config/selectors.js";
import { logger } from "../logger.js";

import {
  translateLotteryResultText,
  type LotteryOutcome,
} from "./lottery-result-translator.js";

// Phase 3 Sprint 7 — Rakuten lottery result scraping flow. Navigates
// the lottery result page, locates the row by `data-stock-identifier`,
// reads `.result-label`, and translates the text into the
// `LotteryResult` enum (or null when the result is not yet published).
// Mirror of `translate_lottery_result` in `docs/03-detailed-design/acl.md` §3.5.

export type CheckResultOptions = {
  readonly stockIdentifier: string;
  readonly resultPageUrl: string;
};

export type CheckResultOutcome =
  | { readonly status: "resolved"; readonly result: LotteryOutcome }
  | {
      readonly status: "failure";
      readonly reason: string;
      readonly screenshotPath: string | null;
    };

const SCREENSHOT_DIR =
  process.env["BROWSER_SCREENSHOT_DIR"] ?? "/tmp/ipo-browser-screenshots";
const NAVIGATION_TIMEOUT_MS = 10_000;

export async function rakutenCheckResult(
  page: Page,
  options: CheckResultOptions,
): Promise<CheckResultOutcome> {
  const selectors = loadSelectors().rakuten.lotteryResult;

  try {
    await page.goto(options.resultPageUrl, {
      waitUntil: "domcontentloaded",
      timeout: NAVIGATION_TIMEOUT_MS,
    });

    const containerSelector = await firstMatchingSelector(page, selectors.resultContainer);
    if (containerSelector === null) {
      return await failure(
        page,
        "result_container_not_found",
        `tried ${allSelectors(selectors.resultContainer).join(", ")}`,
      );
    }

    const rowSelector = await firstMatchingSelector(
      page,
      selectors.resultRow,
      `[data-stock-identifier="${options.stockIdentifier}"]`,
    );
    if (rowSelector === null) {
      // No row for this stock yet — treat as not-yet-published rather than failure
      // so ipo-result-checker keeps polling until the broker publishes the row.
      return { status: "resolved", result: null };
    }

    const labelLocator = page
      .locator(rowSelector)
      .locator(allSelectors(selectors.resultLabel).join(", "))
      .first();

    const labelVisible = await labelLocator
      .waitFor({ state: "attached", timeout: NAVIGATION_TIMEOUT_MS })
      .then(() => true)
      .catch(() => false);
    if (!labelVisible) {
      return { status: "resolved", result: null };
    }

    const rawText = await labelLocator.innerText().catch(() => "");
    const translated = translateLotteryResultText(rawText);
    return { status: "resolved", result: translated };
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    logger.error({
      event: "flows.check_result.exception",
      error: error instanceof Error ? error.message : String(error),
    });
    const screenshotPath = await tryScreenshot(page, "check-result-exception");
    return { status: "failure", reason, screenshotPath };
  }
}

async function firstMatchingSelector(
  page: Page,
  definition: SelectorDefinition,
  refinement?: string,
): Promise<string | null> {
  for (const selector of allSelectors(definition)) {
    const resolved = refinement === undefined ? selector : `${selector}${refinement}`;
    const count = await page
      .locator(resolved)
      .count()
      .catch(() => 0);
    if (count > 0) {
      return resolved;
    }
  }
  return null;
}

async function failure(
  page: Page,
  code: string,
  message: string,
): Promise<CheckResultOutcome> {
  const screenshotPath = await tryScreenshot(page, `check-result-${code}`);
  return {
    status: "failure",
    reason: `${code}: ${message}`,
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
