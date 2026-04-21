import express, { type Express, type Request, type Response } from "express";

import type { AccountCredential } from "./domain/account-credential.js";
import type { DependencyContainer } from "./infrastructure/dependency-container.js";
import { createDependencyContainer } from "./infrastructure/dependency-container.js";
import { createApplyMessageHandler } from "./presentation/handlers/pubsub-message-handler.js";

/**
 * Creates the Express application.
 */
export function createApp(
  container: DependencyContainer = createDependencyContainer(),
): Express {
  const app = express();

  app.use(express.json());
  app.get("/health", (_request, response) => {
    response.json({ status: "ok" });
  });

  app.get("/internal/stocks", async (_request, response, next) => {
    try {
      response.json(await container.stockCatalogClient().fetchStocks());
    } catch (error) {
      next(error);
    }
  });

  app.post("/internal/accounts/test", async (request, response, next) => {
    try {
      const credential = parseCredentialRequest(request.body);
      response.json(await container.brokerPort().testConnection(credential));
    } catch (error) {
      next(error);
    }
  });

  app.post(
    "/internal/lottery-results/check",
    async (request: Request, response: Response, next) => {
      try {
        const credential = parseCredentialRequest(request.body["credential"]);
        const stockIdentifier = parseStockIdentifier(request.body);
        response.json({
          result: await container
            .brokerPort()
            .checkLotteryResult(credential, stockIdentifier),
        });
      } catch (error) {
        next(error);
      }
    },
  );

  app.post(
    "/internal/pubsub/apply",
    createApplyMessageHandler(container),
  );

  app.use((error: unknown, _request: Request, response: Response, next: () => void) => {
    void next;
    const message = error instanceof Error ? error.message : "unexpected error";
    response.status(500).json({
      code: "INTERNAL_SERVER_ERROR",
      message,
    });
  });

  return app;
}

/**
 * Parses a broker credential request body.
 */
function parseCredentialRequest(body: unknown): AccountCredential {
  if (typeof body !== "object" || body === null) {
    throw new Error("credential is required");
  }

  const record = body as Record<string, unknown>;
  const mailCredential = record["mailCredential"];
  const hasInlineMailCredential =
    typeof mailCredential === "object" && mailCredential !== null;

  const mailSource = hasInlineMailCredential
    ? (mailCredential as Record<string, unknown>)
    : record;

  return {
    loginId: requiredString(record["loginId"], "loginId"),
    loginPassword: requiredString(record["loginPassword"], "loginPassword"),
    tradingPassword: requiredString(record["tradingPassword"], "tradingPassword"),
    mailCredential: {
      mailAddress: requiredString(mailSource["mailAddress"], "mailAddress"),
      mailPassword: requiredString(mailSource["mailPassword"], "mailPassword"),
      imapHost: requiredString(mailSource["imapHost"], "imapHost"),
      imapPort: requiredNumber(mailSource["imapPort"], "imapPort"),
    },
  };
}

/**
 * Parses a stock identifier from the lottery result request.
 */
function parseStockIdentifier(body: unknown): string {
  if (typeof body !== "object" || body === null) {
    throw new Error("stockIdentifier is required");
  }
  return requiredString(
    (body as Record<string, unknown>)["stockIdentifier"],
    "stockIdentifier",
  );
}

/**
 * Requires a string value.
 */
function requiredString(value: unknown, fieldName: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${fieldName} is required`);
  }
  return value;
}

/**
 * Requires a number value.
 */
function requiredNumber(value: unknown, fieldName: string): number {
  if (typeof value !== "number" || Number.isNaN(value)) {
    throw new Error(`${fieldName} is required`);
  }
  return value;
}
