import { test, expect } from "@playwright/test";

// Phase 3 Sprint 7 TST-074 — 抽選結果確認 E2E.
// HTML モックの result_page の4 パターン網羅 + ipo-browser の
// /internal/lottery-results/check が LotteryResult 3 variants と
// 未発表 (null) を正しく返すことを smoke レベルで確認する。

test.describe("Rakuten lottery result HTML fixture", () => {
  test("result_page exposes 4 rows with result-label text", async ({ page }) => {
    await page.goto("/rakuten/result_page.html");
    await expect(page.locator("h1")).toHaveText("IPO 抽選結果");
    const labels = await page.locator(".result-label").evaluateAll((nodes) =>
      nodes.map((node) => (node.textContent ?? "").trim()),
    );
    expect(labels).toEqual(["当選", "落選", "補欠", "未発表"]);
  });

  test("result_page rows are addressable by data-stock-identifier", async ({
    page,
  }) => {
    await page.goto("/rakuten/result_page.html");
    const won = await page
      .locator('[data-stock-identifier="01HA1234567890ABCDEFGHJKMN"] .result-label')
      .innerText();
    expect(won.trim()).toBe("当選");
    const lost = await page
      .locator('[data-stock-identifier="01HB1234567890ABCDEFGHJKMN"] .result-label')
      .innerText();
    expect(lost.trim()).toBe("落選");
    const alternate = await page
      .locator('[data-stock-identifier="01HC1234567890ABCDEFGHJKMN"] .result-label')
      .innerText();
    expect(alternate.trim()).toBe("補欠");
    const pending = await page
      .locator('[data-stock-identifier="01HD1234567890ABCDEFGHJKMN"] .result-label')
      .innerText();
    expect(pending.trim()).toBe("未発表");
  });
});

test.describe("ipo-browser /internal/lottery-results/check", () => {
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

  test("Won stock returns {result: 'Won'}", async ({ request }) => {
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-results/check`,
      {
        data: {
          credential,
          stockIdentifier: "01HA1234567890ABCDEFGHJKMN",
        },
      },
    );
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ result: "Won" });
  });

  test("Lost stock returns {result: 'Lost'}", async ({ request }) => {
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-results/check`,
      {
        data: {
          credential,
          stockIdentifier: "01HB1234567890ABCDEFGHJKMN",
        },
      },
    );
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ result: "Lost" });
  });

  test("Alternate stock returns {result: 'Alternate'}", async ({ request }) => {
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-results/check`,
      {
        data: {
          credential,
          stockIdentifier: "01HC1234567890ABCDEFGHJKMN",
        },
      },
    );
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ result: "Alternate" });
  });

  test("Pending stock returns {result: null}", async ({ request }) => {
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-results/check`,
      {
        data: {
          credential,
          stockIdentifier: "01HD1234567890ABCDEFGHJKMN",
        },
      },
    );
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ result: null });
  });

  test("Unknown stockIdentifier returns {result: null}", async ({ request }) => {
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-results/check`,
      {
        data: {
          credential,
          stockIdentifier: "01UNKNOWN567890ABCDEFGHJKMN",
        },
      },
    );
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ result: null });
  });
});
