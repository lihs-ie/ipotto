import { describe, expect, it } from "vitest";
import type { Page } from "playwright";

import { IpoListPage } from "./ipo-list-page.js";

type BlockState = {
  readonly companyName: string;
  readonly hasApplyButton: boolean;
  readonly hasApplicationDetailButton: boolean;
  readonly hasStockDetailButton: boolean;
  readonly hasStockLink: boolean;
};

class FakeHasTextLocator {
  public constructor(public readonly hasText: string) {}
}

class FakeActionTarget {
  public constructor(
    private readonly page: FakePage,
    private readonly companyName: string,
    private readonly kind: string,
  ) {}

  public first(): FakeActionTarget {
    return this;
  }

  public async click(): Promise<void> {
    this.page.clicked = {
      companyName: this.companyName,
      kind: this.kind,
    };
  }
}

class FakeActionCollection {
  public constructor(
    private readonly page: FakePage,
    private readonly block: BlockState,
    private readonly kind: string,
  ) {}

  public filter(options: { hasText: string }): FakeActionCollection {
    const kind =
      options.hasText === "ブックビルディング申込"
        ? "apply"
        : options.hasText === "申込詳細を見る"
          ? "application_detail"
          : "stock_detail";
    return new FakeActionCollection(this.page, this.block, kind);
  }

  public async count(): Promise<number> {
    switch (this.kind) {
      case "apply":
        return this.block.hasApplyButton ? 1 : 0;
      case "application_detail":
        return this.block.hasApplicationDetailButton ? 1 : 0;
      case "stock_detail":
        return this.block.hasStockDetailButton ? 1 : 0;
      case "stock_link":
        return this.block.hasStockLink ? 1 : 0;
      default:
        return 0;
    }
  }

  public first(): FakeActionTarget {
    return new FakeActionTarget(this.page, this.block.companyName, this.kind);
  }
}

class FakeBlockLocator {
  public constructor(
    private readonly page: FakePage,
    private readonly block: BlockState,
  ) {}

  public locator(selector: string): FakeActionCollection {
    if (selector === "button") {
      return new FakeActionCollection(this.page, this.block, "button");
    }
    if (selector === ".pcmm_ipolt-ipo-block__stockname") {
      return new FakeActionCollection(this.page, this.block, "stock_link");
    }
    return new FakeActionCollection(this.page, this.block, "unknown");
  }
}

class FakeBlockCollection {
  public constructor(
    private readonly page: FakePage,
    private readonly blocks: readonly BlockState[],
  ) {}

  public filter(options: { has: FakeHasTextLocator }): FakeBlockCollection {
    return new FakeBlockCollection(
      this.page,
      this.blocks.filter((block) => block.companyName === options.has.hasText),
    );
  }

  public async count(): Promise<number> {
    return this.blocks.length;
  }

  public nth(index: number): FakeBlockLocator {
    const block = this.blocks[index];
    if (block === undefined) {
      throw new Error(`missing fake block at index ${index}`);
    }
    return new FakeBlockLocator(this.page, block);
  }
}

class FakePage {
  public clicked:
    | {
        readonly companyName: string;
        readonly kind: string;
      }
    | null = null;
  public waitForLoadStateCalls: string[] = [];

  public constructor(private readonly blocks: readonly BlockState[]) {}

  public locator(
    selector: string,
    options?: { hasText?: string },
  ): FakeBlockCollection | FakeHasTextLocator {
    if (selector === ".pcmm_ipolt-ipo-block") {
      return new FakeBlockCollection(this, this.blocks);
    }
    if (
      selector === ".pcmm_ipolt-ipo-block__stockname" &&
      options?.hasText !== undefined
    ) {
      return new FakeHasTextLocator(options.hasText);
    }
    throw new Error(`unexpected selector: ${selector}`);
  }

  public async goto(): Promise<void> {}

  public async waitForLoadState(state: string): Promise<void> {
    this.waitForLoadStateCalls.push(state);
  }
}

describe("IpoListPage", () => {
  it("prefers the apply action when duplicate company blocks exist", async () => {
    const page = new FakePage([
      {
        companyName: "テストIPO",
        hasApplyButton: false,
        hasApplicationDetailButton: true,
        hasStockDetailButton: false,
        hasStockLink: true,
      },
      {
        companyName: "テストIPO",
        hasApplyButton: true,
        hasApplicationDetailButton: false,
        hasStockDetailButton: false,
        hasStockLink: true,
      },
    ]);

    const ipoListPage = new IpoListPage(page as unknown as Page);

    await ipoListPage.openApplicationForCompany("テストIPO");

    expect(page.clicked).toEqual({
      companyName: "テストIPO",
      kind: "apply",
    });
    expect(page.waitForLoadStateCalls).toEqual(["domcontentloaded"]);
  });

  it("falls back to the stock link when no buttons are available", async () => {
    const page = new FakePage([
      {
        companyName: "リンクIPO",
        hasApplyButton: false,
        hasApplicationDetailButton: false,
        hasStockDetailButton: false,
        hasStockLink: true,
      },
    ]);

    const ipoListPage = new IpoListPage(page as unknown as Page);

    await ipoListPage.openApplicationForCompany("リンクIPO");

    expect(page.clicked).toEqual({
      companyName: "リンクIPO",
      kind: "stock_link",
    });
  });

  it("throws when the company is missing or no entry point exists", async () => {
    const page = new FakePage([
      {
        companyName: "対象外IPO",
        hasApplyButton: false,
        hasApplicationDetailButton: false,
        hasStockDetailButton: false,
        hasStockLink: false,
      },
    ]);
    const ipoListPage = new IpoListPage(page as unknown as Page);

    await expect(
      ipoListPage.openApplicationForCompany("存在しないIPO"),
    ).rejects.toThrow("IPO stock not found on list page: 存在しないIPO");
    await expect(
      ipoListPage.openApplicationForCompany("対象外IPO"),
    ).rejects.toThrow("IPO application entry point not found: 対象外IPO");
  });
});
