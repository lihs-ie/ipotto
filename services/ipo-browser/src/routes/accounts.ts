import { Router, type Request, type Response } from "express";

// Phase 2 Sprint 3 Task 3.4 / Phase 3 Sprint 5 — POST /internal/accounts/test
// (API-013 delegate). Verified against a reachability probe today; Phase 3
// Sprint 5.2+ will swap in the full Playwright login flow against 楽天証券.
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

export function accountsRouter(): Router {
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

      const mockServerUrl =
        process.env["MOCK_SERVER_URL"] ?? "http://html-mock-server:80";
      try {
        const reachability = await fetch(mockServerUrl);
        response.json({
          success: reachability.ok,
          message: reachability.ok
            ? "broker site reachable"
            : `broker site returned ${reachability.status}`,
          testedAt: new Date().toISOString(),
        });
      } catch (error) {
        const reason = error instanceof Error ? error.message : String(error);
        response.json({
          success: false,
          message: `broker site unreachable: ${reason}`,
          testedAt: new Date().toISOString(),
        });
      }
    },
  );

  return router;
}
