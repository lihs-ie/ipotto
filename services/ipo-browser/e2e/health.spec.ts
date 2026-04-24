import { test, expect } from "@playwright/test";

test.describe("HTML Mock Server Health Check", () => {
  test("mock server serves index page", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h1")).toHaveText("HTML Mock Server");
  });

  test("login page fixture is accessible", async ({ page }) => {
    await page.goto("/rakuten/login_page.html");
    await expect(page.locator("h1")).toHaveText("楽天証券 ログイン");
  });

  test("login form elements exist", async ({ page }) => {
    await page.goto("/rakuten/login_page.html");
    await expect(page.locator("#loginid")).toBeVisible();
    await expect(page.locator("#passwd")).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test("image authentication fixture exposes alt-tagged image buttons (Phase 3 Sprint 6)", async ({
    page,
  }) => {
    await page.goto("/rakuten/image_auth_page.html");
    await expect(page.locator("#image-auth-container")).toBeVisible();
    await expect(page.locator("#image-buttons button")).toHaveCount(10);
    const altTexts = await page.locator("#image-buttons img[alt]").evaluateAll(
      (nodes) => nodes.map((node) => (node as HTMLImageElement).alt),
    );
    expect(altTexts).toHaveLength(10);
    expect(new Set(altTexts).size).toBe(10);
    await expect(page.locator("#submit-button")).toBeVisible();
  });
});

test.describe("ipo-browser Service Health Check", () => {
  test("health endpoint returns ok", async ({ request }) => {
    const browserServiceUrl =
      process.env["IPO_BROWSER_URL"] ?? "http://localhost:8081";
    const response = await request.get(`${browserServiceUrl}/health`);
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ status: "ok" });
  });

  test("stocks stub returns empty array (Phase 3 Sprint 5.1)", async ({
    request,
  }) => {
    const browserServiceUrl =
      process.env["IPO_BROWSER_URL"] ?? "http://localhost:8081";
    const response = await request.get(`${browserServiceUrl}/internal/stocks`);
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual([]);
  });

  test("lottery-results rejects missing stockIdentifier with 400 (Phase 3 Sprint 7)", async ({
    request,
  }) => {
    const browserServiceUrl =
      process.env["IPO_BROWSER_URL"] ?? "http://localhost:8081";
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-results/check`,
      {
        data: {},
      },
    );
    expect(response.status()).toBe(400);
  });

  test("lottery-applications rejects missing credential fields with 400 (Phase 3 Sprint 7)", async ({
    request,
  }) => {
    const browserServiceUrl =
      process.env["IPO_BROWSER_URL"] ?? "http://localhost:8081";
    const response = await request.post(
      `${browserServiceUrl}/internal/lottery-applications/submit`,
      {
        data: {
          stockIdentifier: "01HA1234567890ABCDEFGHJKMN",
          companyName: "テスト第一株式会社",
          shares: 100,
          price: 1400,
          credential: {},
        },
      },
    );
    expect(response.status()).toBe(400);
  });
});
