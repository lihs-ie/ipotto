import { describe, expect, it } from "vitest";

import type {
  AccountCredential,
  ActiveSecuritiesAccount,
  ConnectionTestResult,
} from "./account-credential.js";

describe("account-credential contracts", () => {
  it("keeps the credential shape used by browser automation", () => {
    const credential: AccountCredential = {
      loginId: "login-id",
      loginPassword: "login-password",
      tradingPassword: "trading-password",
      mailCredential: {
        mailAddress: "user@example.com",
        mailPassword: "mail-password",
        imapHost: "imap.example.com",
        imapPort: 993,
      },
    };

    const account: ActiveSecuritiesAccount = {
      identifier: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      securitiesCompany: "Rakuten",
      credential,
    };

    const result: ConnectionTestResult = {
      success: true,
      message: "connected",
      testedAt: "2026-04-13T00:00:00.000Z",
    };

    expect(account.credential.mailCredential.imapPort).toBe(993);
    expect(result.success).toBe(true);
    expect(result.testedAt).toContain("2026-04-13");
  });
});
