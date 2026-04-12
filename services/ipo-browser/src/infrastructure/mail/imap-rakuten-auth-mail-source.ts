import { ImapFlow, type SearchObject } from "imapflow";

import type { AccountCredential } from "../../domain/account-credential.js";
import {
  RakutenAuthMailSourceError,
  type RakutenAuthMailSource,
} from "./rakuten-auth-mail-parser.js";

/**
 * Mail sender and subject filters used for Rakuten authentication mails.
 */
export interface RakutenAuthMailSearchConfig {
  readonly fromDomain: string;
  readonly subjectKeywords: readonly string[];
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
      fromDomain: "rakuten-sec.co.jp",
      subjectKeywords: ["認証"],
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
        from: this.searchConfig.fromDomain,
        subject: this.searchConfig.subjectKeywords[0],
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
        const decodedBody = decodeMailBody(message.source);
        if (decodedBody !== null && decodedBody.trim() !== "") {
          return decodedBody;
        }
      }

      return null;
    } catch (error) {
      throw new RakutenAuthMailSourceError(
        error instanceof Error
          ? error.message
          : "failed to retrieve Rakuten authentication mail",
      );
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
  const normalizedSubject = (message.subject ?? "").toLowerCase();
  const hasMatchingSubject = searchConfig.subjectKeywords.some((keyword) =>
    normalizedSubject.includes(keyword.toLowerCase()),
  );
  const hasMatchingSender = message.from.some((sender) =>
    normalizeMailAddress(sender).endsWith(
      normalizeMailAddress(searchConfig.fromDomain),
    ),
  );

  return hasMatchingSubject && hasMatchingSender;
}

/**
 * Extracts a text body from the raw IMAP message source.
 */
function decodeMailBody(source: string | null): string | null {
  if (source === null || source.trim() === "") {
    return null;
  }

  if (!hasHeaderSection(source)) {
    return source;
  }

  const boundary = extractBoundary(source);
  if (boundary !== null) {
    const multipartBody = decodeMultipartBody(source, boundary);
    if (multipartBody !== null) {
      return multipartBody;
    }
  }

  const [, body = ""] = splitHeadersAndBody(source);
  return decodeTransferEncodedBody(parseHeaders(source), body);
}

/**
 * Returns the MIME boundary when the source is multipart.
 */
function extractBoundary(source: string): string | null {
  const contentType = parseHeaders(source).get("content-type");
  if (contentType === undefined || !contentType.toLowerCase().includes("multipart/")) {
    return null;
  }

  const match = /boundary="?([^";]+)"?/iu.exec(contentType);
  return match?.[1] ?? null;
}

/**
 * Extracts the first text/plain part from a multipart source.
 */
function decodeMultipartBody(source: string, boundary: string): string | null {
  const parts = source.split(`--${boundary}`);
  for (const part of parts) {
    const trimmed = part.trim();
    if (trimmed === "" || trimmed === "--") {
      continue;
    }

    const headers = parseHeaders(trimmed);
    const contentType = headers.get("content-type")?.toLowerCase() ?? "";
    if (!contentType.includes("text/plain")) {
      continue;
    }

    const [, body = ""] = splitHeadersAndBody(trimmed);
    const decoded = decodeTransferEncodedBody(headers, body);
    if (decoded.trim() !== "") {
      return decoded;
    }
  }

  return null;
}

/**
 * Splits raw message source into header and body sections.
 */
function splitHeadersAndBody(source: string): readonly [string, string] {
  const separator = source.search(/\r?\n\r?\n/u);
  if (separator === -1) {
    return [source, ""];
  }

  const header = source.slice(0, separator);
  const body = source.slice(source.slice(0, separator).length).replace(
    /^\r?\n\r?\n/u,
    "",
  );
  return [header, body];
}

/**
 * Returns whether the raw source looks like an RFC822 message with headers.
 */
function hasHeaderSection(source: string): boolean {
  const firstLine = source.split(/\r?\n/u, 1)[0] ?? "";
  return /^[A-Za-z0-9-]+:\s/u.test(firstLine);
}

/**
 * Parses message headers into a normalized map.
 */
function parseHeaders(source: string): Map<string, string> {
  const [headerSection] = splitHeadersAndBody(source);
  const headers = new Map<string, string>();
  let currentName: string | null = null;

  for (const line of headerSection.split(/\r?\n/u)) {
    if (/^\s+/u.test(line) && currentName !== null) {
      headers.set(currentName, `${headers.get(currentName) ?? ""} ${line.trim()}`);
      continue;
    }

    const separatorIndex = line.indexOf(":");
    if (separatorIndex === -1) {
      continue;
    }

    currentName = line.slice(0, separatorIndex).trim().toLowerCase();
    headers.set(currentName, line.slice(separatorIndex + 1).trim());
  }

  return headers;
}

/**
 * Decodes a body according to Content-Transfer-Encoding when possible.
 */
function decodeTransferEncodedBody(
  headers: ReadonlyMap<string, string>,
  body: string,
): string {
  const transferEncoding =
    headers.get("content-transfer-encoding")?.toLowerCase() ?? "7bit";
  const trimmedBody = body.trim();

  if (transferEncoding === "base64") {
    return Buffer.from(trimmedBody.replace(/\s+/gu, ""), "base64").toString("utf8");
  }

  if (transferEncoding === "quoted-printable") {
    return decodeQuotedPrintable(trimmedBody);
  }

  return body;
}

/**
 * Decodes a quoted-printable body into UTF-8 text.
 */
function decodeQuotedPrintable(value: string): string {
  const withoutSoftBreaks = value.replace(/=(\r?\n)/gu, "");
  const bytes: number[] = [];

  for (let index = 0; index < withoutSoftBreaks.length; index += 1) {
    const current = withoutSoftBreaks[index];
    if (current === "=") {
      const hex = withoutSoftBreaks.slice(index + 1, index + 3);
      if (/^[0-9a-f]{2}$/iu.test(hex)) {
        bytes.push(Number.parseInt(hex, 16));
        index += 2;
        continue;
      }
    }

    bytes.push(withoutSoftBreaks.charCodeAt(index));
  }

  return Buffer.from(bytes).toString("utf8");
}

/**
 * Normalizes a mail address for suffix comparison.
 */
function normalizeMailAddress(value: string): string {
  return value.trim().toLowerCase();
}
