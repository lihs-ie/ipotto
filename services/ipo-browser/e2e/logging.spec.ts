import { expect, test } from "@playwright/test";

test.describe("Structured logging smoke", () => {
  test("ipo-browser health endpoint returns structured JSON", async ({
    request,
  }) => {
    const browserServiceUrl =
      process.env["IPO_BROWSER_URL"] ?? "http://localhost:8081";
    const response = await request.get(`${browserServiceUrl}/health`);
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty("status");
  });
});
