import { describe, expect, it, vi } from "vitest";
import type { SearchObject } from "imapflow";

import type { AccountCredential } from "../../domain/account-credential.js";
import {
  type ImapClientFactoryPort,
  type ImapClientPort,
  ImapRakutenAuthMailSource,
  type ImapFetchedMail,
} from "./imap-rakuten-auth-mail-source.js";
import { RakutenAuthMailSourceError } from "./rakuten-auth-mail-parser.js";

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
  public readonly connect = vi.fn(async () => {
    if (this.connectError !== null) {
      throw this.connectError;
    }
  });
  public readonly openInbox = vi.fn(async () => undefined);
  public readonly logout = vi.fn(async () => undefined);
  public lastSearchQuery: SearchObject | null = null;

  public constructor(
    private readonly uids: readonly number[],
    private readonly messages: ReadonlyMap<number, ImapFetchedMail>,
    private readonly connectError: Error | null = null,
  ) {}

  public async search(query: SearchObject): Promise<readonly number[]> {
    this.lastSearchQuery = query;
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
    expect(client.lastSearchQuery).toEqual({
      since: new Date("2026-04-11T12:00:00.000Z"),
      from: "rakuten-sec.co.jp",
      subject: "認証",
    });
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

  it("accepts messages received at the exact requested timestamp", async () => {
    const exactMessage: ImapFetchedMail = {
      source: "さくら + みかん",
      subject: "ログイン認証",
      from: ["notify@rakuten-sec.co.jp"],
      receivedAt: new Date("2026-04-11T12:00:00.000Z"),
    };
    const client = new FakeImapClient([10], new Map([[10, exactMessage]]));
    const source = new ImapRakutenAuthMailSource({
      create: () => client,
    });

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).resolves.toBe("さくら + みかん");
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

  it("matches Rakuten subdomains and subjects containing authentication keywords", async () => {
    const matchingMessage: ImapFetchedMail = {
      source: "さくら + みかん",
      subject: "ログイン追加認証のお知らせ",
      from: ["noreply@service.mail.rakuten-sec.co.jp"],
      receivedAt: new Date("2026-04-11T12:00:03.000Z"),
    };
    const client = new FakeImapClient([10], new Map([[10, matchingMessage]]));
    const source = new ImapRakutenAuthMailSource({
      create: () => client,
    });

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).resolves.toBe("さくら + みかん");
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

  it("maps IMAP connection failures to a mail source error", async () => {
    const source = new ImapRakutenAuthMailSource({
      create: () => new FakeImapClient([], new Map(), new Error("auth failed")),
    });

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).rejects.toThrow(RakutenAuthMailSourceError);
  });

  it("decodes quoted-printable text bodies", async () => {
    const quotedPrintableMessage: ImapFetchedMail = {
      source: [
        "Content-Type: text/plain; charset=UTF-8",
        "Content-Transfer-Encoding: quoted-printable",
        "",
        "=E3=81=95=E3=81=8F=E3=82=89 + =E3=81=BF=E3=81=8B=E3=82=93",
      ].join("\r\n"),
      subject: "ログイン認証",
      from: ["notify@rakuten-sec.co.jp"],
      receivedAt: new Date("2026-04-11T12:00:03.000Z"),
    };
    const client = new FakeImapClient([10], new Map([[10, quotedPrintableMessage]]));
    const source = new ImapRakutenAuthMailSource({
      create: () => client,
    });

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).resolves.toBe("さくら + みかん");
  });

  it("decodes base64 text bodies", async () => {
    const base64Message: ImapFetchedMail = {
      source: [
        "Content-Type: text/plain; charset=UTF-8",
        "Content-Transfer-Encoding: base64",
        "",
        Buffer.from("さくら + みかん", "utf8").toString("base64"),
      ].join("\r\n"),
      subject: "ログイン認証",
      from: ["notify@rakuten-sec.co.jp"],
      receivedAt: new Date("2026-04-11T12:00:03.000Z"),
    };
    const client = new FakeImapClient([10], new Map([[10, base64Message]]));
    const source = new ImapRakutenAuthMailSource({
      create: () => client,
    });

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).resolves.toBe("さくら + みかん");
  });

  it("extracts text from multipart mails", async () => {
    const multipartMessage: ImapFetchedMail = {
      source: [
        'Content-Type: multipart/alternative; boundary="boundary-1"',
        "",
        "--boundary-1",
        "Content-Type: text/plain; charset=UTF-8",
        "",
        "さくら + みかん",
        "--boundary-1",
        "Content-Type: text/html; charset=UTF-8",
        "",
        "<p>ignored</p>",
        "--boundary-1--",
      ].join("\r\n"),
      subject: "ログイン認証",
      from: ["notify@rakuten-sec.co.jp"],
      receivedAt: new Date("2026-04-11T12:00:03.000Z"),
    };
    const client = new FakeImapClient([10], new Map([[10, multipartMessage]]));
    const source = new ImapRakutenAuthMailSource({
      create: () => client,
    });

    await expect(
      source.fetchLatestAuthenticationMail(
        credential,
        new Date("2026-04-11T12:00:00.000Z"),
      ),
    ).resolves.toBe("さくら + みかん");
  });
});
