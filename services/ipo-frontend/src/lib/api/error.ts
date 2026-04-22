import { type ApiErrorResponse } from "@ipotto/shared";
import { type ZodError } from "zod";

/// Network-level failure (fetch threw — DNS, CORS, offline, etc.).
export type NetworkError = {
  kind: "network";
  message: string;
};

/// HTTP 4xx / 5xx with an `{ error: { code, message } }` body.
export type HttpError = {
  kind: "http";
  status: number;
  body: ApiErrorResponse;
};

/// HTTP response succeeded but its body failed Zod validation.
export type ValidationError = {
  kind: "validation";
  issues: ZodError;
};

/// HTTP response where the body could not be parsed as JSON, or the
/// server returned an error status without the expected envelope.
export type UnexpectedError = {
  kind: "unexpected";
  status?: number;
  message: string;
};

export type ApiError =
  | NetworkError
  | HttpError
  | ValidationError
  | UnexpectedError;
