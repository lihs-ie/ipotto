import { describe, expect, it } from "vitest";
import type { Page } from "playwright";

import { LoginPage } from "./login-page.js";

class FakeLocator {
  public constructor(
    private readonly state: {
      count: number;
      text?: string | null;
      onFill?: (value: string) => void;
      onClick?: () => void;
    },
  ) {}

  public async fill(value: string): Promise<void> {
    this.state.onFill?.(value);
  }

  public async click(): Promise<void> {
    this.state.onClick?.();
  }

  public async count(): Promise<number> {
    return this.state.count;
  }

  public async textContent(): Promise<string | null> {
    return this.state.text ?? null;
  }

  public first(): FakeLocator {
    return this;
  }
}

class FakePage {
  public titleValue = "";
  public gotoUrl: string | null = null;
  public waitForLoadStateCalls: string[] = [];
  public readonly locatorStates = new Map<
    string,
    {
      count: number;
      text?: string | null;
      onFill?: (value: string) => void;
      onClick?: () => void;
    }
  >();

  public locator(selector: string): FakeLocator {
    return new FakeLocator(this.locatorStates.get(selector) ?? { count: 0 });
  }

  public async goto(url: string): Promise<void> {
    this.gotoUrl = url;
  }

  public async waitForLoadState(state: string): Promise<void> {
    this.waitForLoadStateCalls.push(state);
  }

  public async title(): Promise<string> {
    return this.titleValue;
  }

  public getByRole(): FakeLocator {
    return new FakeLocator({ count: 0 });
  }
}

describe("LoginPage", () => {
  it("uses Rakuten login form selectors from the real login page first", async () => {
    const page = new FakePage();
    const recorded = {
      loginId: "",
      password: "",
      clicked: false,
    };
    page.locatorStates.set("#form-login-id", {
      count: 1,
      onFill: (value) => {
        recorded.loginId = value;
      },
    });
    page.locatorStates.set("#form-login-pass", {
      count: 1,
      onFill: (value) => {
        recorded.password = value;
      },
    });
    page.locatorStates.set("#login-btn", {
      count: 1,
      onClick: () => {
        recorded.clicked = true;
      },
    });

    const loginPage = new LoginPage(page as unknown as Page);

    await loginPage.enterLoginId("login-id");
    await loginPage.enterPassword("password");
    await loginPage.clickSubmit();

    expect(recorded).toEqual({
      loginId: "login-id",
      password: "password",
      clicked: true,
    });
    expect(page.waitForLoadStateCalls).toEqual(["domcontentloaded"]);
  });

  it("falls back to name-based selectors when Rakuten ids are unavailable", async () => {
    const page = new FakePage();
    const recorded = {
      loginId: "",
      password: "",
      clicked: false,
    };
    page.locatorStates.set('input[name="loginid"]', {
      count: 1,
      onFill: (value) => {
        recorded.loginId = value;
      },
    });
    page.locatorStates.set('input[name="passwd"]', {
      count: 1,
      onFill: (value) => {
        recorded.password = value;
      },
    });
    page.locatorStates.set('input[type="submit"]', {
      count: 1,
      onClick: () => {
        recorded.clicked = true;
      },
    });

    const loginPage = new LoginPage(page as unknown as Page);

    await loginPage.enterLoginId("login-id");
    await loginPage.enterPassword("password");
    await loginPage.clickSubmit();

    expect(recorded).toEqual({
      loginId: "login-id",
      password: "password",
      clicked: true,
    });
    expect(page.waitForLoadStateCalls).toEqual(["domcontentloaded"]);
  });

  it("detects dashboard success from DOM markers without relying only on the title", async () => {
    const page = new FakePage();
    page.titleValue = "楽天証券";
    page.locatorStates.set('form[name="HomeForm"][action*="/app/home.do"]', {
      count: 1,
    });

    const loginPage = new LoginPage(page as unknown as Page);

    await expect(loginPage.isLoginSuccessful()).resolves.toBe(true);
  });

  it("detects image authentication from ratPageName markers", async () => {
    const page = new FakePage();
    page.locatorStates.set(
      'input#ratPageName[value*="[member]/app/sotp_login.do"]',
      { count: 1 },
    );

    const loginPage = new LoginPage(page as unknown as Page);

    await expect(loginPage.isImageAuthenticationRequired()).resolves.toBe(true);
  });

  it("reads the first non-empty error message from fallback selectors", async () => {
    const page = new FakePage();
    page.locatorStates.set('[role="alert"]', {
      count: 1,
      text: "ログインに失敗しました",
    });

    const loginPage = new LoginPage(page as unknown as Page);

    await expect(loginPage.getErrorMessage()).resolves.toBe(
      "ログインに失敗しました",
    );
  });
});
