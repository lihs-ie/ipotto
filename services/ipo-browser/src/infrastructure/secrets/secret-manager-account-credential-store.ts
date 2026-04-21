import { SecretManagerServiceClient } from "@google-cloud/secret-manager";

import type { AccountCredential } from "../../domain/account-credential.js";

interface AccountCredentialSecretPayload {
  readonly loginId: string;
  readonly loginPassword: string;
  readonly tradingPassword: string;
  readonly mailCredential: {
    readonly mailAddress: string;
    readonly mailPassword: string;
    readonly imapHost: string;
    readonly imapPort: number;
  };
}

/**
 * Secret Manager backed credential store for broker accounts.
 */
export class SecretManagerAccountCredentialStore {
  /**
   * Creates the credential store.
   */
  public constructor(
    private readonly client: SecretManagerServiceClient,
    private readonly projectId: string,
  ) {}

  /**
   * Loads an account credential from Secret Manager.
   */
  public async getAccountCredential(secretKey: string): Promise<AccountCredential> {
    let version;
    try {
      [version] = await this.client.accessSecretVersion({
        name: normalizeSecretVersionName(this.projectId, secretKey),
      });
    } catch {
      throw new Error("failed to access account credential secret");
    }

    const data = version.payload?.data;
    if (data === undefined || data === null) {
      throw new Error("account credential secret payload is empty");
    }

    const payloadText =
      typeof data === "string" ? data : Buffer.from(data).toString("utf8");
    let payload;
    try {
      payload = JSON.parse(payloadText) as AccountCredentialSecretPayload;
    } catch {
      throw new Error("account credential secret payload is invalid");
    }

    return {
      loginId: payload.loginId,
      loginPassword: payload.loginPassword,
      tradingPassword: payload.tradingPassword,
      mailCredential: {
        mailAddress: payload.mailCredential.mailAddress,
        mailPassword: payload.mailCredential.mailPassword,
        imapHost: payload.mailCredential.imapHost,
        imapPort: payload.mailCredential.imapPort,
      },
    };
  }
}

/**
 * Normalizes a secret name into a version path.
 */
function normalizeSecretVersionName(
  projectId: string,
  secretKey: string,
): string {
  if (secretKey.includes("/versions/")) {
    return secretKey;
  }
  return `projects/${projectId}/secrets/${secretKey}/versions/latest`;
}
