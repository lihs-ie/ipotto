import { z } from "zod";
import { describe, expect, it, vi } from "vitest";

import { createApiClient } from "./client";

const stubResponse = (init: {
  status: number;
  json?: unknown;
  text?: string;
}): Response => {
  const bodyText = init.json !== undefined ? JSON.stringify(init.json) : init.text ?? "";
  return new Response(bodyText, {
    status: init.status,
    headers: { "content-type": "application/json" },
  });
};

describe("createApiClient", () => {
  const responseSchema = z.object({ ok: z.boolean() });

  it("parses a successful response and returns Ok(value)", async () => {
    const fetchImpl = vi.fn().mockResolvedValue(
      stubResponse({ status: 200, json: { ok: true } }),
    );
    const client = createApiClient({
      baseUrl: "http://api.test",
      getIdToken: async () => "token",
      fetchImpl,
    });
    const result = await client.get("/resource", responseSchema);
    expect(result.ok).toBe(true);
    if (!result.ok) throw new Error("expected ok result");
    expect(result.value).toEqual({ ok: true });
  });

  it("attaches the bearer token from getIdToken", async () => {
    const fetchImpl = vi.fn().mockResolvedValue(
      stubResponse({ status: 200, json: { ok: true } }),
    );
    const client = createApiClient({
      baseUrl: "http://api.test",
      getIdToken: async () => "id-token-xyz",
      fetchImpl,
    });
    await client.get("/resource", responseSchema);
    const init = fetchImpl.mock.calls[0]?.[1] as RequestInit;
    const headers = new Headers(init.headers);
    expect(headers.get("authorization")).toBe("Bearer id-token-xyz");
  });

  it("omits the authorization header when getIdToken returns null", async () => {
    const fetchImpl = vi.fn().mockResolvedValue(
      stubResponse({ status: 200, json: { ok: true } }),
    );
    const client = createApiClient({
      baseUrl: "http://api.test",
      getIdToken: async () => null,
      fetchImpl,
    });
    await client.get("/resource", responseSchema);
    const init = fetchImpl.mock.calls[0]?.[1] as RequestInit;
    const headers = new Headers(init.headers);
    expect(headers.has("authorization")).toBe(false);
  });

  it("maps HTTP 404 with error envelope to Err(HttpError)", async () => {
    const fetchImpl = vi.fn().mockResolvedValue(
      stubResponse({
        status: 404,
        json: { error: { code: "NOT_FOUND", message: "missing" } },
      }),
    );
    const client = createApiClient({
      baseUrl: "http://api.test",
      getIdToken: async () => null,
      fetchImpl,
    });
    const result = await client.get("/missing", responseSchema);
    expect(result.ok).toBe(false);
    if (result.ok) throw new Error("expected error result");
    expect(result.error.kind).toBe("http");
    if (result.error.kind !== "http") throw new Error("unreachable");
    expect(result.error.status).toBe(404);
    expect(result.error.body.error.code).toBe("NOT_FOUND");
  });

  it("maps thrown fetch errors to NetworkError", async () => {
    const fetchImpl = vi.fn().mockRejectedValue(new TypeError("offline"));
    const client = createApiClient({
      baseUrl: "http://api.test",
      getIdToken: async () => null,
      fetchImpl,
    });
    const result = await client.get("/ping", responseSchema);
    expect(result.ok).toBe(false);
    if (result.ok) throw new Error("expected error");
    expect(result.error.kind).toBe("network");
  });

  it("maps Zod validation failure to ValidationError", async () => {
    const fetchImpl = vi.fn().mockResolvedValue(
      stubResponse({ status: 200, json: { ok: "yes" } }),
    );
    const client = createApiClient({
      baseUrl: "http://api.test",
      getIdToken: async () => null,
      fetchImpl,
    });
    const result = await client.get("/resource", responseSchema);
    expect(result.ok).toBe(false);
    if (result.ok) throw new Error("expected error");
    expect(result.error.kind).toBe("validation");
  });

  it("serializes request bodies as JSON and sets content-type", async () => {
    const fetchImpl = vi.fn().mockResolvedValue(
      stubResponse({ status: 200, json: { ok: true } }),
    );
    const client = createApiClient({
      baseUrl: "http://api.test",
      getIdToken: async () => null,
      fetchImpl,
    });
    await client.post("/resource", responseSchema, {
      body: { companyName: "テスト株式会社" },
    });
    const init = fetchImpl.mock.calls[0]?.[1] as RequestInit;
    const headers = new Headers(init.headers);
    expect(headers.get("content-type")).toBe("application/json");
    expect(init.body).toBe(JSON.stringify({ companyName: "テスト株式会社" }));
  });

  it("appends query params to the URL", async () => {
    const fetchImpl = vi.fn().mockResolvedValue(
      stubResponse({ status: 200, json: { ok: true } }),
    );
    const client = createApiClient({
      baseUrl: "http://api.test",
      getIdToken: async () => null,
      fetchImpl,
    });
    await client.get("/resource", responseSchema, {
      query: { status: "Eligible" },
    });
    const url = fetchImpl.mock.calls[0]?.[0] as string;
    expect(url).toBe("http://api.test/resource?status=Eligible");
  });

  it("returns Ok(null) on 204 when schema accepts null", async () => {
    const fetchImpl = vi.fn().mockResolvedValue(
      new Response(null, { status: 204 }),
    );
    const client = createApiClient({
      baseUrl: "http://api.test",
      getIdToken: async () => null,
      fetchImpl,
    });
    const result = await client.delete("/resource/1");
    expect(result.ok).toBe(true);
  });
});
