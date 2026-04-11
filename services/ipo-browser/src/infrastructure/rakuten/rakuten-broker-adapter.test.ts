import { describe, expect, it } from "vitest";

import type { ApplicationResult } from "../../domain/application-result.js";
import { BrowserSessionStorage } from "../session/browser-session-storage.js";
import {
  type ImageAuthenticationKeywordProvider,
  RakutenBrokerAdapter,
} from "./rakuten-broker-adapter.js";
import type { PageFactoryPort, RakutenSession } from "./page-factory.js";

class FakeLoginPage {
  public imageAuthenticationRequired = false;
  public loginSuccessful = false;

  public async navigate(): Promise<void> {}
  public async enterLoginId(): Promise<void> {}
  public async enterPassword(): Promise<void> {}
  public async clickSubmit(): Promise<void> {
    if (!this.imageAuthenticationRequired) {
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
    return null;
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

  public async navigate(url: string): Promise<void> {
    this.navigatedUrl = url;
  }

  public async openApplicationForCompany(companyName: string): Promise<void> {
    this.openedCompanyName = companyName;
  }
}

class FakeIpoApplicationPage {
  public result: ApplicationResult = { status: "success" };
  public inputPage = true;

  public async navigate(): Promise<void> {}
  public async prepareForInput(): Promise<void> {}
  public async isInputPage(): Promise<boolean> {
    return this.inputPage;
  }
  public async enterShares(): Promise<void> {}
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

describe("RakutenBrokerAdapter", () => {
  it("returns success when page object flow succeeds", async () => {
    const pageFactory = new FakePageFactory();
    const adapter = new RakutenBrokerAdapter(
      pageFactory,
      new BrowserSessionStorage("/tmp/ipotto-browser-tests"),
      new StaticKeywordProvider(),
      {
        dryRun: true,
        loginPageUrl: "http://example.test/login",
        ipoListPageUrl: "http://example.test/ipo-list",
        applicationPageUrl: "http://example.test/application",
        mockLotteryResult: "Won",
      },
    );

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
    const adapter = new RakutenBrokerAdapter(
      pageFactory,
      new BrowserSessionStorage("/tmp/ipotto-browser-tests"),
      keywordProvider,
      {
        dryRun: true,
        loginPageUrl: "http://example.test/login",
        ipoListPageUrl: "http://example.test/ipo-list",
        applicationPageUrl: "http://example.test/application",
        mockLotteryResult: "Won",
      },
    );

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
    const adapter = new RakutenBrokerAdapter(
      pageFactory,
      new BrowserSessionStorage("/tmp/ipotto-browser-tests"),
      keywordProvider,
      {
        dryRun: true,
        loginPageUrl: "http://example.test/login",
        ipoListPageUrl: "http://example.test/ipo-list",
        applicationPageUrl: "http://example.test/application",
        mockLotteryResult: "Won",
      },
    );

    const result = await adapter.applyForIpo(account, stock);

    expect(result.status).toBe("success");
    expect(pageFactory.session.image.resendCount).toBe(1);
    expect(keywordProvider.callCount).toBe(2);
  });

  it("returns already applied from the application page", async () => {
    const pageFactory = new FakePageFactory();
    pageFactory.session.application.result = { status: "already_applied" };
    pageFactory.session.application.inputPage = false;
    const adapter = new RakutenBrokerAdapter(
      pageFactory,
      new BrowserSessionStorage("/tmp/ipotto-browser-tests"),
      new StaticKeywordProvider(),
      {
        dryRun: true,
        loginPageUrl: "http://example.test/login",
        ipoListPageUrl: "http://example.test/ipo-list",
        applicationPageUrl: "http://example.test/application",
        mockLotteryResult: "Won",
      },
    );

    const result = await adapter.applyForIpo(
      account,
      { ...stock, companyName: "申込済みテスト株式会社" },
    );

    expect(result.status).toBe("already_applied");
  });
});
