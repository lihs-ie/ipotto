import { promises as fs } from "node:fs";
import { join } from "node:path";

import type { Page } from "playwright";

import { logger } from "../logger.js";

import {
  allSelectors,
  loadSelectors,
  type SelectorDefinition,
} from "../config/selectors.js";

import { translateApplyResultText } from "./apply-result-translator.js";

// Phase 3 Sprint 7 — Rakuten IPO apply flow. Navigates the IPO apply
// list, identifies the target stock row by `data-stock-identifier`,
// fills the apply form with shares / price / trading password, submits,
// and translates the result page into an `ApplyResult`.
//
// The keyword table used by translateApplyResultText mirrors
// `translate_application_result` in
// `services/ipo-backend-shared/src/acl/browser/application_result.rs`
// so the TS and Rust sides agree on semantics for every result page
// reachable on the mock server and on the real broker site.

export type ApplyOptions = {
  readonly stockIdentifier: string;
  readonly companyName: string;
  readonly shares: number;
  readonly price: number;
  readonly tradingPassword: string;
  readonly applyListUrl: string;
};

export type ApplyResult =
  | { readonly status: "success" }
  | {
      readonly status: "failure";
      readonly reason: string;
      readonly screenshotPath: string | null;
    }
  | { readonly status: "already_applied" }
  | { readonly status: "insufficient_balance" };

const SCREENSHOT_DIR =
  process.env["BROWSER_SCREENSHOT_DIR"] ?? "/tmp/ipo-browser-screenshots";
const NAVIGATION_TIMEOUT_MS = 10_000;
const RESULT_INDICATOR_TIMEOUT_MS = 10_000;

export async function rakutenApply(
  page: Page,
  options: ApplyOptions,
): Promise<ApplyResult> {
  const selectors = loadSelectors().rakuten;

  try {
    await page.goto(options.applyListUrl, {
      waitUntil: "domcontentloaded",
      timeout: NAVIGATION_TIMEOUT_MS,
    });

    const stockRowSelector = await firstMatchingSelector(
      page,
      selectors.applyList.stockRow,
      `[data-stock-identifier="${options.stockIdentifier}"]`,
    );
    if (stockRowSelector === null) {
      return await failure(
        page,
        "stock_not_found_in_apply_list",
        `no row with data-stock-identifier=${options.stockIdentifier}`,
      );
    }

    const applyButton = page
      .locator(stockRowSelector)
      .locator(allSelectors(selectors.applyList.applyButton).join(", "))
      .first();
    const applyButtonVisible = await applyButton
      .waitFor({ state: "visible", timeout: NAVIGATION_TIMEOUT_MS })
      .then(() => true)
      .catch(() => false);
    if (!applyButtonVisible) {
      return await failure(
        page,
        "apply_button_not_found",
        `apply button missing on stock row ${options.stockIdentifier}`,
      );
    }

    await Promise.all([
      page
        .waitForLoadState("domcontentloaded", { timeout: NAVIGATION_TIMEOUT_MS })
        .catch(() => undefined),
      applyButton.click(),
    ]);

    const sharesSelector = await firstMatchingSelector(
      page,
      selectors.applyForm.sharesInput,
    );
    if (sharesSelector === null) {
      return await failure(
        page,
        "shares_input_not_found",
        `tried ${allSelectors(selectors.applyForm.sharesInput).join(", ")}`,
      );
    }
    await page.fill(sharesSelector, String(options.shares));

    const priceSelector = await firstMatchingSelector(
      page,
      selectors.applyForm.priceInput,
    );
    if (priceSelector === null) {
      return await failure(
        page,
        "price_input_not_found",
        `tried ${allSelectors(selectors.applyForm.priceInput).join(", ")}`,
      );
    }
    await page.fill(priceSelector, String(options.price));

    const tradingPasswordSelector = await firstMatchingSelector(
      page,
      selectors.applyForm.tradingPasswordInput,
    );
    if (tradingPasswordSelector !== null) {
      await page.fill(tradingPasswordSelector, options.tradingPassword);
    }

    const submitSelector = await firstMatchingSelector(
      page,
      selectors.applyForm.submitButton,
    );
    if (submitSelector === null) {
      return await failure(
        page,
        "submit_button_not_found",
        `tried ${allSelectors(selectors.applyForm.submitButton).join(", ")}`,
      );
    }
    await Promise.all([
      page
        .waitForLoadState("domcontentloaded", { timeout: NAVIGATION_TIMEOUT_MS })
        .catch(() => undefined),
      page.click(submitSelector),
    ]);

    await page
      .waitForLoadState("networkidle", { timeout: RESULT_INDICATOR_TIMEOUT_MS })
      .catch(() => undefined);

    return await classifyResultPage(page, selectors.applyResult);
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    logger.error({ event: "flows.apply.exception", error: reason });
    const screenshotPath = await tryScreenshot(page, "apply-exception");
    return { status: "failure", reason, screenshotPath };
  }
}

async function classifyResultPage(
  page: Page,
  resultSelectors: {
    readonly successIndicator: SelectorDefinition;
    readonly duplicateIndicator: SelectorDefinition;
    readonly insufficientBalanceIndicator: SelectorDefinition;
    readonly failureIndicator: SelectorDefinition;
  },
): Promise<ApplyResult> {
  const successSelector = await firstVisibleSelector(
    page,
    resultSelectors.successIndicator,
  );
  if (successSelector !== null) {
    return { status: "success" };
  }
  const duplicateSelector = await firstVisibleSelector(
    page,
    resultSelectors.duplicateIndicator,
  );
  if (duplicateSelector !== null) {
    return { status: "already_applied" };
  }
  const insufficientSelector = await firstVisibleSelector(
    page,
    resultSelectors.insufficientBalanceIndicator,
  );
  if (insufficientSelector !== null) {
    return { status: "insufficient_balance" };
  }
  const failureSelector = await firstVisibleSelector(
    page,
    resultSelectors.failureIndicator,
  );
  if (failureSelector !== null) {
    const message = await page
      .locator(failureSelector)
      .first()
      .innerText()
      .catch(() => "");
    const translated = translateApplyResultText(message);
    if (translated.status === "success") {
      return { status: "success" };
    }
    if (translated.status === "already_applied") {
      return { status: "already_applied" };
    }
    if (translated.status === "insufficient_balance") {
      return { status: "insufficient_balance" };
    }
    const screenshotPath = await tryScreenshot(page, "apply-failure");
    return {
      status: "failure",
      reason: translated.reason,
      screenshotPath,
    };
  }

  const bodyText = await page.innerText("body").catch(() => "");
  const translated = translateApplyResultText(bodyText);
  if (translated.status === "success") {
    return { status: "success" };
  }
  if (translated.status === "already_applied") {
    return { status: "already_applied" };
  }
  if (translated.status === "insufficient_balance") {
    return { status: "insufficient_balance" };
  }
  const screenshotPath = await tryScreenshot(page, "apply-unrecognised");
  return {
    status: "failure",
    reason: "unrecognised_result_page",
    screenshotPath,
  };
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

async function firstVisibleSelector(
  page: Page,
  definition: SelectorDefinition,
): Promise<string | null> {
  for (const selector of allSelectors(definition)) {
    const visible = await page
      .locator(selector)
      .first()
      .isVisible()
      .catch(() => false);
    if (visible) {
      return selector;
    }
  }
  return null;
}

async function failure(
  page: Page,
  code: string,
  message: string,
): Promise<ApplyResult> {
  const screenshotPath = await tryScreenshot(page, `apply-${code}`);
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
