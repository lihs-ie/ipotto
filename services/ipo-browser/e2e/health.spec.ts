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
});
