import { describe, expect, it, vi } from "vitest";

import type { AccountCredential } from "../../domain/account-credential.js";
import {
  MailRetrievalTimeoutError,
  RakutenAuthMailParseError,
  type RakutenAuthMailSource,
} from "./rakuten-auth-mail-parser.js";
import { PollingImageAuthenticationKeywordProvider } from "./polling-image-authentication-keyword-provider.js";

const credential: AccountCredential = {
  loginId: "login",
  loginPassword: "password",
  tradingPassword: "1234",
  mailCredential: {
    mailAddress: "test@example.com",
    mailPassword: "mail-password",
    imapHost: "imap.example.com",
    imapPort: 993,
  },
};

describe("PollingImageAuthenticationKeywordProvider", () => {
  it("returns keywords from the first available mail", async () => {
    const source: RakutenAuthMailSource = {
      fetchLatestAuthenticationMail: vi.fn(
        async (): Promise<string | null> => "さくら + みかん",
      ),
    };

    const provider = new PollingImageAuthenticationKeywordProvider(
      source,
      { pollingIntervalMs: 10, timeoutMs: 100 },
      { now: () => new Date(0) },
      async () => undefined,
    );

    await expect(provider.fetchKeywords(credential)).resolves.toEqual({
      firstKeyword: "さくら",
      secondKeyword: "みかん",
    });
  });

  it("polls again when the first mail has not arrived yet", async () => {
    let attempt = 0;
    const source: RakutenAuthMailSource = {
      fetchLatestAuthenticationMail: vi.fn(async () => {
        attempt += 1;
        return attempt === 2 ? "さくら + みかん" : null;
      }),
    };

    const provider = new PollingImageAuthenticationKeywordProvider(
      source,
      { pollingIntervalMs: 10, timeoutMs: 100 },
      {
        now: () => new Date(attempt * 10),
      },
      async () => undefined,
    );

    await expect(provider.fetchKeywords(credential)).resolves.toEqual({
      firstKeyword: "さくら",
      secondKeyword: "みかん",
    });
    expect(attempt).toBe(2);
  });

  it("times out when no mail arrives within the deadline", async () => {
    let attempt = 0;
    const source: RakutenAuthMailSource = {
      fetchLatestAuthenticationMail: vi.fn(async () => {
        attempt += 1;
        return null;
      }),
    };

    const provider = new PollingImageAuthenticationKeywordProvider(
      source,
      { pollingIntervalMs: 10, timeoutMs: 25 },
      {
        now: () => new Date(attempt * 10),
      },
      async () => undefined,
    );

    await expect(provider.fetchKeywords(credential)).rejects.toThrow(
      MailRetrievalTimeoutError,
    );
  });

  it("propagates parse failures when the mail body is malformed", async () => {
    const source: RakutenAuthMailSource = {
      fetchLatestAuthenticationMail: vi.fn(async () => "キーワードがありません"),
    };

    const provider = new PollingImageAuthenticationKeywordProvider(
      source,
      { pollingIntervalMs: 10, timeoutMs: 100 },
      { now: () => new Date(0) },
      async () => undefined,
    );

    await expect(provider.fetchKeywords(credential)).rejects.toThrow(
      RakutenAuthMailParseError,
    );
  });
});
