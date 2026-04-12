import { describe, expect, it } from "vitest";

import type { ApplicationResult } from "../../domain/application-result.js";
import {
  MailRetrievalTimeoutError,
} from "../mail/rakuten-auth-mail-parser.js";
import { BrowserSessionStorage } from "../session/browser-session-storage.js";
import {
  FixtureRakutenNavigationTargetResolver,
  ProductionRakutenNavigationTargetResolver,
} from "./rakuten-navigation-target-resolver.js";
import {
  type ImageAuthenticationKeywordProvider,
  RakutenBrokerAdapter,
} from "./rakuten-broker-adapter.js";
import type { PageFactoryPort, RakutenSession } from "./page-factory.js";

class FakeLoginPage {
  public imageAuthenticationRequired = false;
  public loginSuccessful = false;
  public errorMessage: string | null = null;
  public submitSucceeds = true;

  public async navigate(): Promise<void> {}
  public async enterLoginId(): Promise<void> {}
  public async enterPassword(): Promise<void> {}
  public async clickSubmit(): Promise<void> {
    if (!this.imageAuthenticationRequired && this.submitSucceeds) {
      this.loginSuccessful = true;
    }
  }
  public async isImageAuthenticationRequired(): Promise<boolean> {
    return this.imageAuthenticationRequired;
  }
  public async isLoginSuccessful(): Promise<boolean> {
    return this.loginSuccessful;
  }
  public async getErrorMessage(): Promise<string | null> {
    return this.errorMessage;
  }
}

class FakeImageAuthenticationPage {
  public succeeded = true;
  public resendRequired = false;
  public resendCount = 0;

  public async getImageButtonAltTexts(): Promise<readonly string[]> {
    return ["さくら", "みかん"];
  }
  public async clickImageByAltText(): Promise<void> {}
  public async submitSelection(): Promise<void> {}
  public async isAuthenticationSuccessful(): Promise<boolean> {
    return this.succeeded;
  }
  public async getErrorMessage(): Promise<string | null> {
    if (this.succeeded) {
      return null;
    }
    if (this.resendRequired) {
      return "選択した画像が間違っていたため、認証コードが失効しました。";
    }
    return "画像認証に失敗しました";
  }
  public async requiresCodeResend(): Promise<boolean> {
    return this.resendRequired;
  }
  public async resendCode(): Promise<void> {
    this.resendCount += 1;
    this.resendRequired = false;
    this.succeeded = true;
  }
}

class FakeIpoListPage {
  public navigatedUrl: string | null = null;
  public openedCompanyName: string | null = null;
  public navigateCallCount = 0;
  public navigateError: Error | null = null;

  public async navigate(url: string): Promise<void> {
    this.navigateCallCount += 1;
    if (this.navigateError !== null) {
      throw this.navigateError;
    }
    this.navigatedUrl = url;
  }

  public async openApplicationForCompany(companyName: string): Promise<void> {
    this.openedCompanyName = companyName;
  }
}

class FakeIpoApplicationPage {
  public result: ApplicationResult = { status: "success" };
  public inputPage = true;
  public enterSharesCallCount = 0;
  public enterSharesError: Error | null = null;
  public prepareForInputError: Error | null = null;

  public async navigate(): Promise<void> {}
  public async prepareForInput(): Promise<void> {
    if (this.prepareForInputError !== null) {
      throw this.prepareForInputError;
    }
  }
  public async isInputPage(): Promise<boolean> {
    return this.inputPage;
  }
  public async enterShares(): Promise<void> {
    this.enterSharesCallCount += 1;
    if (this.enterSharesError !== null) {
      throw this.enterSharesError;
    }
  }
  public async enterPrice(): Promise<void> {}
  public async enterTradingPassword(): Promise<void> {}
  public async clickConfirm(): Promise<void> {}
  public async clickSubmit(): Promise<void> {}
  public async readApplicationResult() {
    return this.result;
  }
}

class FakeRakutenSession implements RakutenSession {
  public readonly login = new FakeLoginPage();
  public readonly image = new FakeImageAuthenticationPage();
  public readonly list = new FakeIpoListPage();
  public readonly application = new FakeIpoApplicationPage();
  public closed = false;

  public loginPage(): FakeLoginPage {
    return this.login;
  }

  public imageAuthenticationPage(): FakeImageAuthenticationPage {
    return this.image;
  }

  public ipoListPage(): FakeIpoListPage {
    return this.list;
  }

  public ipoApplicationPage(): FakeIpoApplicationPage {
    return this.application;
  }

  public async close(): Promise<void> {
    this.closed = true;
  }
}

class FakePageFactory implements PageFactoryPort {
  public readonly session = new FakeRakutenSession();

  public async createSession(): Promise<RakutenSession> {
    return this.session;
  }
}

class StaticKeywordProvider implements ImageAuthenticationKeywordProvider {
  public callCount = 0;

  public async fetchKeywords() {
    this.callCount += 1;
    return { firstKeyword: "さくら", secondKeyword: "みかん" };
  }
}

class TimeoutKeywordProvider implements ImageAuthenticationKeywordProvider {
  public callCount = 0;

  public async fetchKeywords(): Promise<{
    readonly firstKeyword: string;
    readonly secondKeyword: string;
  }> {
    this.callCount += 1;
    throw new MailRetrievalTimeoutError(
      "authentication mail was not received within 120000ms",
    );
  }
}

const account = {
  identifier: "account-1",
  securitiesCompany: "Rakuten",
  credential: {
    loginId: "login",
    loginPassword: "password",
    tradingPassword: "1234",
    mailCredential: {
      mailAddress: "test@example.com",
      mailPassword: "mail-password",
      imapHost: "imap.example.com",
      imapPort: 993,
    },
  },
} as const;

const stock = {
  identifier: "stock-1",
  companyName: "テスト株式会社",
  price: 1400,
  shares: 100,
  bookBuildingStartDate: "2026-04-01",
  bookBuildingEndDate: "2026-04-10",
} as const;

/**
 * Creates a production-style navigation target resolver for tests.
 */
function createProductionNavigationTargetResolver(): ProductionRakutenNavigationTargetResolver {
  return new ProductionRakutenNavigationTargetResolver(
    "http://example.test/login",
    "http://example.test/ipo-list",
    "http://example.test/application",
  );
}

/**
 * Creates a broker adapter for tests.
 */
function createAdapter(
  pageFactory: PageFactoryPort,
  keywordProvider: ImageAuthenticationKeywordProvider,
): RakutenBrokerAdapter {
  return new RakutenBrokerAdapter(
    pageFactory,
    new BrowserSessionStorage("/tmp/ipotto-browser-tests"),
    keywordProvider,
    createProductionNavigationTargetResolver(),
    { mockLotteryResult: "Won" },
  );
}

describe("RakutenBrokerAdapter", () => {
  it("returns success when page object flow succeeds", async () => {
    const pageFactory = new FakePageFactory();
    const adapter = createAdapter(pageFactory, new StaticKeywordProvider());

    const result = await adapter.applyForIpo(account, stock);

    expect(result.status).toBe("success");
    expect(pageFactory.session.list.openedCompanyName).toBe("テスト株式会社");
    expect(pageFactory.session.closed).toBe(true);
  });

  it("solves image authentication before succeeding", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.login.imageAuthenticationRequired = true;
    pageFactory.session.login.loginSuccessful = false;
    const keywordProvider = new StaticKeywordProvider();
    const adapter = createAdapter(pageFactory, keywordProvider);

    const result = await adapter.applyForIpo(account, stock);

    expect(result.status).toBe("success");
    expect(keywordProvider.callCount).toBe(1);
  });

  it("resends the authentication code and retries once when the code expired", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.login.imageAuthenticationRequired = true;
    pageFactory.session.login.loginSuccessful = false;
    pageFactory.session.image.succeeded = false;
    pageFactory.session.image.resendRequired = true;
    const keywordProvider = new StaticKeywordProvider();
    const adapter = createAdapter(pageFactory, keywordProvider);

    const result = await adapter.applyForIpo(account, stock);

    expect(result.status).toBe("success");
    expect(pageFactory.session.image.resendCount).toBe(1);
    expect(keywordProvider.callCount).toBe(2);
  });

  it("returns already applied from the application page", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.application.result = { status: "already_applied" };
    pageFactory.session.application.inputPage = false;
    const adapter = createAdapter(pageFactory, new StaticKeywordProvider());

    const result = await adapter.applyForIpo(
      account,
      { ...stock, companyName: "申込済みテスト株式会社" },
    );

    expect(result.status).toBe("already_applied");
  });

  it("classifies mail retrieval failures separately", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.login.imageAuthenticationRequired = true;
    const keywordProvider = new TimeoutKeywordProvider();
    const adapter = createAdapter(pageFactory, keywordProvider);

    const result = await adapter.applyForIpo(account, stock);

    expect(result).toEqual({
      status: "failure",
      reason: "authentication mail was not received within 120000ms",
      category: "mail_retrieval",
    });
    expect(keywordProvider.callCount).toBe(1);
  });

  it("classifies image authentication failures separately", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.login.imageAuthenticationRequired = true;
    pageFactory.session.image.succeeded = false;
    const adapter = createAdapter(pageFactory, new StaticKeywordProvider());

    const result = await adapter.applyForIpo(account, stock);

    expect(result).toEqual({
      status: "failure",
      reason: "画像認証に失敗しました",
      category: "image_authentication",
    });
  });

  it("classifies login failures as application failures", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.login.submitSucceeds = false;
    pageFactory.session.login.errorMessage = "invalid login credentials";
    const adapter = createAdapter(pageFactory, new StaticKeywordProvider());

    const result = await adapter.applyForIpo(account, stock);

    expect(result).toEqual({
      status: "failure",
      reason: "invalid login credentials",
      category: "application",
    });
  });

  it("classifies missing 2FA page transitions as application failures", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.login.submitSucceeds = false;
    pageFactory.session.login.errorMessage = "2FA page not reached";
    const adapter = createAdapter(pageFactory, new StaticKeywordProvider());

    const result = await adapter.applyForIpo(account, stock);

    expect(result).toEqual({
      status: "failure",
      reason: "2FA page not reached",
      category: "application",
    });
  });

  it("classifies unexpected page transitions as application failures", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.application.prepareForInputError = new Error(
      "unexpected page transition",
    );
    const adapter = createAdapter(pageFactory, new StaticKeywordProvider());

    const result = await adapter.applyForIpo(account, stock);

    expect(result).toEqual({
      status: "failure",
      reason: "unexpected page transition",
      category: "application",
    });
  });

  it("does not retry when a selector is missing", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.application.enterSharesError = new Error(
      "selector missing",
    );
    const adapter = createAdapter(pageFactory, new StaticKeywordProvider());

    const result = await adapter.applyForIpo(account, stock);

    expect(result).toEqual({
      status: "failure",
      reason: "selector missing",
      category: "application",
    });
    expect(pageFactory.session.application.enterSharesCallCount).toBe(1);
  });

  it("does not retry when the broker returns a temporary application error", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.list.navigateError = new Error("temporary broker error");
    const adapter = createAdapter(pageFactory, new StaticKeywordProvider());

    const result = await adapter.applyForIpo(account, stock);

    expect(result).toEqual({
      status: "failure",
      reason: "temporary broker error",
      category: "application",
    });
    expect(pageFactory.session.list.navigateCallCount).toBe(1);
  });

  it("keeps production navigation free from fixture scenario hints", async () => {
    const pageFactory = new FakePageFactory();
    const adapter = createAdapter(pageFactory, new StaticKeywordProvider());

    await adapter.applyForIpo(account, stock);

    expect(pageFactory.session.list.navigatedUrl).toBe("http://example.test/ipo-list");
  });

  it("encodes fixture scenarios only through the fixture resolver", async () => {
    const pageFactory = new FakePageFactory();
    const adapter = new RakutenBrokerAdapter(
      pageFactory,
      new BrowserSessionStorage("/tmp/ipotto-browser-tests"),
      new StaticKeywordProvider(),
      new FixtureRakutenNavigationTargetResolver("http://fixture.test"),
      { mockLotteryResult: "Won" },
    );

    await adapter.applyForIpo(account, { ...stock, companyName: "残高不足テスト株式会社" });

    expect(pageFactory.session.list.navigatedUrl).toBe(
      "http://fixture.test/rakuten/ipo_list_page.html?companyName=%E6%AE%8B%E9%AB%98%E4%B8%8D%E8%B6%B3%E3%83%86%E3%82%B9%E3%83%88%E6%A0%AA%E5%BC%8F%E4%BC%9A%E7%A4%BE&scenario=insufficient_balance",
    );
  });
});
