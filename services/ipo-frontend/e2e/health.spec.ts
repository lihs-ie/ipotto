import { test, expect } from "@playwright/test";

test.describe("Health Check", () => {
  test("API health endpoint returns ok", async ({ request }) => {
    const response = await request.get("/api/health");
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ status: "ok" });
  });

  test("login page loads with heading", async ({ page }) => {
    await page.goto("/login");
    await expect(page.locator("h1")).toBeVisible();
  });
});
