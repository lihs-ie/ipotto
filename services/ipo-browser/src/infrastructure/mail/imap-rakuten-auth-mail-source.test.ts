import { describe, expect, it, vi } from "vitest";

import type { AccountCredential } from "../../domain/account-credential.js";
import {
  type ImapClientFactoryPort,
  type ImapClientPort,
  ImapRakutenAuthMailSource,
  type ImapFetchedMail,
} from "./imap-rakuten-auth-mail-source.js";

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

class FakeImapClient implements ImapClientPort {
  public readonly connect = vi.fn(async () => undefined);
  public readonly openInbox = vi.fn(async () => undefined);
  public readonly logout = vi.fn(async () => undefined);

  public constructor(
    private readonly uids: readonly number[],
    private readonly messages: ReadonlyMap<number, ImapFetchedMail>,
  ) {}

  public async search(): Promise<readonly number[]> {
    return this.uids;
  }

  public async fetchMessage(uid: number): Promise<ImapFetchedMail | null> {
    return this.messages.get(uid) ?? null;
  }
}

describe("ImapRakutenAuthMailSource", () => {
  it("returns the latest matching Rakuten authentication mail", async () => {
    const latestMessage: ImapFetchedMail = {
      source: "さくら + みかん",
      subject: "ログイン認証",
      from: ["notify@rakuten-sec.co.jp"],
      receivedAt: new Date("2026-04-11T12:00:03.000Z"),
    };
    const client = new FakeImapClient([10], new Map([[10, latestMessage]]));
    const factory: ImapClientFactoryPort = {
      create: () => client,
    };

    const source = new ImapRakutenAuthMailSource(factory);

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).resolves.toBe("さくら + みかん");
    expect(client.connect).toHaveBeenCalledOnce();
    expect(client.openInbox).toHaveBeenCalledOnce();
    expect(client.logout).toHaveBeenCalledOnce();
  });

  it("ignores messages older than the requested timestamp", async () => {
    const staleMessage: ImapFetchedMail = {
      source: "さくら + みかん",
      subject: "ログイン認証",
      from: ["notify@rakuten-sec.co.jp"],
      receivedAt: new Date("2026-04-11T11:59:59.000Z"),
    };
    const client = new FakeImapClient([10], new Map([[10, staleMessage]]));
    const source = new ImapRakutenAuthMailSource({
      create: () => client,
    });

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).resolves.toBeNull();
  });

  it("ignores non-Rakuten or non-authentication mails", async () => {
    const unrelatedMessage: ImapFetchedMail = {
      source: "さくら + みかん",
      subject: "週次レポート",
      from: ["notify@example.com"],
      receivedAt: new Date("2026-04-11T12:00:03.000Z"),
    };
    const client = new FakeImapClient([10], new Map([[10, unrelatedMessage]]));
    const source = new ImapRakutenAuthMailSource({
      create: () => client,
    });

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).resolves.toBeNull();
  });

  it("returns null when no mail exists", async () => {
    const client = new FakeImapClient([], new Map());
    const source = new ImapRakutenAuthMailSource({
      create: () => client,
    });

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).resolves.toBeNull();
  });
});
