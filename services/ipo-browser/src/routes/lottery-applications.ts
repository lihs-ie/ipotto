import { Router, type Request, type Response } from "express";

import type { BrowserManager } from "../browser/manager.js";
import { rakutenApply, type ApplyResult } from "../flows/apply.js";
import { runImageAuthentication } from "../flows/image-auth/flow.js";
import { logger } from "../logger.js";
import {
  rakutenLogin,
  type TwoFactorHandler,
  type TwoFactorOutcome,
} from "../flows/login.js";
import { ImapMailReader } from "../mail/imap-reader.js";
import type { NotificationPublisher } from "../notifications/publisher.js";

// Phase 3 Sprint 7 — POST /internal/lottery-applications/submit.
// Carries out the full "login → (2FA) → navigate to apply list →
// submit apply form → classify result" sequence against the Rakuten
// Securities site (html-mock-server in CI / e2e). The ipo-api side's
// `BrowserServiceClient::apply_for_ipo` calls this route with the
// BrowserApplyRequest shape below and expects one of the four
// ApplicationResult variants in return; failures and unrecognised
// result pages publish an OperationErrorOccurred event to ipo-api's
// notification dispatcher (SEV1), while `already_applied` stays a
// normal-flow response.

type SubmitLotteryApplicationRequest = {
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
  companyName: string;
  shares: number;
  price: number;
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

const topLevelFields = [
  "stockIdentifier",
  "companyName",
  "shares",
  "price",
] as const;

const MAIL_OTP_TIMEOUT_MS = 120_000;

export function lotteryApplicationsRouter(
  manager: BrowserManager,
  publisher: NotificationPublisher,
): Router {
  const router = Router();

  router.post(
    "/internal/lottery-applications/submit",
    async (request: Request, response: Response) => {
      const body = request.body as Partial<SubmitLotteryApplicationRequest> | undefined;
      const validationError = validate(body);
      if (validationError !== null) {
        response.status(400).json({
          status: "failure",
          reason: validationError,
        });
        return;
      }

      const validated = body as SubmitLotteryApplicationRequest;
      const mockServerUrl =
        process.env["MOCK_SERVER_URL"] ?? "http://html-mock-server:80";
      const loginPageUrl = `${mockServerUrl}/rakuten/login_page.html?e2e-bypass=1`;
      const applyListUrl = `${mockServerUrl}/rakuten/apply_list_page.html`;
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
            operationType: "application_submit_login",
            errorMessage: loginResult.reason,
          });
          response.json({
            status: "failure",
            reason: loginResult.reason,
            screenshotPath: loginResult.screenshotPath,
          });
          return;
        }

        const applyPage = await context.newPage();
        try {
          const applyResult: ApplyResult = await rakutenApply(applyPage, {
            stockIdentifier: validated.stockIdentifier,
            companyName: validated.companyName,
            shares: validated.shares,
            price: validated.price,
            tradingPassword: validated.credential.tradingPassword,
            applyListUrl,
          });

          if (applyResult.status === "failure") {
            await publisher.publishOperationError({
              serviceName: "ipo-browser",
              operationType: "application_submit",
              errorMessage: applyResult.reason,
            });
          } else if (applyResult.status === "insufficient_balance") {
            await publisher.publishOperationError({
              serviceName: "ipo-browser",
              operationType: "application_submit",
              errorMessage: "broker site reported insufficient balance",
            });
          }

          response.json(applyResult);
        } finally {
          await applyPage.close().catch(() => undefined);
        }
      } catch (error) {
        const reason = error instanceof Error ? error.message : String(error);
        logger.error({
          event: "routes.lottery_applications.submit_failed",
          loginId: validated.credential.loginId,
          error: reason,
        });
        await publisher.publishOperationError({
          serviceName: "ipo-browser",
          operationType: "application_submit",
          errorMessage: `browser automation error: ${reason}`,
        });
        response.status(502).json({
          status: "failure",
          reason: `browser automation error: ${reason}`,
        });
      }
    },
  );

  return router;
}

function validate(
  body: Partial<SubmitLotteryApplicationRequest> | undefined,
): string | null {
  if (body === undefined || body === null) {
    return "request body is required";
  }

  const missingTop = topLevelFields.filter((field) => {
    const value = body[field];
    return value === undefined || value === null || value === "";
  });
  if (missingTop.length > 0) {
    return `missing fields: ${missingTop.join(", ")}`;
  }

  if (typeof body.shares !== "number" || !Number.isFinite(body.shares) || body.shares <= 0) {
    return "shares must be a positive number";
  }
  if (typeof body.price !== "number" || !Number.isFinite(body.price) || body.price < 0) {
    return "price must be a non-negative number";
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
  credential: SubmitLotteryApplicationRequest["credential"],
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
