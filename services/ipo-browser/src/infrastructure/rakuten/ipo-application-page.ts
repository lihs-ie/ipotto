import type { Page } from "playwright";

import type { ApplicationResult } from "../../domain/application-result.js";
import {
  createAlreadyAppliedResult,
  createInsufficientBalanceResult,
  createSuccessfulApplicationResult,
} from "../../domain/application-result.js";
import { detectApplicationResultFromText } from "./application-result-detector.js";

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
  }

  /**
   * Returns whether the page is ready for input.
   */
  public async isInputPage(): Promise<boolean> {
    return (
      (await this.page
        .locator("#orderValueInput, #orderValue, #priceSpinnerComBox, #passwordInputText")
        .count()) > 0
    );
  }

  /**
   * Enters the share count.
   */
  public async enterShares(shares: number): Promise<void> {
    const field = this.page.locator("#orderValueInput, #orderValue, #shares").first();
    await field.fill(String(shares));
  }

  /**
   * Enters the application price.
   */
  public async enterPrice(price: number): Promise<void> {
    const select = this.page.locator("#priceSpinnerComBox").first();
    if ((await select.count()) > 0) {
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

    const field = this.page.locator("#price").first();
    if ((await field.count()) > 0) {
      await field.fill(String(price));
    }
  }

  /**
   * Enters the trading password.
   */
  public async enterTradingPassword(password: string): Promise<void> {
    const field = this.page
      .locator("#passwordInputText, input[name=\"password\"], #trading-password")
      .first();
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
}
