import express, { type Request, type Response } from "express";

const app = express();
const port = parseInt(process.env["PORT"] ?? "8081", 10);

app.use(express.json());

app.get("/health", (_request, response) => {
  response.json({ status: "ok" });
});

// Phase 2 Sprint 3 Task 3.4 — POST /internal/accounts/test (API-013 delegate).
// ipo-api's BrowserServiceClient forwards securities-account credentials here
// so broker-specific automation can verify the account is usable before
// returning a ConnectionTestResult. Today this is a reachability check
// against the mock broker site; Phase 3 Sprint 5 will replace it with a full
// Playwright login flow against 楽天証券.
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

app.post(
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

app.listen(port, "0.0.0.0", () => {
  console.log(`ipo-browser listening on port ${port}`);
});
