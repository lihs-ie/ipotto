import { test, expect } from "@playwright/test";

// Phase 3 Sprint 7 TST-072 — IPO 申込フルフロー.
// HTML モックに対する fixture 正当性検証 + ipo-browser の
// /internal/lottery-applications/submit が各 ApplicationResult variant
// を返すことを smoke レベルで確認する。

test.describe("Rakuten apply HTML fixtures", () => {
  test("apply_list_page exposes 4 stock rows with data-stock-identifier", async ({
    page,
  }) => {
    await page.goto("/rakuten/apply_list_page.html");
    await expect(page.locator("h1")).toHaveText("IPO 申込一覧");
    await expect(page.locator("article.ipo-stock")).toHaveCount(4);
    const identifiers = await page
      .locator("article.ipo-stock")
      .evaluateAll((nodes) =>
        nodes.map((node) => node.getAttribute("data-stock-identifier")),
      );
    expect(identifiers).toEqual([
      "01HA1234567890ABCDEFGHJKMN",
      "01HB1234567890ABCDEFGHJKMN",
      "01HC1234567890ABCDEFGHJKMN",
      "01HD1234567890ABCDEFGHJKMN",
    ]);
  });

  test("apply_form_page exposes shares / price / tradingPassword inputs and submit", async ({
    page,
  }) => {
    await page.goto("/rakuten/apply_form_page.html?stockIdentifier=01HA1234567890ABCDEFGHJKMN");
    await expect(page.locator("#shares")).toBeVisible();
    await expect(page.locator("#price")).toBeVisible();
    await expect(page.locator("#tradingPassword")).toBeVisible();
    await expect(page.locator("#submit-apply")).toBeVisible();
  });

  test("apply_success_page contains the success keyword", async ({ page }) => {
    await page.goto("/rakuten/apply_success_page.html");
    await expect(page.locator(".apply-success")).toContainText("受け付けました");
  });

  test("apply_duplicate_page contains the duplicate keyword", async ({ page }) => {
    await page.goto("/rakuten/apply_duplicate_page.html");
    await expect(page.locator(".apply-duplicate")).toContainText("既に申込済み");
  });

  test("apply_insufficient_balance_page contains the balance keyword", async ({
    page,
  }) => {
    await page.goto("/rakuten/apply_insufficient_balance_page.html");
    await expect(page.locator(".apply-insufficient-balance")).toContainText(
      "残高が不足",
    );
  });

  test("apply_failure_page uses .apply-error class", async ({ page }) => {
    await page.goto("/rakuten/apply_failure_page.html");
    await expect(page.locator(".apply-error")).toBeVisible();
  });

  test("apply_form routing script picks the right destination per stock prefix", async ({
    page,
  }) => {
    for (const [identifier, expected] of [
      ["01HA1234567890ABCDEFGHJKMN", "/rakuten/apply_success_page.html"],
      ["01HB1234567890ABCDEFGHJKMN", "/rakuten/apply_duplicate_page.html"],
      ["01HC1234567890ABCDEFGHJKMN", "/rakuten/apply_insufficient_balance_page.html"],
      ["01HD1234567890ABCDEFGHJKMN", "/rakuten/apply_failure_page.html"],
    ] as const) {
      await page.goto(`/rakuten/apply_form_page.html?stockIdentifier=${identifier}`);
      const action = await page.locator("#apply-form").getAttribute("action");
      expect(action).toBe(expected);
    }
  });
});

test.describe("ipo-browser /internal/lottery-applications/submit", () => {
  const browserServiceUrl =
    process.env["IPO_BROWSER_URL"] ?? "http://localhost:8081";

  const credential = {
    loginId: "smokeuser",
    loginPassword: "smoke-password-1",
    tradingPassword: "smoke-trading-1",
    mailAddress: "smoke@example.com",
    mailPassword: "smoke-mail-1",
    imapHost: "imap.example.com",
    imapPort: 993,
  };

  test("success path returns {status: 'success'} for 01HA prefix stock", async ({
    request,
  }) => {
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-applications/submit`,
      {
        data: {
          credential,
          stockIdentifier: "01HA1234567890ABCDEFGHJKMN",
          companyName: "テスト第一株式会社",
          shares: 100,
          price: 1400,
        },
      },
    );
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ status: "success" });
  });

  test("duplicate path returns {status: 'already_applied'} for 01HB prefix stock", async ({
    request,
  }) => {
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-applications/submit`,
      {
        data: {
          credential,
          stockIdentifier: "01HB1234567890ABCDEFGHJKMN",
          companyName: "テスト第二株式会社",
          shares: 100,
          price: 2100,
        },
      },
    );
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ status: "already_applied" });
  });

  test("insufficient-balance path returns {status: 'insufficient_balance'} for 01HC prefix stock", async ({
    request,
  }) => {
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-applications/submit`,
      {
        data: {
          credential,
          stockIdentifier: "01HC1234567890ABCDEFGHJKMN",
          companyName: "テスト第三株式会社",
          shares: 100,
          price: 1800,
        },
      },
    );
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ status: "insufficient_balance" });
  });
});
