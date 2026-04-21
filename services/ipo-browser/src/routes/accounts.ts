import { Router, type Request, type Response } from "express";

import type { BrowserManager } from "../browser/manager.js";
import { rakutenLogin } from "../flows/login.js";

// Phase 3 Sprint 5.3 — POST /internal/accounts/test (API-013 delegate).
// Drives the Rakuten login flow through Playwright against the broker
// site provided by the `MOCK_SERVER_URL` env var (defaults to the
// docker-compose html-mock-server fixture). Phase 3 Sprint 7.3 will
// factor the stand-alone connection-test flow out of this endpoint;
// for now success is: login form interaction completed without
// throwing.

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

export function accountsRouter(manager: BrowserManager): Router {
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

      try {
        const context = await manager.acquire(validated.loginId);
        const result = await rakutenLogin(context, {
          loginId: validated.loginId,
          password: validated.loginPassword,
          loginPageUrl,
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
        console.error(
          `browser automation error (loginId=${validated.loginId})`,
          error,
        );
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
