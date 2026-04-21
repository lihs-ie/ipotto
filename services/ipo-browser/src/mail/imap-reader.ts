import { ImapFlow } from "imapflow";
import { simpleParser } from "mailparser";

import {
  pollForImageAuthenticationKeyword,
  type MailMessageSupplier,
} from "./polling.js";
import type {
  ImageAuthenticationKeyword,
  MailCredential,
  MailPollingConfig,
} from "./types.js";

// Phase 3 Sprint 6.1 — IMAP-backed mail reader for the Rakuten 2FA
// image-auth keywords. Opens a fresh IMAP session per poll attempt
// (Rakuten mail volume is low and `imapflow` does not expose a cheap
// idle loop for this niche), searches `INBOX` for a match on the
// broker from-address plus subject `認証` within the caller-supplied
// cutoff window, parses the body with `mailparser`, and hands it back
// to the polling loop.

export type ImapMailReaderOptions = {
  readonly credential: MailCredential;
  readonly config?: MailPollingConfig;
  /**
   * Required fragment matched against the "From" header. Rakuten sends
   * their 2FA mails from rakuten-sec.co.jp, so the default keeps
   * production matching tight while allowing integration tests to
   * substitute a different sender.
   */
  readonly fromMatcher?: string;
  readonly subjectMatcher?: string;
};

const DEFAULT_FROM_MATCHER = "rakuten-sec.co.jp";
const DEFAULT_SUBJECT_MATCHER = "認証";

export class ImapMailReader {
  constructor(private readonly options: ImapMailReaderOptions) {}

  async fetchImageAuthenticationKeywords(
    receivedAfter: Date,
    timeoutMs?: number,
  ): Promise<ImageAuthenticationKeyword> {
    const supplier: MailMessageSupplier = () =>
      this.fetchLatestMailBody(receivedAfter);

    return pollForImageAuthenticationKeyword(supplier, {
      timeoutMs,
      config: this.options.config,
    });
  }

  private async fetchLatestMailBody(cutoff: Date): Promise<string | null> {
    const client = new ImapFlow({
      host: this.options.credential.imapHost,
      port: this.options.credential.imapPort,
      secure: true,
      auth: {
        user: this.options.credential.mailAddress,
        pass: this.options.credential.mailPassword,
      },
      logger: false,
    });

    try {
      await client.connect();
    } catch (error) {
      try {
        client.close();
      } catch {
        /* Connection that failed to open has nothing to tear down. */
      }
      throw error;
    }

    try {
      const lock = await client.getMailboxLock("INBOX");
      try {
        const uids = await client.search(
          {
            from: this.options.fromMatcher ?? DEFAULT_FROM_MATCHER,
            subject: this.options.subjectMatcher ?? DEFAULT_SUBJECT_MATCHER,
            since: cutoff,
          },
          { uid: true },
        );
        if (!uids || uids.length === 0) {
          return null;
        }
        const latest = uids[uids.length - 1];
        if (latest === undefined) {
          return null;
        }
        const message = await client.fetchOne(
          String(latest),
          { source: true },
          { uid: true },
        );
        if (!message || !message.source) {
          return null;
        }
        const parsed = await simpleParser(message.source);
        const body = parsed.text ?? parsed.html;
        if (typeof body !== "string" || body.length === 0) {
          return null;
        }
        return body;
      } finally {
        lock.release();
      }
    } finally {
      try {
        await client.logout();
      } catch {
        /* Already-closed connections can legitimately throw; ignore. */
      }
    }
  }
}
