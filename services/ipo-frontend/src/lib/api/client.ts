import { apiErrorSchema } from "@ipotto/shared";
import { z, type ZodTypeAny } from "zod";

import { type ApiError } from "./error";
import { type AsyncResult, err, ok } from "./result";

type GetIdToken = () => Promise<string | null>;

export type ApiClientOptions = {
  baseUrl: string;
  getIdToken: GetIdToken;
  fetchImpl?: typeof fetch;
};

type RequestInitWithoutMethod = Omit<RequestInit, "method" | "body">;

type JsonRequestInit = {
  body?: unknown;
  query?: Record<string, string | number | boolean | undefined>;
} & RequestInitWithoutMethod;

export type ApiClient = {
  get: <S extends ZodTypeAny>(
    path: string,
    schema: S,
    init?: JsonRequestInit,
  ) => AsyncResult<z.output<S>, ApiError>;
  post: <S extends ZodTypeAny>(
    path: string,
    schema: S,
    init?: JsonRequestInit,
  ) => AsyncResult<z.output<S>, ApiError>;
  put: <S extends ZodTypeAny>(
    path: string,
    schema: S,
    init?: JsonRequestInit,
  ) => AsyncResult<z.output<S>, ApiError>;
  delete: (
    path: string,
    init?: JsonRequestInit,
  ) => AsyncResult<null, ApiError>;
};

const buildUrl = (
  baseUrl: string,
  path: string,
  query?: Record<string, string | number | boolean | undefined>,
): string => {
  const url = new URL(path, baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`);
  if (query) {
    for (const [key, rawValue] of Object.entries(query)) {
      if (rawValue === undefined) continue;
      url.searchParams.set(key, String(rawValue));
    }
  }
  return url.toString();
};

const decodeError = async (response: Response): Promise<ApiError> => {
  let bodyText: string;
  try {
    bodyText = await response.text();
  } catch (caught) {
    return {
      kind: "unexpected",
      status: response.status,
      message:
        caught instanceof Error
          ? caught.message
          : "failed to read error body",
    };
  }
  if (bodyText.trim() === "") {
    return {
      kind: "unexpected",
      status: response.status,
      message: `empty body for status ${response.status}`,
    };
  }
  try {
    const parsed = apiErrorSchema.parse(JSON.parse(bodyText));
    return { kind: "http", status: response.status, body: parsed };
  } catch (caught) {
    return {
      kind: "unexpected",
      status: response.status,
      message:
        caught instanceof Error ? caught.message : `unrecognized error body`,
    };
  }
};

export const createApiClient = (options: ApiClientOptions): ApiClient => {
  const fetchImpl = options.fetchImpl ?? fetch;

  const request = async <S extends ZodTypeAny>(
    method: string,
    path: string,
    schema: S,
    init?: JsonRequestInit,
  ): AsyncResult<z.output<S>, ApiError> => {
    const headers = new Headers(init?.headers);
    if (init?.body !== undefined) {
      headers.set("content-type", "application/json");
    }
    const idToken = await options.getIdToken();
    if (idToken) {
      headers.set("authorization", `Bearer ${idToken}`);
    }
    const url = buildUrl(options.baseUrl, path, init?.query);

    let response: Response;
    try {
      response = await fetchImpl(url, {
        method,
        headers,
        body: init?.body !== undefined ? JSON.stringify(init.body) : undefined,
      });
    } catch (caught) {
      return err({
        kind: "network",
        message: caught instanceof Error ? caught.message : "network failure",
      });
    }

    if (!response.ok) {
      return err(await decodeError(response));
    }

    if (response.status === 204) {
      const parsed = schema.safeParse(null);
      if (parsed.success) return ok(parsed.data);
      return err({ kind: "validation", issues: parsed.error });
    }

    let raw: unknown;
    try {
      raw = await response.json();
    } catch (caught) {
      return err({
        kind: "unexpected",
        status: response.status,
        message:
          caught instanceof Error
            ? caught.message
            : "failed to parse success body",
      });
    }

    const parsed = schema.safeParse(raw);
    if (!parsed.success) {
      return err({ kind: "validation", issues: parsed.error });
    }
    return ok(parsed.data);
  };

  return {
    get: (path, schema, init) => request("GET", path, schema, init),
    post: (path, schema, init) => request("POST", path, schema, init),
    put: (path, schema, init) => request("PUT", path, schema, init),
    delete: async (path, init) => {
      const headers = new Headers(init?.headers);
      const idToken = await options.getIdToken();
      if (idToken) headers.set("authorization", `Bearer ${idToken}`);
      const url = buildUrl(options.baseUrl, path, init?.query);
      let response: Response;
      try {
        response = await fetchImpl(url, { method: "DELETE", headers });
      } catch (caught) {
        return err({
          kind: "network",
          message:
            caught instanceof Error ? caught.message : "network failure",
        });
      }
      if (!response.ok) {
        return err(await decodeError(response));
      }
      return ok(null);
    },
  };
};
