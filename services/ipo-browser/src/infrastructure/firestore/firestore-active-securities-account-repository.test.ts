import { describe, expect, it, vi } from "vitest";

import { FirestoreActiveSecuritiesAccountRepository } from "./firestore-active-securities-account-repository.js";

describe("FirestoreActiveSecuritiesAccountRepository", () => {
  it("restores credentials from Secret Manager for active accounts", async () => {
    const get = vi.fn().mockResolvedValue({
      docs: [
        {
          data: () => ({
            identifier: "account-1",
            securitiesCompany: "Rakuten",
            credentialSecretKey: "secret-1",
          }),
        },
      ],
    });
    const where = vi.fn().mockReturnValue({ get });
    const collection = vi.fn().mockReturnValue({ where });
    const firestore = { collection } as const;
    const credentialStore = {
      getAccountCredential: vi.fn().mockResolvedValue({
        loginId: "login-id",
        loginPassword: "login-password",
        tradingPassword: "trading-password",
        mailCredential: {
          mailAddress: "user@example.com",
          mailPassword: "mail-password",
          imapHost: "imap.example.com",
          imapPort: 993,
        },
      }),
    };

    const repository = new FirestoreActiveSecuritiesAccountRepository(
      firestore as never,
      credentialStore as never,
    );

    await expect(repository.findActive()).resolves.toEqual([
      {
        identifier: "account-1",
        securitiesCompany: "Rakuten",
        credential: {
          loginId: "login-id",
          loginPassword: "login-password",
          tradingPassword: "trading-password",
          mailCredential: {
            mailAddress: "user@example.com",
            mailPassword: "mail-password",
            imapHost: "imap.example.com",
            imapPort: 993,
          },
        },
      },
    ]);
    expect(collection).toHaveBeenCalledWith("securities_accounts");
    expect(where).toHaveBeenCalledWith("activation.isActive", "==", true);
    expect(credentialStore.getAccountCredential).toHaveBeenCalledWith("secret-1");
  });
});
