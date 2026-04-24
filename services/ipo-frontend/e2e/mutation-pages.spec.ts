import { expect, test } from "@playwright/test";

/// Ensures the mutation-facing routes actually ship the login flow
/// (auth-gated routes render shell-placeholder copy server-side when
/// the client has not yet bootstrapped) rather than 404-ing or 500-ing.
test.describe("Mutation pages bootstrap", () => {
  test("login page remains reachable", async ({ page }) => {
    await page.goto("/login");
    await expect(page.getByText("IPOtto", { exact: true })).toBeVisible();
    await expect(page.getByRole("heading", { level: 1 })).toContainText(
      "ようこそ",
    );
  });

  test("api health endpoint is still wired", async ({ request }) => {
    const response = await request.get("/api/health");
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toEqual({ status: "ok" });
  });
});
