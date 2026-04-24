import { expect, test } from "@playwright/test";

test.describe("Authentication flow", () => {
  test("login page shows Google Sign-In button", async ({ page }) => {
    await page.goto("/login");
    await expect(
      page.getByRole("button", { name: /Googleでログイン/ }),
    ).toBeVisible();
  });

  test("login page shows IPOtto branding and welcome heading", async ({
    page,
  }) => {
    await page.goto("/login");
    await expect(page.getByText("IPOtto", { exact: true })).toBeVisible();
    await expect(page.getByRole("heading", { level: 1 })).toContainText(
      "ようこそ",
    );
  });
});
