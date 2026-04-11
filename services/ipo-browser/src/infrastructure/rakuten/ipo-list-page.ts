import type { Page } from "playwright";

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
    const block = this.page
      .locator(".pcmm_ipolt-ipo-block")
      .filter({
        has: this.page.locator(".pcmm_ipolt-ipo-block__stockname", {
          hasText: companyName,
        }),
      })
      .first();

    if ((await block.count()) === 0) {
      throw new Error(`IPO stock not found on list page: ${companyName}`);
    }

    const applicationButton = block
      .locator("button")
      .filter({ hasText: "ブックビルディング申込" })
      .first();
    if ((await applicationButton.count()) > 0) {
      await Promise.all([
        this.page.waitForLoadState("domcontentloaded"),
        applicationButton.click(),
      ]);
      return;
    }

    const detailButton = block
      .locator("button")
      .filter({ hasText: /申込詳細を見る|銘柄詳細を見る/u })
      .first();
    if ((await detailButton.count()) > 0) {
      await Promise.all([
        this.page.waitForLoadState("domcontentloaded"),
        detailButton.click(),
      ]);
      return;
    }

    const stockNameLink = block.locator(".pcmm_ipolt-ipo-block__stockname").first();
    if ((await stockNameLink.count()) > 0) {
      await Promise.all([
        this.page.waitForLoadState("domcontentloaded"),
        stockNameLink.click(),
      ]);
      return;
    }

    throw new Error(`IPO application entry point not found: ${companyName}`);
  }
}
