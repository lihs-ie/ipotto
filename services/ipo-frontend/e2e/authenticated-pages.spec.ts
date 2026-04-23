import { test, expect } from "./fixtures";

test.describe("Authenticated API via frontend stack", () => {
  test("GET /api/v1/dashboard returns valid summary", async ({
    idToken,
    apiBaseUrl,
    request,
  }) => {
    const response = await request.get(`${apiBaseUrl}/api/v1/dashboard`, {
      headers: { authorization: `Bearer ${idToken}` },
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty("statusCounts");
    expect(body).toHaveProperty("recentActivities");
    expect(body).toHaveProperty("upcomingStocks");
    expect(body).toHaveProperty("systemStatus");
  });

  test("GET /api/v1/stocks returns items array", async ({
    idToken,
    apiBaseUrl,
    request,
  }) => {
    const response = await request.get(`${apiBaseUrl}/api/v1/stocks`, {
      headers: { authorization: `Bearer ${idToken}` },
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty("items");
    expect(body).toHaveProperty("totalCount");
  });

  test("GET /api/v1/exclusions returns items array", async ({
    idToken,
    apiBaseUrl,
    request,
  }) => {
    const response = await request.get(`${apiBaseUrl}/api/v1/exclusions`, {
      headers: { authorization: `Bearer ${idToken}` },
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty("items");
    expect(body).toHaveProperty("totalCount");
  });

  test("GET /api/v1/accounts returns items array", async ({
    idToken,
    apiBaseUrl,
    request,
  }) => {
    const response = await request.get(`${apiBaseUrl}/api/v1/accounts`, {
      headers: { authorization: `Bearer ${idToken}` },
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty("items");
    expect(body).toHaveProperty("totalCount");
  });

  test("GET /api/v1/notifications/settings returns setting", async ({
    idToken,
    apiBaseUrl,
    request,
  }) => {
    const response = await request.get(
      `${apiBaseUrl}/api/v1/notifications/settings`,
      { headers: { authorization: `Bearer ${idToken}` } },
    );
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty("identifier");
    expect(body).toHaveProperty("enabled");
    expect(body).toHaveProperty("channels");
  });

  test("GET /api/v1/logs returns paginated list", async ({
    idToken,
    apiBaseUrl,
    request,
  }) => {
    const response = await request.get(`${apiBaseUrl}/api/v1/logs`, {
      headers: { authorization: `Bearer ${idToken}` },
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty("items");
    expect(body).toHaveProperty("hasMore");
  });

  test("unauthenticated request returns 401", async ({
    apiBaseUrl,
    request,
  }) => {
    const response = await request.get(`${apiBaseUrl}/api/v1/stocks`);
    expect(response.status()).toBe(401);
  });
});
