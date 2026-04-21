import type { Page } from "playwright";

import {
  selectPreferredIpoListBlockAction,
  type IpoListBlockActionKind,
  type IpoListBlockSnapshot,
} from "./ipo-list-block-action.js";

/**
 * Public contract for the IPO list page object.
 */
export interface IpoListPagePort {
  navigate(url: string): Promise<void>;
  openApplicationForCompany(companyName: string): Promise<void>;
}

/**
 * Page object for the Rakuten IPO list page.
 */
export class IpoListPage implements IpoListPagePort {
  /**
   * Creates the page object.
   */
  public constructor(private readonly page: Page) {}

  /**
   * Navigates to the IPO list page.
   */
  public async navigate(url: string): Promise<void> {
    await this.page.goto(url, { waitUntil: "domcontentloaded" });
  }

  /**
   * Opens the application or detail flow for the target company.
   */
  public async openApplicationForCompany(companyName: string): Promise<void> {
    const matchingBlocks = this.page
      .locator(".pcmm_ipolt-ipo-block")
      .filter({
        has: this.page.locator(".pcmm_ipolt-ipo-block__stockname", {
          hasText: companyName,
        }),
      });

    const matchingBlockCount = await matchingBlocks.count();
    if (matchingBlockCount === 0) {
      throw new Error(`IPO stock not found on list page: ${companyName}`);
    }

    const snapshots: IpoListBlockSnapshot[] = [];
    for (let index = 0; index < matchingBlockCount; index += 1) {
      const block = matchingBlocks.nth(index);
      snapshots.push({
        index,
        hasApplyButton:
          (await block
            .locator("button")
            .filter({ hasText: "ブックビルディング申込" })
            .count()) > 0,
        hasApplicationDetailButton:
          (await block
            .locator("button")
            .filter({ hasText: "申込詳細を見る" })
            .count()) > 0,
        hasStockDetailButton:
          (await block
            .locator("button")
            .filter({ hasText: "銘柄詳細を見る" })
            .count()) > 0,
        hasStockLink:
          (await block.locator(".pcmm_ipolt-ipo-block__stockname").count()) > 0,
      });
    }

    const selectedAction = selectPreferredIpoListBlockAction(snapshots);
    if (selectedAction === null) {
      throw new Error(`IPO application entry point not found: ${companyName}`);
    }

    const selectedBlock = matchingBlocks.nth(selectedAction.index);
    const actionTarget = resolveActionTarget(selectedBlock, selectedAction.kind);

    await Promise.all([
      this.page.waitForLoadState("domcontentloaded"),
      actionTarget.click(),
    ]);
  }
}

/**
 * Resolves the click target for the selected action kind.
 */
function resolveActionTarget(
  block: ReturnType<Page["locator"]>,
  actionKind: IpoListBlockActionKind,
) {
  switch (actionKind) {
    case "apply":
      return block
        .locator("button")
        .filter({ hasText: "ブックビルディング申込" })
        .first();
    case "application_detail":
      return block.locator("button").filter({ hasText: "申込詳細を見る" }).first();
    case "stock_detail":
      return block.locator("button").filter({ hasText: "銘柄詳細を見る" }).first();
    case "stock_link":
      return block.locator(".pcmm_ipolt-ipo-block__stockname").first();
  }
}
