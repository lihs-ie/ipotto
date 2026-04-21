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
import {
  MailRetrievalTimeoutError,
  RakutenAuthMailParseError,
  RakutenAuthMailSourceError,
} from "../mail/rakuten-auth-mail-parser.js";
import { BrowserSessionStorage } from "../session/browser-session-storage.js";
import type { PageFactoryPort, RakutenSession } from "./page-factory.js";
import type {
  RakutenNavigationTargetResolver,
} from "./rakuten-navigation-target-resolver.js";

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
  readonly mockLotteryResult: LotteryResult;
}

class ImageAuthenticationError extends Error {
  /**
   * Creates the image authentication error.
   */
  public constructor(message: string) {
    super(message);
    this.name = "ImageAuthenticationError";
  }
}

class MailAuthenticationRetrievalError extends Error {
  /**
   * Creates the mail retrieval error used during image authentication.
   */
  public constructor(message: string) {
    super(message);
    this.name = "MailAuthenticationRetrievalError";
  }
}

const IMAGE_AUTHENTICATION_MAX_ATTEMPTS = 2;

/**
 * Retry policy for Rakuten browser automation.
 *
 * - image authentication code expiry: retry once after resending the code
 * - mail retrieval timeout/source errors: do not retry
 * - selector missing and page transition errors: do not retry
 * - temporary broker-side errors: do not retry until a stable transient marker is identified
 */
const RAKUTEN_RETRY_POLICY = {
  imageAuthenticationMaxAttempts: IMAGE_AUTHENTICATION_MAX_ATTEMPTS,
} as const;

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
    private readonly navigationTargetResolver: RakutenNavigationTargetResolver,
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
      const result = await applicationPage.readApplicationResult();
      return result;
    } catch (error) {
      return mapRakutenAutomationError(error);
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
    const loginPage = session.loginPage();
    await loginPage.navigate(this.navigationTargetResolver.resolveLoginPageUrl());
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
    for (
      let attempt = 0;
      attempt < RAKUTEN_RETRY_POLICY.imageAuthenticationMaxAttempts;
      attempt += 1
    ) {
      const keywords =
        await this.fetchImageAuthenticationKeywords(credential);
      try {
        await imageAuthenticationPage.clickImageByAltText(keywords.firstKeyword);
        await imageAuthenticationPage.clickImageByAltText(keywords.secondKeyword);
        await imageAuthenticationPage.submitSelection();
      } catch (error) {
        throw new ImageAuthenticationError(
          error instanceof Error
            ? error.message
            : "image authentication failed",
        );
      }

      if (await imageAuthenticationPage.isAuthenticationSuccessful()) {
        return;
      }

      if (
        attempt === 0 &&
        (await imageAuthenticationPage.requiresCodeResend())
      ) {
        try {
          await imageAuthenticationPage.resendCode();
        } catch (error) {
          throw new ImageAuthenticationError(
            error instanceof Error
              ? error.message
              : "image authentication resend failed",
          );
        }
        continue;
      }

      throw new ImageAuthenticationError(
        (await imageAuthenticationPage.getErrorMessage()) ??
          "image authentication failed",
      );
    }
  }

  /**
   * Fetches image authentication keywords and classifies mail failures.
   */
  private async fetchImageAuthenticationKeywords(
    credential: AccountCredential,
  ): Promise<ImageAuthenticationKeyword> {
    try {
      return await this.keywordProvider.fetchKeywords(credential);
    } catch (error) {
      if (
        error instanceof MailRetrievalTimeoutError ||
        error instanceof RakutenAuthMailParseError ||
        error instanceof RakutenAuthMailSourceError
      ) {
        throw new MailAuthenticationRetrievalError(error.message);
      }

      throw error;
    }
  }

  /**
   * Opens the IPO application flow from the list page when available.
   */
  private async openApplicationPage(
    session: RakutenSession,
    stock: TargetIpoStock,
  ): Promise<void> {
    if (this.navigationTargetResolver.hasIpoListPageTarget()) {
      const ipoListPage = session.ipoListPage();
      await ipoListPage.navigate(
        this.navigationTargetResolver.resolveIpoListPageUrl(stock),
      );
      await ipoListPage.openApplicationForCompany(stock.companyName);
      return;
    }

    const applicationPage = session.ipoApplicationPage();
    await applicationPage.navigate(
      this.navigationTargetResolver.resolveApplicationPageUrl(stock),
    );
  }
}

/**
 * Maps broker automation errors to the shared application result.
 *
 * `application` covers browser-side failures that are not specific to mail
 * retrieval or image authentication, including login failures, 2FA page
 * transition failures, selector mismatches, and unexpected navigation.
 */
function mapRakutenAutomationError(error: unknown): ApplicationResult {
  const message =
    error instanceof Error ? error.message : "browser automation failed";

  if (error instanceof MailAuthenticationRetrievalError) {
    return createFailedApplicationResult(message, "mail_retrieval");
  }

  if (error instanceof ImageAuthenticationError) {
    return createFailedApplicationResult(message, "image_authentication");
  }

  return createFailedApplicationResult(message, "application");
}
