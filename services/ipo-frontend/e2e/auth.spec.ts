import { expect, test } from "@playwright/test";

test.describe("Authentication flow", () => {
  test("login page shows Google Sign-In button", async ({ page }) => {
    await page.goto("/login");
    await expect(
      page.getByRole("button", { name: /Googleでログイン/ }),
    ).toBeVisible();
  });

  test("unauthenticated dashboard redirects to /login", async ({ page }) => {
    await page.goto("/");
    await page.waitForURL("**/login", { timeout: 10_000 });
    await expect(page).toHaveURL(/\/login$/);
  });
});
