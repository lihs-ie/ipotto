import type { RequestHandler } from "express";

import {
  ApplyForLotteryValidationError,
} from "../../application/use-cases/apply-for-lottery-use-case.js";
import type { DependencyContainer } from "../../infrastructure/dependency-container.js";

interface PubSubPushEnvelope {
  readonly message: {
    readonly data: string;
  };
}

class ApplyMessageValidationError extends Error {
  /**
   * Creates the request validation error.
   */
  public constructor(message: string) {
    super(message);
    this.name = "ApplyMessageValidationError";
  }
}

/**
 * Creates the apply Pub/Sub handler.
 */
export function createApplyMessageHandler(
  container: DependencyContainer,
): RequestHandler {
  return async (request, response, next) => {
    try {
      const input = parseApplyMessageBody(request.body);
      const result = await container.applyForLotteryUseCase().execute(input);
      response.status(200).json(result);
    } catch (error) {
      if (
        error instanceof ApplyMessageValidationError ||
        error instanceof ApplyForLotteryValidationError
      ) {
        response.status(400).json({
          code: "BAD_REQUEST",
          message: error.message,
        });
        return;
      }

      if (error instanceof Error) {
        response.status(503).json({
          code: "SERVICE_UNAVAILABLE",
          message: "apply workflow failed before completion",
        });
        return;
      }

      next(error);
    }
  };
}

/**
 * Parses an apply message body from either raw JSON or Pub/Sub push format.
 */
export function parseApplyMessageBody(
  body: unknown,
): { readonly targetDate: string } {
  if (isPubSubEnvelope(body)) {
    try {
      const json = JSON.parse(
        Buffer.from(body.message.data, "base64").toString("utf8"),
      ) as unknown;
      if (isRecord(json)) {
        return parseApplyRequestPayload(json);
      }
    } catch {
      throw new ApplyMessageValidationError("Pub/Sub message data must be valid JSON");
    }
  }

  if (isRecord(body)) {
    return parseApplyRequestPayload(body);
  }

  throw new ApplyMessageValidationError("targetDate is required");
}

/**
 * Parses and validates the concrete apply payload.
 */
function parseApplyRequestPayload(
  payload: Record<string, unknown>,
): { readonly targetDate: string } {
  if ("accountIds" in payload) {
    throw new ApplyMessageValidationError("accountIds is not supported yet");
  }
  if ("stockIds" in payload) {
    throw new ApplyMessageValidationError("stockIds is not supported yet");
  }
  if (typeof payload["targetDate"] !== "string") {
    throw new ApplyMessageValidationError("targetDate is required");
  }

  return {
    targetDate: validateTargetDate(payload["targetDate"]),
  };
}

/**
 * Validates a target date string.
 */
function validateTargetDate(targetDate: string): string {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(targetDate)) {
    throw new ApplyMessageValidationError("targetDate must be in YYYY-MM-DD format");
  }
  return targetDate;
}

/**
 * Returns whether the value is a string-keyed record.
 */
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/**
 * Returns whether the value is a Pub/Sub push envelope.
 */
function isPubSubEnvelope(value: unknown): value is PubSubPushEnvelope {
  return (
    isRecord(value) &&
    isRecord(value["message"]) &&
    typeof value["message"]["data"] === "string"
  );
}
