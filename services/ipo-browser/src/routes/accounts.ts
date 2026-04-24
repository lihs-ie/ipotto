import { Router, type Request, type Response } from "express";

import type { BrowserManager } from "../browser/manager.js";
import { runImageAuthentication } from "../flows/image-auth/flow.js";
import { logger } from "../logger.js";
import {
  rakutenLogin,
  type TwoFactorHandler,
  type TwoFactorOutcome,
} from "../flows/login.js";
import { ImapMailReader } from "../mail/imap-reader.js";
import type { NotificationPublisher } from "../notifications/publisher.js";

// Phase 3 Sprint 5.3 / 6.4 / 6.5 — POST /internal/accounts/test
// (API-013 delegate). Composes the 3-tier defence-in-depth contract
// from docs/03-detailed-design/acl.md §3.7:
//   1. Session restore — BrowserManager reuses launchPersistentContext
//      user-data directories so Rakuten's device token skips 2FA on
//      subsequent logins.
//   2. Mail OTP — `ImapMailReader` polls the broker's 認証 mail
//      within 120 s / 3 s cadence; the resulting keyword pair drives
//      `runImageAuthentication` which clicks the matching buttons.
//   3. Manual fallback — on any Tier-2 failure we publish an
//      OperationErrorOccurred event to /internal/pubsub/ipo-notification
//      so the domain dispatcher fans out LINE / SendGrid / Slack
//      notifications. Per the ACL spec we never retry image-auth
//      automatically (3 consecutive failures lock the broker account).

type ConnectionTestRequest = {
  loginId: string;
  loginPassword: string;
  tradingPassword: string;
  mailAddress: string;
  mailPassword: string;
  imapHost: string;
  imapPort: number;
};

const credentialFields = [
  "loginId",
  "loginPassword",
  "tradingPassword",
  "mailAddress",
  "mailPassword",
  "imapHost",
  "imapPort",
] as const satisfies ReadonlyArray<keyof ConnectionTestRequest>;

const MAIL_OTP_TIMEOUT_MS = 120_000;

export function accountsRouter(
  manager: BrowserManager,
  publisher: NotificationPublisher,
): Router {
  const router = Router();

  router.post(
    "/internal/accounts/test",
    async (request: Request, response: Response) => {
      const body = request.body as Partial<ConnectionTestRequest> | undefined;
      const missing = credentialFields.filter((field) => {
        const value = body?.[field];
        return value === undefined || value === null || value === "";
      });
      if (missing.length > 0) {
        response.status(400).json({
          success: false,
          message: `missing credential fields: ${missing.join(", ")}`,
          testedAt: new Date().toISOString(),
        });
        return;
      }

      const validated = body as ConnectionTestRequest;
      const mockServerUrl =
        process.env["MOCK_SERVER_URL"] ?? "http://html-mock-server:80";
      const loginPageUrl = `${mockServerUrl}/rakuten/login_page.html`;
      const twoFactorHandler = buildTwoFactorHandler(validated, publisher);

      try {
        const context = await manager.acquire(validated.loginId);
        const result = await rakutenLogin(context, {
          loginId: validated.loginId,
          password: validated.loginPassword,
          loginPageUrl,
          twoFactorHandler,
        });
        if (result.status === "success") {
          response.json({
            success: true,
            message: result.message,
            testedAt: new Date().toISOString(),
          });
        } else {
          response.json({
            success: false,
            message: result.reason,
            testedAt: new Date().toISOString(),
          });
        }
      } catch (error) {
        const reason = error instanceof Error ? error.message : String(error);
        logger.error({
          event: "routes.accounts.automation_failed",
          loginId: validated.loginId,
          error: reason,
        });
        response.status(502).json({
          success: false,
          message: `browser automation error: ${reason}`,
          testedAt: new Date().toISOString(),
        });
      }
    },
  );

  return router;
}

function buildTwoFactorHandler(
  credential: ConnectionTestRequest,
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
