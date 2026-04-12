import type { Page } from "playwright";

const LOGIN_ID_SELECTORS = [
  "#form-login-id",
  "#loginid",
  'input[name="loginid"]',
  'input[name="loginId"]',
  'input[name="user_id"]',
] as const;

const PASSWORD_SELECTORS = [
  "#form-login-pass",
  "#passwd",
  'input[name="passwd"]',
  'input[name="password"]',
  'input[type="password"]',
] as const;

const SUBMIT_SELECTORS = [
  "#login-btn",
  'button[type="submit"]',
  'input[type="submit"]',
  'button:has-text("ログイン")',
] as const;

const IMAGE_AUTHENTICATION_SELECTORS = [
  '[data-page="image-authentication"]',
  'input#ratPageName[value*="[member]/app/sotp_login.do"]',
] as const;

const DASHBOARD_SUCCESS_SELECTORS = [
  '[data-page="dashboard"]',
  'input#ratPageName[value*="[member]/app/home.do"]',
  'form[name="HomeForm"][action*="/app/home.do"]',
  'a.pcm-gl-nav-02__link[href*="/app/home.do"]',
  'a.pcmm-m1-home-header__qa[href*="/ITS/qaHome0001.html"]',
] as const;

const ERROR_MESSAGE_SELECTORS = [
  '[data-role="error-message"]',
  '[role="alert"]',
  ".pcm-form__error",
  ".pcmm-form__error",
] as const;

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
    await fillFirstAvailable(this.page, LOGIN_ID_SELECTORS, loginId);
  }

  /**
   * Enters the password.
   */
  public async enterPassword(password: string): Promise<void> {
    await fillFirstAvailable(this.page, PASSWORD_SELECTORS, password);
  }

  /**
   * Submits the login form.
   */
  public async clickSubmit(): Promise<void> {
    const submitSelector = await findFirstAvailableSelector(
      this.page,
      SUBMIT_SELECTORS,
    );

    await Promise.all([
      this.page.waitForLoadState("domcontentloaded"),
      this.page.locator(submitSelector).click(),
    ]);
  }

  /**
   * Returns whether image authentication is required.
   */
  public async isImageAuthenticationRequired(): Promise<boolean> {
    if (await hasAnyLocator(this.page, IMAGE_AUTHENTICATION_SELECTORS)) {
      return true;
    }

    const title = await this.page.title();
    if (
      title.includes("多要素認証設定") ||
      title.includes("ログイン認証 | 楽天証券[PC]")
    ) {
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
    if (await hasAnyLocator(this.page, DASHBOARD_SUCCESS_SELECTORS)) {
      return true;
    }

    const title = await this.page.title();
    if (
      title.includes("ホーム | 楽天証券[PC]") ||
      title.includes("トップ | ホーム | 楽天証券[PC]")
    ) {
      return true;
    }
    return false;
  }

  /**
   * Returns the current error message if present.
   */
  public async getErrorMessage(): Promise<string | null> {
    for (const selector of ERROR_MESSAGE_SELECTORS) {
      const locator = this.page.locator(selector).first();
      if ((await locator.count()) === 0) {
        continue;
      }
      const text = await locator.textContent();
      if (text !== null && text.trim() !== "") {
        return text.trim();
      }
    }
    return null;
  }
}

/**
 * Fills the first available selector from the candidate list.
 */
async function fillFirstAvailable(
  page: Page,
  selectors: readonly string[],
  value: string,
): Promise<void> {
  const selector = await findFirstAvailableSelector(page, selectors);
  await page.locator(selector).fill(value);
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
 * Finds the first available selector from the candidate list.
 */
async function findFirstAvailableSelector(
  page: Page,
  selectors: readonly string[],
): Promise<string> {
  for (const selector of selectors) {
    if ((await page.locator(selector).count()) > 0) {
      return selector;
    }
  }

  throw new Error(`required login selector was not found: ${selectors[0]}`);
}
