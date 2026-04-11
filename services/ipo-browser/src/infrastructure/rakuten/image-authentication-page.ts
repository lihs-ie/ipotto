import type { Page } from "playwright";

import {
  buildImageAuthenticationChoice,
  findMatchingImageAuthenticationChoice,
} from "./image-authentication-choice.js";

/**
 * Public contract for image authentication.
 */
export interface ImageAuthenticationPagePort {
  getImageButtonAltTexts(): Promise<readonly string[]>;
  clickImageByAltText(altText: string): Promise<void>;
  submitSelection(): Promise<void>;
  isAuthenticationSuccessful(): Promise<boolean>;
  getErrorMessage(): Promise<string | null>;
  requiresCodeResend(): Promise<boolean>;
  resendCode(): Promise<void>;
}

/**
 * Page object for image authentication.
 */
export class ImageAuthenticationPage implements ImageAuthenticationPagePort {
  /**
   * Creates the page object.
   */
  public constructor(private readonly page: Page) {}

  /**
   * Returns all resolvable image labels on the page.
   */
  public async getImageButtonAltTexts(): Promise<readonly string[]> {
    const choices = await this.readChoices();
    return choices.flatMap((choice) => choice.labels.slice(0, 1));
  }

  /**
   * Clicks an image button by keyword or derived image label.
   */
  public async clickImageByAltText(altText: string): Promise<void> {
    const choices = await this.readChoices();
    const matchedChoice = findMatchingImageAuthenticationChoice(
      choices,
      altText,
    );
    if (matchedChoice === null) {
      const availableLabels = choices.flatMap((choice) => choice.labels);
      throw new Error(
        `image authentication keyword not found: ${altText}; available labels: ${availableLabels.join(", ")}`,
      );
    }

    if (matchedChoice.buttonId !== null) {
      await this.page.locator(`#${matchedChoice.buttonId}`).click();
      return;
    }

    await this.imageButtons().nth(matchedChoice.buttonIndex).click();
  }

  /**
   * Submits the selected images when the real page requires it.
   */
  public async submitSelection(): Promise<void> {
    const submitButton = this.page
      .locator(
        'input[type="button"][value="設定する"], input.pcmm_emoji-btn__primary[value="設定する"], [data-auth-submit]',
      )
      .first();
    if ((await submitButton.count()) === 0) {
      return;
    }

    await Promise.all([
      this.page.waitForLoadState("domcontentloaded"),
      submitButton.click(),
    ]);
  }

  /**
   * Returns whether authentication succeeded.
   */
  public async isAuthenticationSuccessful(): Promise<boolean> {
    await this.page.waitForLoadState("domcontentloaded");
    if ((await this.page.locator('[data-page="dashboard"]').count()) > 0) {
      return true;
    }

    if ((await this.imageButtons().count()) > 0) {
      return false;
    }

    return (await this.getErrorMessage()) === null;
  }

  /**
   * Returns the error message if authentication failed.
   */
  public async getErrorMessage(): Promise<string | null> {
    const candidates = [
      this.page.locator(".pcmm_emoji-art.err_msg:visible .pcmm_emoji-art__txt"),
      this.page.locator(
        ".pcmm_emoji-art.pcmm_emoji--is-error:visible .pcmm_emoji-art__txt",
      ),
      this.page.locator(".pcmm_emoji-error.error_filed.error_valid:visible"),
      this.page.locator('[data-role="error-message"]:visible'),
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
   * Returns whether the current page requires a new authentication code.
   */
  public async requiresCodeResend(): Promise<boolean> {
    const errorMessage = await this.getErrorMessage();
    if (
      errorMessage !== null &&
      (errorMessage.includes("失効") || errorMessage.includes("再送信"))
    ) {
      return true;
    }

    return (
      (await this.page
        .locator(
          'input[type="button"][value="認証コードを再送信する"], button:has-text("認証コードを再送信する")',
        )
        .count()) > 0
    );
  }

  /**
   * Requests a new authentication code from the current page.
   */
  public async resendCode(): Promise<void> {
    const resendButton = this.page
      .locator(
        'input[type="button"][value="認証コードを再送信する"], button:has-text("認証コードを再送信する")',
      )
      .first();
    if ((await resendButton.count()) === 0) {
      throw new Error("authentication code resend button not found");
    }

    await Promise.all([
      this.page.waitForLoadState("domcontentloaded"),
      resendButton.click(),
    ]);
  }

  /**
   * Returns all image buttons used for authentication.
   */
  private imageButtons() {
    return this.page.locator(
      '.pcmm_emoji-img__block button.pcmm_emoji-img, [data-auth-image]',
    );
  }

  /**
   * Reads all image choices from the current page.
   */
  private async readChoices() {
    return this.imageButtons().evaluateAll((elements) =>
      elements.map((element, index) => {
        const button = element instanceof HTMLButtonElement ? element : null;
        const image = element.querySelector("img");
        return {
          buttonId: button?.id ?? null,
          buttonIndex: index,
          onclick: button?.getAttribute("onclick") ?? null,
          imageAlt: image?.getAttribute("alt") ?? null,
          imageSrc: image?.getAttribute("src") ?? null,
        };
      }),
    ).then((choices) =>
      choices.map((choice) => buildImageAuthenticationChoice(choice)),
    );
  }
}
