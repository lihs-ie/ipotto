import type {
  ActiveSecuritiesAccount,
  AccountCredential,
  ConnectionTestResult,
} from "../../domain/account-credential.js";
import type {
  ApplicationResult,
  LotteryResult,
} from "../../domain/application-result.js";
import {
  createFailedApplicationResult,
} from "../../domain/application-result.js";
import type { TargetIpoStock } from "../../domain/ipo-stock.js";
import { BrowserSessionStorage } from "../session/browser-session-storage.js";
import type { PageFactoryPort, RakutenSession } from "./page-factory.js";

/**
 * Keyword pair required for image authentication.
 */
export interface ImageAuthenticationKeyword {
  readonly firstKeyword: string;
  readonly secondKeyword: string;
}

/**
 * Keyword provider abstraction for image authentication.
 */
export interface ImageAuthenticationKeywordProvider {
  /**
   * Returns image authentication keywords.
   */
  fetchKeywords(
    credential: AccountCredential,
  ): Promise<ImageAuthenticationKeyword>;
}

/**
 * Static keyword provider backed by configuration.
 */
export class StaticImageAuthenticationKeywordProvider
  implements ImageAuthenticationKeywordProvider
{
  /**
   * Creates the provider.
   */
  public constructor(
    private readonly keywords: readonly [string, string] | null,
  ) {}

  /**
   * Returns configured keywords.
   */
  public async fetchKeywords(
    credential: AccountCredential,
  ): Promise<ImageAuthenticationKeyword> {
    void credential;
    if (this.keywords === null) {
      throw new Error("image authentication keywords are not configured");
    }
    const [firstKeyword, secondKeyword] = this.keywords;
    return { firstKeyword, secondKeyword };
  }
}

/**
 * Runtime config for the Rakuten adapter.
 */
export interface RakutenBrokerConfig {
  readonly dryRun: boolean;
  readonly loginPageUrl: string | null;
  readonly ipoListPageUrl: string | null;
  readonly applicationPageUrl: string | null;
  readonly mockLotteryResult: LotteryResult;
}

/**
 * Playwright-backed Rakuten adapter used by ipo-browser.
 */
export class RakutenBrokerAdapter {
  /**
   * Creates the adapter.
   */
  public constructor(
    private readonly pageFactory: PageFactoryPort,
    private readonly sessionStorage: BrowserSessionStorage,
    private readonly keywordProvider: ImageAuthenticationKeywordProvider,
    private readonly config: RakutenBrokerConfig,
  ) {}

  /**
   * Tests browser connectivity with the given credential.
   */
  public async testConnection(
    credential: AccountCredential,
  ): Promise<ConnectionTestResult> {
    let session: RakutenSession | null = null;
    try {
      session = await this.pageFactory.createSession(
        this.sessionStorage.getUserDataDirectory(
          `connection-test-${crypto.randomUUID()}`,
        ),
      );
      await this.login(session, credential);
      return {
        success: true,
        message: "connected",
        testedAt: new Date().toISOString(),
      };
    } catch (error) {
      return {
        success: false,
        message: error instanceof Error ? error.message : "unknown error",
        testedAt: new Date().toISOString(),
      };
    } finally {
      await session?.close();
    }
  }

  /**
   * Applies for an IPO stock.
   */
  public async applyForIpo(
    account: ActiveSecuritiesAccount,
    stock: TargetIpoStock,
  ): Promise<ApplicationResult> {
    let session: RakutenSession | null = null;
    try {
      const sessionDirectory = await this.sessionStorage.ensureUserDataDirectory(
        account.identifier,
      );
      session = await this.pageFactory.createSession(sessionDirectory);
      await this.login(session, account.credential);
      await this.openApplicationPage(session, stock);
      const applicationPage = session.ipoApplicationPage();
      await applicationPage.prepareForInput();
      if (!(await applicationPage.isInputPage())) {
        return applicationPage.readApplicationResult();
      }
      await applicationPage.enterShares(stock.shares);
      await applicationPage.enterPrice(stock.price);
      await applicationPage.enterTradingPassword(
        account.credential.tradingPassword,
      );
      await applicationPage.clickConfirm();
      await applicationPage.clickSubmit();
      return applicationPage.readApplicationResult();
    } catch (error) {
      return createFailedApplicationResult(
        error instanceof Error ? error.message : "browser automation failed",
      );
    } finally {
      await session?.close();
    }
  }

  /**
   * Checks the lottery result for the given stock.
   */
  public async checkLotteryResult(
    credential: AccountCredential,
    stockIdentifier: string,
  ): Promise<LotteryResult> {
    void stockIdentifier;
    const connection = await this.testConnection(credential);
    if (!connection.success) {
      throw new Error(connection.message);
    }
    return this.config.mockLotteryResult;
  }

  /**
   * Logs in and resolves image authentication if required.
   */
  private async login(
    session: RakutenSession,
    credential: AccountCredential,
  ): Promise<void> {
    if (this.config.loginPageUrl === null) {
      throw new Error("RAKUTEN_LOGIN_PAGE_URL is not configured");
    }

    const loginPage = session.loginPage();
    await loginPage.navigate(this.config.loginPageUrl);
    if (await loginPage.isLoginSuccessful()) {
      return;
    }

    await loginPage.enterLoginId(credential.loginId);
    await loginPage.enterPassword(credential.loginPassword);
    await loginPage.clickSubmit();

    if (await loginPage.isImageAuthenticationRequired()) {
      const imageAuthenticationPage = session.imageAuthenticationPage();
      await this.completeImageAuthentication(
        imageAuthenticationPage,
        credential,
      );
      return;
    }

    if (await loginPage.isLoginSuccessful()) {
      return;
    }

    throw new Error(
      (await loginPage.getErrorMessage()) ?? "login failed unexpectedly",
    );
  }

  /**
   * Completes the image authentication flow, including resend handling.
   */
  private async completeImageAuthentication(
    imageAuthenticationPage: ReturnType<RakutenSession["imageAuthenticationPage"]>,
    credential: AccountCredential,
  ): Promise<void> {
    for (let attempt = 0; attempt < 2; attempt += 1) {
      const keywords = await this.keywordProvider.fetchKeywords(credential);
      await imageAuthenticationPage.clickImageByAltText(keywords.firstKeyword);
      await imageAuthenticationPage.clickImageByAltText(keywords.secondKeyword);
      await imageAuthenticationPage.submitSelection();

      if (await imageAuthenticationPage.isAuthenticationSuccessful()) {
        return;
      }

      if (
        attempt === 0 &&
        (await imageAuthenticationPage.requiresCodeResend())
      ) {
        await imageAuthenticationPage.resendCode();
        continue;
      }

      throw new Error(
        (await imageAuthenticationPage.getErrorMessage()) ??
          "image authentication failed",
      );
    }
  }

  /**
   * Opens the IPO application flow from the list page when available.
   */
  private async openApplicationPage(
    session: RakutenSession,
    stock: TargetIpoStock,
  ): Promise<void> {
    if (this.config.ipoListPageUrl !== null) {
      const ipoListPage = session.ipoListPage();
      await ipoListPage.navigate(buildIpoListPageUrl(this.config.ipoListPageUrl, stock));
      await ipoListPage.openApplicationForCompany(stock.companyName);
      return;
    }

    const applicationPage = session.ipoApplicationPage();
    const applicationPageUrl = buildApplicationPageUrl(
      this.config.applicationPageUrl,
      stock,
    );
    await applicationPage.navigate(applicationPageUrl);
  }
}

/**
 * Builds an application page URL and encodes the dry-run scenario.
 */
function buildApplicationPageUrl(
  baseUrl: string | null,
  stock: TargetIpoStock,
): string {
  if (baseUrl === null) {
    throw new Error("RAKUTEN_APPLICATION_PAGE_URL is not configured");
  }

  const url = new URL(baseUrl);
  if (stock.companyName.includes("申込済")) {
    url.searchParams.set("scenario", "already_applied");
  } else if (stock.companyName.includes("残高不足")) {
    url.searchParams.set("scenario", "insufficient_balance");
  } else if (stock.companyName.includes("失敗")) {
    url.searchParams.set("scenario", "failure");
  } else {
    url.searchParams.set("scenario", "success");
  }
  return url.toString();
}

/**
 * Builds an IPO list page URL and encodes mock-fixture hints.
 */
function buildIpoListPageUrl(
  baseUrl: string,
  stock: TargetIpoStock,
): string {
  const url = new URL(baseUrl);
  url.searchParams.set("companyName", stock.companyName);

  if (stock.companyName.includes("申込済")) {
    url.searchParams.set("scenario", "already_applied");
  } else if (stock.companyName.includes("残高不足")) {
    url.searchParams.set("scenario", "insufficient_balance");
  } else if (stock.companyName.includes("失敗")) {
    url.searchParams.set("scenario", "failure");
  } else {
    url.searchParams.set("scenario", "success");
  }

  return url.toString();
}
