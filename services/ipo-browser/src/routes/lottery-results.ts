import { Router, type Request, type Response } from "express";

import type { BrowserManager } from "../browser/manager.js";
import {
  rakutenCheckResult,
  type CheckResultOutcome,
} from "../flows/check-result.js";
import { runImageAuthentication } from "../flows/image-auth/flow.js";
import { logger } from "../logger.js";
import {
  rakutenLogin,
  type TwoFactorHandler,
  type TwoFactorOutcome,
} from "../flows/login.js";
import { ImapMailReader } from "../mail/imap-reader.js";
import type { NotificationPublisher } from "../notifications/publisher.js";

// Phase 3 Sprint 7 — POST /internal/lottery-results/check.
// Scrapes the Rakuten Securities lottery result page for a specific
// stock and translates the label text into the LotteryResult enum
// (Won / Lost / Alternate) or `null` when the result is not yet
// published. The stub response shape `{ result: <value> | null }` is
// preserved so both `ipo-api::BrowserServiceClient` and
// `ipo-result-checker::BrowserServiceClient` continue to work without
// contract changes.

type LotteryResultCheckRequest = {
  credential: {
    loginId: string;
    loginPassword: string;
    tradingPassword: string;
    mailAddress: string;
    mailPassword: string;
    imapHost: string;
    imapPort: number;
  };
  stockIdentifier: string;
};

const credentialFields = [
  "loginId",
  "loginPassword",
  "tradingPassword",
  "mailAddress",
  "mailPassword",
  "imapHost",
  "imapPort",
] as const;

const MAIL_OTP_TIMEOUT_MS = 120_000;

export function lotteryResultsRouter(
  manager: BrowserManager,
  publisher: NotificationPublisher,
): Router {
  const router = Router();

  router.post(
    "/internal/lottery-results/check",
    async (request: Request, response: Response) => {
      const body = request.body as Partial<LotteryResultCheckRequest> | undefined;
      const validationError = validate(body);
      if (validationError !== null) {
        response.status(400).json({
          error: validationError,
        });
        return;
      }

      const validated = body as LotteryResultCheckRequest;
      const mockServerUrl =
        process.env["MOCK_SERVER_URL"] ?? "http://html-mock-server:80";
      const loginPageUrl = `${mockServerUrl}/rakuten/login_page.html?e2e-bypass=1`;
      const resultPageUrl = `${mockServerUrl}/rakuten/result_page.html`;
      const twoFactorHandler = buildTwoFactorHandler(
        validated.credential,
        publisher,
      );

      try {
        const context = await manager.acquire(validated.credential.loginId);
        const loginResult = await rakutenLogin(context, {
          loginId: validated.credential.loginId,
          password: validated.credential.loginPassword,
          loginPageUrl,
          twoFactorHandler,
        });
        if (loginResult.status === "failure") {
          await publisher.publishOperationError({
            serviceName: "ipo-browser",
            operationType: "check_lottery_result_login",
            errorMessage: loginResult.reason,
          });
          response.json({ result: null });
          return;
        }

        const resultPage = await context.newPage();
        try {
          const outcome: CheckResultOutcome = await rakutenCheckResult(
            resultPage,
            {
              stockIdentifier: validated.stockIdentifier,
              resultPageUrl,
            },
          );
          if (outcome.status === "failure") {
            await publisher.publishOperationError({
              serviceName: "ipo-browser",
              operationType: "check_lottery_result",
              errorMessage: outcome.reason,
            });
            response.json({ result: null });
            return;
          }
          response.json({ result: outcome.result });
        } finally {
          await resultPage.close().catch(() => undefined);
        }
      } catch (error) {
        const reason = error instanceof Error ? error.message : String(error);
        logger.error({
          event: "routes.lottery_results.check_failed",
          loginId: validated.credential.loginId,
          error: reason,
        });
        await publisher.publishOperationError({
          serviceName: "ipo-browser",
          operationType: "check_lottery_result",
          errorMessage: `browser automation error: ${reason}`,
        });
        response.status(502).json({
          error: `browser automation error: ${reason}`,
        });
      }
    },
  );

  return router;
}

function validate(
  body: Partial<LotteryResultCheckRequest> | undefined,
): string | null {
  if (body === undefined || body === null) {
    return "request body is required";
  }
  if (
    body.stockIdentifier === undefined ||
    body.stockIdentifier === null ||
    body.stockIdentifier === ""
  ) {
    return "stockIdentifier is required";
  }
  const credential = body.credential;
  if (credential === undefined || credential === null) {
    return "credential is required";
  }
  const missingCred = credentialFields.filter((field) => {
    const value = (credential as Record<string, unknown>)[field];
    return value === undefined || value === null || value === "";
  });
  if (missingCred.length > 0) {
    return `missing credential fields: ${missingCred.join(", ")}`;
  }
  return null;
}

function buildTwoFactorHandler(
  credential: LotteryResultCheckRequest["credential"],
  publisher: NotificationPublisher,
): TwoFactorHandler {
  const cutoff = new Date();
  const reader = new ImapMailReader({
    credential: {
      mailAddress: credential.mailAddress,
      mailPassword: credential.mailPassword,
      imapHost: credential.imapHost,
      imapPort: credential.imapPort,
    },
  });

  return async ({ page }): Promise<TwoFactorOutcome> => {
    let keywords;
    try {
      keywords = await reader.fetchImageAuthenticationKeywords(
        cutoff,
        MAIL_OTP_TIMEOUT_MS,
      );
    } catch (error) {
      const reason = error instanceof Error ? error.message : String(error);
      await publisher.publishOperationError({
        serviceName: "ipo-browser",
        operationType: "image_authentication_mail_otp",
        errorMessage: `mail OTP retrieval failed: ${reason}`,
      });
      return {
        status: "failed",
        reason: `mail OTP retrieval failed: ${reason}`,
        screenshotPath: null,
      };
    }

    const result = await runImageAuthentication(page, { keywords });
    if (result.status === "success") {
      return {
        status: "authenticated",
        message: "image authentication solved via mail OTP",
      };
    }
    await publisher.publishOperationError({
      serviceName: "ipo-browser",
      operationType: "image_authentication",
      errorMessage: result.reason,
    });
    return {
      status: "failed",
      reason: result.reason,
      screenshotPath: result.screenshotPath,
    };
  };
}
