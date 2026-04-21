import { describe, expect, it, vi } from "vitest";
import type { SecretManagerServiceClient } from "@google-cloud/secret-manager";

import { SecretManagerAccountCredentialStore } from "./secret-manager-account-credential-store.js";

describe("SecretManagerAccountCredentialStore", () => {
  it("returns account credentials from a valid secret payload", async () => {
    const accessSecretVersion = vi.fn(async () => [
      {
        payload: {
          data: Buffer.from(
            JSON.stringify({
              loginId: "login-id",
              loginPassword: "login-password",
              tradingPassword: "trading-password",
              mailCredential: {
                mailAddress: "test@example.com",
                mailPassword: "mail-password",
                imapHost: "imap.example.com",
                imapPort: 993,
              },
            }),
            "utf8",
          ),
        },
      },
    ]);

    const store = new SecretManagerAccountCredentialStore(
      { accessSecretVersion } as unknown as SecretManagerServiceClient,
      "ipotto-local",
    );

    await expect(store.getAccountCredential("ipo-account-account-1")).resolves.toEqual({
      loginId: "login-id",
      loginPassword: "login-password",
      tradingPassword: "trading-password",
      mailCredential: {
        mailAddress: "test@example.com",
        mailPassword: "mail-password",
        imapHost: "imap.example.com",
        imapPort: 993,
      },
    });
  });

  it("does not leak secret payload contents when parsing fails", async () => {
    const accessSecretVersion = vi.fn(async () => [
      {
        payload: {
          data: Buffer.from('{"loginPassword":"super-secret"', "utf8"),
        },
      },
    ]);

    const store = new SecretManagerAccountCredentialStore(
      { accessSecretVersion } as unknown as SecretManagerServiceClient,
      "ipotto-local",
    );

    await expect(store.getAccountCredential("ipo-account-account-1")).rejects.toThrow(
      "account credential secret payload is invalid",
    );
  });

  it("does not leak secret identifiers when access fails", async () => {
    const accessSecretVersion = vi.fn(async () => {
      throw new Error("permission denied for projects/x/secrets/ipo-account-account-1");
    });

    const store = new SecretManagerAccountCredentialStore(
      { accessSecretVersion } as unknown as SecretManagerServiceClient,
      "ipotto-local",
    );

    await expect(store.getAccountCredential("ipo-account-account-1")).rejects.toThrow(
      "failed to access account credential secret",
    );
  });
});
