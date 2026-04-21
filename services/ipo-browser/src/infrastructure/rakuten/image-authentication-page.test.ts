import { describe, expect, it } from "vitest";
import type { Page } from "playwright";

import { ImageAuthenticationPage } from "./image-authentication-page.js";

class FakeLocator {
  public constructor(
    private readonly state: {
      count: number;
      text?: string | null;
      onClick?: () => void;
    },
  ) {}

  public first(): FakeLocator {
    return this;
  }

  public nth(): FakeLocator {
    return this;
  }

  public async count(): Promise<number> {
    return this.state.count;
  }

  public async click(): Promise<void> {
    this.state.onClick?.();
  }

  public async textContent(): Promise<string | null> {
    return this.state.text ?? null;
  }

  public async evaluateAll(): Promise<unknown[]> {
    return [];
  }
}

class FakePage {
  public readonly locatorStates = new Map<
    string,
    {
      count: number;
      text?: string | null;
      onClick?: () => void;
    }
  >();

  public locator(selector: string): FakeLocator {
    return new FakeLocator(this.locatorStates.get(selector) ?? { count: 0 });
  }

  public async waitForLoadState(): Promise<void> {}
}

describe("ImageAuthenticationPage", () => {
  it("returns false when resend is not required and no resend button exists", async () => {
    const page = new FakePage();
    const authenticationPage = new ImageAuthenticationPage(page as unknown as Page);

    await expect(authenticationPage.requiresCodeResend()).resolves.toBe(false);
  });

  it("throws when resend is requested but the resend button does not exist", async () => {
    const page = new FakePage();
    const authenticationPage = new ImageAuthenticationPage(page as unknown as Page);

    await expect(authenticationPage.resendCode()).rejects.toThrow(
      "authentication code resend button not found",
    );
  });
});
