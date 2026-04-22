import { expect, test } from "@playwright/test";

test.describe("Structured logging smoke", () => {
  test("health endpoint returns structured JSON that a logger consumer can parse", async ({
    request,
  }) => {
    const response = await request.get("/health");
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty("status");
  });
});
