import { describe, expect, it } from "vitest";
import type { Page } from "playwright";

import { IpoApplicationPage } from "./ipo-application-page.js";

class FakeLocator {
  public constructor(
    private readonly state: {
      count: number;
      onClick?: () => void;
      onFill?: (value: string) => void;
      text?: string | null;
      optionTexts?: string[];
      onSelectOption?: (value: string) => void;
    },
  ) {}

  public first(): FakeLocator {
    return this;
  }

  public async count(): Promise<number> {
    return this.state.count;
  }

  public async click(): Promise<void> {
    this.state.onClick?.();
  }

  public async fill(value: string): Promise<void> {
    this.state.onFill?.(value);
  }

  public locator(): FakeLocator {
    return new FakeLocator({
      count: this.state.optionTexts === undefined ? 0 : this.state.optionTexts.length,
      optionTexts: this.state.optionTexts,
      onSelectOption: this.state.onSelectOption,
    });
  }

  public async allTextContents(): Promise<string[]> {
    return this.state.optionTexts ?? [];
  }

  public async selectOption(option: { value?: string; label?: string }): Promise<void> {
    this.state.onSelectOption?.(option.value ?? option.label ?? "");
  }

  public async textContent(): Promise<string | null> {
    return this.state.text ?? null;
  }
}

class FakePage {
  public titleValue = "";
  public readonly waitForLoadStateCalls: string[] = [];
  public readonly waitForFunctionCalls: Array<{
    selectors: readonly string[];
    titleIncludes: readonly string[];
  }> = [];
  public readonly locatorStates = new Map<
    string,
    {
      count: number;
      onClick?: () => void;
      onFill?: (value: string) => void;
      text?: string | null;
      optionTexts?: string[];
      onSelectOption?: (value: string) => void;
    }
  >();

  public locator(selector: string): FakeLocator {
    return new FakeLocator(this.locatorStates.get(selector) ?? { count: 0 });
  }

  public async goto(): Promise<void> {}

  public async waitForLoadState(state: string): Promise<void> {
    this.waitForLoadStateCalls.push(state);
  }

  public async title(): Promise<string> {
    return this.titleValue;
  }

  public async waitForFunction(
    _fn: unknown,
    arg: {
      selectors: readonly string[];
      titleIncludes: readonly string[];
    },
  ): Promise<void> {
    this.waitForFunctionCalls.push(arg);
  }
}

describe("IpoApplicationPage", () => {
  it("waits for the input stage after leaving the caution page", async () => {
    const page = new FakePage();
    page.locatorStates.set(
      'button:has-text("同意して次へ"), input[type="submit"][value="同意して次へ"]',
      {
        count: 1,
      },
    );

    const applicationPage = new IpoApplicationPage(page as unknown as Page);

    await applicationPage.prepareForInput();

    expect(page.waitForLoadStateCalls).toEqual(["domcontentloaded"]);
    expect(page.waitForFunctionCalls).toEqual([
      {
        selectors: [
          "#orderValueInput",
          "#orderValue",
          "#priceSpinnerComBox",
          '[data-page="ipo-application"]',
          'input#ratPageName[value*="ipo_jp_join_input.do"]',
        ],
        titleIncludes: ["BB参加 / 受付", "楽天証券 IPO申し込み"],
      },
    ]);
  });

  it("waits for the confirm stage after clicking confirm", async () => {
    const page = new FakePage();
    page.locatorStates.set(
      'button:has-text("申込内容を確認する"), button#confirm-button, input[type="submit"][value="申込内容を確認する"]',
      {
        count: 1,
      },
    );

    const applicationPage = new IpoApplicationPage(page as unknown as Page);

    await applicationPage.clickConfirm();

    expect(page.waitForFunctionCalls).toEqual([
      {
        selectors: [
          "#passwordInputText",
          '[data-page="ipo-confirm"]',
          'input#ratPageName[value*="ipo_jp_join_confirm.do"]',
        ],
        titleIncludes: ["BB参加 / 確認"],
      },
    ]);
  });

  it("waits for the result stage after clicking submit", async () => {
    const page = new FakePage();
    page.locatorStates.set(
      'button:has-text("申し込む"), button#submit-button, input[type="submit"][value="申し込む"]',
      {
        count: 1,
      },
    );

    const applicationPage = new IpoApplicationPage(page as unknown as Page);

    await applicationPage.clickSubmit();

    expect(page.waitForFunctionCalls).toEqual([
      {
        selectors: [
          ".pcmm_ipolt-complete__txt",
          '[data-application-result="success"]',
          '[data-application-result="already-applied"]',
          '[data-application-result="insufficient-balance"]',
          'input#ratPageName[value*="ipo_jp_join_result.do"]',
        ],
        titleIncludes: ["BB参加 / 完了"],
      },
    ]);
  });

  it("skips the explicit stage wait when the input page is already visible", async () => {
    const page = new FakePage();
    page.locatorStates.set(
      'button:has-text("同意して次へ"), input[type="submit"][value="同意して次へ"]',
      {
        count: 1,
      },
    );
    page.locatorStates.set("#orderValueInput", { count: 1 });

    const applicationPage = new IpoApplicationPage(page as unknown as Page);

    await applicationPage.prepareForInput();

    expect(page.waitForFunctionCalls).toEqual([]);
  });

  it("falls back to name-based selectors for shares, price, and trading password", async () => {
    const page = new FakePage();
    const recorded = {
      shares: "",
      price: "",
      password: "",
    };
    page.locatorStates.set('input[name="orderValueInput"]', {
      count: 1,
      onFill: (value) => {
        recorded.shares = value;
      },
    });
    page.locatorStates.set("select.pcmm-slb__input", {
      count: 1,
      optionTexts: ["成行", "600 円", "590 円"],
      onSelectOption: (value: string) => {
        recorded.price = value;
      },
    });
    page.locatorStates.set('input[name="password"]', {
      count: 1,
      onFill: (value) => {
        recorded.password = value;
      },
    });

    const applicationPage = new IpoApplicationPage(page as unknown as Page);

    await applicationPage.enterShares(100);
    await applicationPage.enterPrice(600);
    await applicationPage.enterTradingPassword("1234");

    expect(recorded).toEqual({
      shares: "100",
      price: "600",
      password: "1234",
    });
  });
});
