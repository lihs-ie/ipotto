import { test, expect } from "@playwright/test";

test.describe("Login page", () => {
  test("shows Google Sign-In button", async ({ page }) => {
    await page.goto("/login");
    await expect(
      page.getByRole("button", { name: /Googleでログイン/ }),
    ).toBeVisible();
  });

  test("shows IPOtto branding and welcome heading", async ({ page }) => {
    await page.goto("/login");
    await expect(page.getByText("IPOtto", { exact: true })).toBeVisible();
    await expect(page.getByRole("heading", { level: 1 })).toContainText(
      "ようこそ",
    );
  });

  test("unauthenticated root redirects to login", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveURL(/\/login/, { timeout: 10_000 });
  });

  test("unauthenticated /stocks redirects to login", async ({ page }) => {
    await page.goto("/stocks");
    await expect(page).toHaveURL(/\/login/, { timeout: 10_000 });
  });

  test("unauthenticated /exclusions redirects to login", async ({ page }) => {
    await page.goto("/exclusions");
    await expect(page).toHaveURL(/\/login/, { timeout: 10_000 });
  });

  test("unauthenticated /accounts redirects to login", async ({ page }) => {
    await page.goto("/accounts");
    await expect(page).toHaveURL(/\/login/, { timeout: 10_000 });
  });

  test("unauthenticated /logs redirects to login", async ({ page }) => {
    await page.goto("/logs");
    await expect(page).toHaveURL(/\/login/, { timeout: 10_000 });
  });
});
