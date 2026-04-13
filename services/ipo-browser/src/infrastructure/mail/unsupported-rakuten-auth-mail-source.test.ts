import { describe, expect, it } from "vitest";

import { UnsupportedRakutenAuthMailSource } from "./unsupported-rakuten-auth-mail-source.js";

describe("UnsupportedRakutenAuthMailSource", () => {
  it("always throws because no concrete mail transport is configured", async () => {
    const source = new UnsupportedRakutenAuthMailSource();

    await expect(
      source.fetchLatestAuthenticationMail(
        {
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
        new Date("2026-04-13T00:00:00.000Z"),
      ),
    ).rejects.toThrow("Rakuten authentication mail source is not configured");
  });
});
