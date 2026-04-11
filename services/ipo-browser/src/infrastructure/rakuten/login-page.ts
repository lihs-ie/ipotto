import type { Page } from "playwright";

/**
 * Public contract for the login page object.
 */
export interface LoginPagePort {
  navigate(url: string): Promise<void>;
  enterLoginId(loginId: string): Promise<void>;
  enterPassword(password: string): Promise<void>;
  clickSubmit(): Promise<void>;
  isImageAuthenticationRequired(): Promise<boolean>;
  isLoginSuccessful(): Promise<boolean>;
  getErrorMessage(): Promise<string | null>;
}

/**
 * Page object for the Rakuten login page.
 */
export class LoginPage implements LoginPagePort {
  /**
   * Creates the page object.
   */
  public constructor(private readonly page: Page) {}

  /**
   * Navigates to the login page.
   */
  public async navigate(url: string): Promise<void> {
    await this.page.goto(url, { waitUntil: "domcontentloaded" });
  }

  /**
   * Enters the login id.
   */
  public async enterLoginId(loginId: string): Promise<void> {
    await this.page.locator("#loginid").fill(loginId);
  }

  /**
   * Enters the password.
   */
  public async enterPassword(password: string): Promise<void> {
    await this.page.locator("#passwd").fill(password);
  }

  /**
   * Submits the login form.
   */
  public async clickSubmit(): Promise<void> {
    await Promise.all([
      this.page.waitForLoadState("domcontentloaded"),
      this.page.locator('button[type="submit"]').click(),
    ]);
  }

  /**
   * Returns whether image authentication is required.
   */
  public async isImageAuthenticationRequired(): Promise<boolean> {
    if (
      (await this.page.locator('[data-page="image-authentication"]').count()) > 0
    ) {
      return true;
    }

    const title = await this.page.title();
    if (title.includes("多要素認証設定")) {
      return true;
    }

    return (
      (await this.page
        .getByRole("heading", {
          name: /ログイン追加認証|認証コード画像選択/u,
        })
        .count()) > 0
    );
  }

  /**
   * Returns whether login succeeded and the dashboard is visible.
   */
  public async isLoginSuccessful(): Promise<boolean> {
    if ((await this.page.locator('[data-page="dashboard"]').count()) > 0) {
      return true;
    }

    const title = await this.page.title();
    if (title.includes("ホーム | 楽天証券[PC]")) {
      return true;
    }

    return (
      (await this.page
        .locator(
          'a[href*="/app/home.do"], input#ratPageName[value*="[member]/app/home.do"]',
        )
        .count()) > 0
    );
  }

  /**
   * Returns the current error message if present.
   */
  public async getErrorMessage(): Promise<string | null> {
    const locator = this.page.locator('[data-role="error-message"]').first();
    if ((await locator.count()) === 0) {
      return null;
    }
    const text = await locator.textContent();
    return text === null ? null : text.trim();
  }
}
