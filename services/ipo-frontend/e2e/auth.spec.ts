import { expect, test } from "@playwright/test";

test.describe("Authentication flow", () => {
  test("login page shows Google Sign-In button", async ({ page }) => {
    await page.goto("/login");
    await expect(
      page.getByRole("button", { name: /Googleでログイン/ }),
    ).toBeVisible();
  });

  test("login page reaches the IPOtto heading", async ({ page }) => {
    await page.goto("/login");
    await expect(page.getByRole("heading", { level: 1 })).toContainText(
      "IPOtto",
    );
  });
});
