import type { RequestHandler } from "express";

import type { DependencyContainer } from "../../infrastructure/dependency-container.js";

interface PubSubPushEnvelope {
  readonly message: {
    readonly data: string;
  };
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
      if (error instanceof Error) {
        response.status(400).json({
          code: "BAD_REQUEST",
          message: error.message,
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
  if (isRecord(body) && typeof body["targetDate"] === "string") {
    return {
      targetDate: validateTargetDate(body["targetDate"]),
    };
  }

  if (isPubSubEnvelope(body)) {
    const json = JSON.parse(
      Buffer.from(body.message.data, "base64").toString("utf8"),
    ) as unknown;
    if (isRecord(json) && typeof json["targetDate"] === "string") {
      return {
        targetDate: validateTargetDate(json["targetDate"]),
      };
    }
  }

  throw new Error("targetDate is required");
}

/**
 * Validates a target date string.
 */
function validateTargetDate(targetDate: string): string {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(targetDate)) {
    throw new Error("targetDate must be in YYYY-MM-DD format");
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
