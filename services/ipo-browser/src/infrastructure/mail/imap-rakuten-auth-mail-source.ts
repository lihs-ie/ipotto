import { ImapFlow, type SearchObject } from "imapflow";

import type { AccountCredential } from "../../domain/account-credential.js";
import type { RakutenAuthMailSource } from "./rakuten-auth-mail-parser.js";

/**
 * Mail sender and subject filters used for Rakuten authentication mails.
 */
export interface RakutenAuthMailSearchConfig {
  readonly from: string;
  readonly subject: string;
}

/**
 * Mail payload returned by the IMAP client abstraction.
 */
export interface ImapFetchedMail {
  readonly source: string | null;
  readonly subject: string | null;
  readonly from: readonly string[];
  readonly receivedAt: Date | null;
}

/**
 * Minimal IMAP client contract used by the Rakuten mail source.
 */
export interface ImapClientPort {
  /**
   * Establishes the IMAP connection.
   */
  connect(): Promise<void>;

  /**
   * Opens the target mailbox.
   */
  openInbox(): Promise<void>;

  /**
   * Searches for candidate message UIDs.
   */
  search(query: SearchObject): Promise<readonly number[]>;

  /**
   * Fetches a single message by UID.
   */
  fetchMessage(uid: number): Promise<ImapFetchedMail | null>;

  /**
   * Closes the IMAP session.
   */
  logout(): Promise<void>;
}

/**
 * Factory contract for IMAP clients.
 */
export interface ImapClientFactoryPort {
  /**
   * Creates an IMAP client for the given credential.
   */
  create(credential: AccountCredential): ImapClientPort;
}

class ImapFlowClientAdapter implements ImapClientPort {
  /**
   * Creates the adapter around imapflow.
   */
  public constructor(private readonly client: ImapFlow) {}

  /**
   * Establishes the IMAP connection.
   */
  public async connect(): Promise<void> {
    await this.client.connect();
  }

  /**
   * Opens the inbox in read-only mode.
   */
  public async openInbox(): Promise<void> {
    await this.client.mailboxOpen("INBOX", { readOnly: true });
  }

  /**
   * Searches for candidate message UIDs.
   */
  public async search(query: SearchObject): Promise<readonly number[]> {
    const result = await this.client.search(query, { uid: true });
    return result === false ? [] : result;
  }

  /**
   * Fetches a single message by UID.
   */
  public async fetchMessage(uid: number): Promise<ImapFetchedMail | null> {
    const message = await this.client.fetchOne(
      uid,
      {
        source: true,
        envelope: true,
        internalDate: true,
      },
      { uid: true },
    );

    if (message === false) {
      return null;
    }

    return {
      source: message.source?.toString("utf8") ?? null,
      subject: message.envelope?.subject ?? null,
      from:
        message.envelope?.from?.flatMap((address) =>
          address.address === undefined ? [] : [address.address],
        ) ?? [],
      receivedAt:
        message.internalDate instanceof Date
          ? message.internalDate
          : message.internalDate === undefined
            ? null
            : new Date(message.internalDate),
    };
  }

  /**
   * Closes the IMAP session.
   */
  public async logout(): Promise<void> {
    await this.client.logout();
  }
}

/**
 * Production factory backed by imapflow.
 */
export class ImapFlowClientFactory implements ImapClientFactoryPort {
  /**
   * Creates a concrete IMAP client for the given credential.
   */
  public create(credential: AccountCredential): ImapClientPort {
    return new ImapFlowClientAdapter(
      new ImapFlow({
        host: credential.mailCredential.imapHost,
        port: credential.mailCredential.imapPort,
        secure: credential.mailCredential.imapPort === 993,
        auth: {
          user: credential.mailCredential.mailAddress,
          pass: credential.mailCredential.mailPassword,
        },
        disableAutoIdle: true,
      }),
    );
  }
}

/**
 * IMAP-backed Rakuten authentication mail source.
 */
export class ImapRakutenAuthMailSource implements RakutenAuthMailSource {
  /**
   * Creates the mail source.
   */
  public constructor(
    private readonly clientFactory: ImapClientFactoryPort = new ImapFlowClientFactory(),
    private readonly searchConfig: RakutenAuthMailSearchConfig = {
      from: "rakuten-sec.co.jp",
      subject: "認証",
    },
  ) {}

  /**
   * Returns the latest Rakuten authentication mail body when available.
   */
  public async fetchLatestAuthenticationMail(
    credential: AccountCredential,
    receivedAfter: Date,
  ): Promise<string | null> {
    const client = this.clientFactory.create(credential);

    try {
      await client.connect();
      await client.openInbox();

      const uids = await client.search({
        since: receivedAfter,
        from: this.searchConfig.from,
        subject: this.searchConfig.subject,
      });

      const sortedUids = [...uids].sort((left, right) => right - left);
      for (const uid of sortedUids) {
        const message = await client.fetchMessage(uid);
        if (message === null) {
          continue;
        }
        if (!matchesSearchConfig(message, this.searchConfig)) {
          continue;
        }
        if (
          message.receivedAt !== null &&
          message.receivedAt.getTime() < receivedAfter.getTime()
        ) {
          continue;
        }
        if (message.source !== null && message.source.trim() !== "") {
          return message.source;
        }
      }

      return null;
    } finally {
      try {
        await client.logout();
      } catch {
        // Ignore logout failures to preserve the original result.
      }
    }
  }
}

/**
 * Returns whether the fetched mail matches the configured Rakuten filters.
 */
function matchesSearchConfig(
  message: ImapFetchedMail,
  searchConfig: RakutenAuthMailSearchConfig,
): boolean {
  const normalizedSubject = message.subject ?? "";
  const normalizedSenders = message.from.join(" ");

  return (
    normalizedSubject.includes(searchConfig.subject) &&
    normalizedSenders.includes(searchConfig.from)
  );
}
