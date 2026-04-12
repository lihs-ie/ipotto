import type { Locator, Page } from "playwright";

import type { ApplicationResult } from "../../domain/application-result.js";
import {
  createAlreadyAppliedResult,
  createInsufficientBalanceResult,
  createSuccessfulApplicationResult,
} from "../../domain/application-result.js";
import { detectApplicationResultFromText } from "./application-result-detector.js";

const INPUT_PAGE_MARKERS = [
  "#orderValueInput",
  "#orderValue",
  "#priceSpinnerComBox",
  '[data-page="ipo-application"]',
  'input#ratPageName[value*="ipo_jp_join_input.do"]',
] as const;

const SHARE_INPUT_SELECTORS = [
  "#orderValueInput",
  "#orderValue",
  'input[name="orderValueInput"]',
  'input[name="orderValue"]',
  'input[placeholder*="株"]',
  "#shares",
] as const;

const PRICE_SELECT_SELECTORS = [
  "#priceSpinnerComBox",
  "select.pcmm-slb__input",
  'select[name="price"]',
] as const;

const PRICE_INPUT_SELECTORS = [
  "#price",
  'input[name="price"]',
] as const;

const TRADING_PASSWORD_SELECTORS = [
  "#passwordInputText",
  'input[name="password"]',
  "#trading-password",
] as const;

const CONFIRM_PAGE_MARKERS = [
  "#passwordInputText",
  '[data-page="ipo-confirm"]',
  'input#ratPageName[value*="ipo_jp_join_confirm.do"]',
] as const;

const RESULT_PAGE_MARKERS = [
  ".pcmm_ipolt-complete__txt",
  '[data-application-result="success"]',
  '[data-application-result="already-applied"]',
  '[data-application-result="insufficient-balance"]',
  'input#ratPageName[value*="ipo_jp_join_result.do"]',
] as const;

const INPUT_PAGE_TITLES = ["BB参加 / 受付", "楽天証券 IPO申し込み"] as const;
const CONFIRM_PAGE_TITLES = ["BB参加 / 確認"] as const;
const RESULT_PAGE_TITLES = ["BB参加 / 完了"] as const;

/**
 * Public contract for the application page.
 */
export interface IpoApplicationPagePort {
  navigate(url: string): Promise<void>;
  prepareForInput(): Promise<void>;
  isInputPage(): Promise<boolean>;
  enterShares(shares: number): Promise<void>;
  enterPrice(price: number): Promise<void>;
  enterTradingPassword(password: string): Promise<void>;
  clickConfirm(): Promise<void>;
  clickSubmit(): Promise<void>;
  readApplicationResult(): Promise<ApplicationResult>;
}

/**
 * Page object for IPO application.
 */
export class IpoApplicationPage implements IpoApplicationPagePort {
  /**
   * Creates the page object.
   */
  public constructor(private readonly page: Page) {}

  /**
   * Navigates to the application page.
   */
  public async navigate(url: string): Promise<void> {
    await this.page.goto(url, { waitUntil: "domcontentloaded" });
  }

  /**
   * Advances pre-input pages such as the caution page.
   */
  public async prepareForInput(): Promise<void> {
    const proceedButton = this.page
      .locator(
        'button:has-text("同意して次へ"), input[type="submit"][value="同意して次へ"]',
      )
      .first();

    if ((await proceedButton.count()) === 0) {
      return;
    }

    await Promise.all([
      this.page.waitForLoadState("domcontentloaded"),
      proceedButton.click(),
    ]);
    await this.waitForStage(INPUT_PAGE_MARKERS, INPUT_PAGE_TITLES);
  }

  /**
   * Returns whether the page is ready for input.
   */
  public async isInputPage(): Promise<boolean> {
    return (
      (await hasAnyLocator(this.page, SHARE_INPUT_SELECTORS)) ||
      (await hasAnyLocator(this.page, PRICE_SELECT_SELECTORS)) ||
      (await hasAnyLocator(this.page, TRADING_PASSWORD_SELECTORS))
    );
  }

  /**
   * Enters the share count.
   */
  public async enterShares(shares: number): Promise<void> {
    const field = await findFirstAvailableLocator(this.page, SHARE_INPUT_SELECTORS);
    await field.fill(String(shares));
  }

  /**
   * Enters the application price.
   */
  public async enterPrice(price: number): Promise<void> {
    const select = await findOptionalAvailableLocator(
      this.page,
      PRICE_SELECT_SELECTORS,
    );
    if (select !== null) {
      const value = String(price);
      const options = await select.locator("option").allTextContents();
      const hasMatchingOption = options.some((option) => option.includes(value));
      if (hasMatchingOption) {
        await select.selectOption({ value });
        return;
      }
      const hasMarketOrder = options.some((option) => option.includes("成行"));
      if (hasMarketOrder) {
        await select.selectOption({ label: "成行" });
        return;
      }
    }

    const field = await findOptionalAvailableLocator(this.page, PRICE_INPUT_SELECTORS);
    if (field !== null) {
      await field.fill(String(price));
      return;
    }

    throw new Error(`required application selector was not found: ${PRICE_SELECT_SELECTORS[0]}`);
  }

  /**
   * Enters the trading password.
   */
  public async enterTradingPassword(password: string): Promise<void> {
    const field = await findFirstAvailableLocator(
      this.page,
      TRADING_PASSWORD_SELECTORS,
    );
    await field.fill(password);
  }

  /**
   * Clicks the confirm button.
   */
  public async clickConfirm(): Promise<void> {
    const button = this.page
      .locator(
        'button:has-text("申込内容を確認する"), button#confirm-button, input[type="submit"][value="申込内容を確認する"]',
      )
      .first();
    await Promise.all([this.page.waitForLoadState("domcontentloaded"), button.click()]);
    await this.waitForStage(CONFIRM_PAGE_MARKERS, CONFIRM_PAGE_TITLES);
  }

  /**
   * Clicks the submit button.
   */
  public async clickSubmit(): Promise<void> {
    const button = this.page
      .locator(
        'button:has-text("申し込む"), button#submit-button, input[type="submit"][value="申し込む"]',
      )
      .first();
    await Promise.all([
      this.page.waitForLoadState("domcontentloaded"),
      button.click(),
    ]);
    await this.waitForStage(RESULT_PAGE_MARKERS, RESULT_PAGE_TITLES);
  }

  /**
   * Reads the resulting application status.
   */
  public async readApplicationResult(): Promise<ApplicationResult> {
    const pageText = await this.page.locator("body").textContent();
    const normalizedPageText = pageText === null ? "" : pageText;

    if ((await this.page.locator('[data-application-result="success"]').count()) > 0) {
      return createSuccessfulApplicationResult();
    }
    if (
      (await this.page
        .locator(".pcmm_ipolt-complete__txt")
        .filter({ hasText: "申込を受け付けました" })
        .count()) > 0
    ) {
      return createSuccessfulApplicationResult();
    }
    if (
      (await this.page
        .locator('[data-application-result="already-applied"]')
        .count()) > 0
    ) {
      return createAlreadyAppliedResult();
    }
    if (
      normalizedPageText.includes("ブックビルディング申込済") ||
      normalizedPageText.includes("申込詳細")
    ) {
      return createAlreadyAppliedResult();
    }
    if (
      (await this.page
        .locator('[data-application-result="insufficient-balance"]')
        .count()) > 0
    ) {
      return createInsufficientBalanceResult();
    }
    if (
      normalizedPageText.includes("買付可能額が不足") ||
      normalizedPageText.includes("不足金") ||
      normalizedPageText.includes("残高不足")
    ) {
      return createInsufficientBalanceResult();
    }

    const errorText = await this.readErrorMessage();
    return detectApplicationResultFromText(
      normalizedPageText,
      errorText === null ? "application failed" : errorText.trim(),
    );
  }

  /**
   * Reads a visible error message from the page.
   */
  private async readErrorMessage(): Promise<string | null> {
    const candidates = [
      this.page.locator(".pcmm-art__hdg"),
      this.page.locator(".pcmm-art__txt"),
      this.page.locator(".pcmm_ipolt-err-msg"),
      this.page.locator('[data-role="error-message"]'),
    ];

    for (const candidate of candidates) {
      const locator = candidate.first();
      if ((await locator.count()) === 0) {
        continue;
      }
      const text = await locator.textContent();
      if (text === null) {
        continue;
      }
      const normalized = text.replace(/\s+/g, " ").trim();
      if (normalized !== "") {
        return normalized;
      }
    }

    return null;
  }

  /**
   * Waits until the page reaches the expected application stage.
   */
  private async waitForStage(
    selectors: readonly string[],
    titleIncludes: readonly string[],
  ): Promise<void> {
    if (await this.hasStageMarker(selectors, titleIncludes)) {
      return;
    }

    await this.page.waitForFunction(
      ({ selectors: candidateSelectors, titleIncludes: candidateTitles }) => {
        const title = document.title;
        if (
          candidateTitles.some((candidateTitle) =>
            title.includes(candidateTitle),
          )
        ) {
          return true;
        }

        return candidateSelectors.some(
          (candidateSelector) =>
            document.querySelector(candidateSelector) !== null,
        );
      },
      {
        selectors,
        titleIncludes,
      },
    );
  }

  /**
   * Returns whether the current document already matches the expected stage.
   */
  private async hasStageMarker(
    selectors: readonly string[],
    titleIncludes: readonly string[],
  ): Promise<boolean> {
    const title = await this.page.title();
    if (
      titleIncludes.some((candidateTitle) => title.includes(candidateTitle))
    ) {
      return true;
    }

    for (const selector of selectors) {
      if ((await this.page.locator(selector).count()) > 0) {
        return true;
      }
    }

    return false;
  }
}

/**
 * Returns whether any selector from the candidate list exists.
 */
async function hasAnyLocator(
  page: Page,
  selectors: readonly string[],
): Promise<boolean> {
  for (const selector of selectors) {
    if ((await page.locator(selector).count()) > 0) {
      return true;
    }
  }

  return false;
}

/**
 * Finds the first available locator from the candidate list.
 */
async function findFirstAvailableLocator(
  page: Page,
  selectors: readonly string[],
): Promise<Locator> {
  const locator = await findOptionalAvailableLocator(page, selectors);
  if (locator !== null) {
    return locator;
  }

  throw new Error(`required application selector was not found: ${selectors[0]}`);
}

/**
 * Finds the first available locator from the candidate list when present.
 */
async function findOptionalAvailableLocator(
  page: Page,
  selectors: readonly string[],
): Promise<Locator | null> {
  for (const selector of selectors) {
    const locator = page.locator(selector).first();
    if ((await locator.count()) > 0) {
      return locator;
    }
  }

  return null;
}
