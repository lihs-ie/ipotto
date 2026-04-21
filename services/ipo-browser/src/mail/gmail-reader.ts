import { google, type gmail_v1 } from "googleapis";

import {
  pollForImageAuthenticationKeyword,
  type MailMessageSupplier,
} from "./polling.js";
import type {
  ImageAuthenticationKeyword,
  MailPollingConfig,
} from "./types.js";

// Phase 3 Sprint 6.1 — Gmail-API reader for the Rakuten 2FA image-auth
// keywords. Runs on a long-lived OAuth2 refresh token (operators
// minted out-of-band and stored in Secret Manager) so each poll only
// needs to exchange for a short-lived access token and call
// `users.messages.list` + `.get`. Body extraction walks MIME parts and
// base64url-decodes text/plain (or falls back to text/html) so the
// downstream keyword parser always sees a plain string.

export type GmailRefreshCredential = {
  readonly clientId: string;
  readonly clientSecret: string;
  readonly refreshToken: string;
  readonly userId?: string;
};

export type GmailMailReaderOptions = {
  readonly credential: GmailRefreshCredential;
  readonly config?: MailPollingConfig;
  readonly fromQuery?: string;
  readonly subjectQuery?: string;
  readonly maxResults?: number;
};

const DEFAULT_FROM_QUERY = "rakuten-sec.co.jp";
const DEFAULT_SUBJECT_QUERY = "認証";
const DEFAULT_MAX_RESULTS = 5;

export class GmailMailReader {
  constructor(private readonly options: GmailMailReaderOptions) {}

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
    const gmail = this.buildGmailClient();
    const query = this.buildQuery(cutoff);
    const list = await gmail.users.messages.list({
      userId: this.options.credential.userId ?? "me",
      q: query,
      maxResults: this.options.maxResults ?? DEFAULT_MAX_RESULTS,
    });
    const messages = list.data.messages;
    if (messages === undefined || messages === null || messages.length === 0) {
      return null;
    }
    const latestId = messages[0]?.id;
    if (latestId === undefined || latestId === null) {
      return null;
    }
    const full = await gmail.users.messages.get({
      userId: this.options.credential.userId ?? "me",
      id: latestId,
      format: "full",
    });
    return extractBody(full.data.payload);
  }

  private buildGmailClient(): gmail_v1.Gmail {
    const oauth2 = new google.auth.OAuth2(
      this.options.credential.clientId,
      this.options.credential.clientSecret,
    );
    oauth2.setCredentials({
      refresh_token: this.options.credential.refreshToken,
    });
    return google.gmail({ version: "v1", auth: oauth2 });
  }

  private buildQuery(cutoff: Date): string {
    const fromQuery = this.options.fromQuery ?? DEFAULT_FROM_QUERY;
    const subjectQuery = this.options.subjectQuery ?? DEFAULT_SUBJECT_QUERY;
    const cutoffSeconds = Math.floor(cutoff.getTime() / 1000);
    return `from:${fromQuery} subject:${subjectQuery} after:${cutoffSeconds}`;
  }
}

export function extractBody(
  payload: gmail_v1.Schema$MessagePart | undefined,
): string | null {
  if (payload === undefined || payload === null) {
    return null;
  }
  const mimeType = payload.mimeType ?? "";
  const decoded = decodePart(payload);
  if (mimeType === "text/plain" && decoded !== null) {
    return decoded;
  }
  if (mimeType === "text/html" && decoded !== null) {
    return stripHtml(decoded);
  }
  if (payload.parts !== undefined && payload.parts !== null) {
    const textPart = payload.parts.find((part) => part.mimeType === "text/plain");
    const textBody = textPart !== undefined ? extractBody(textPart) : null;
    if (textBody !== null) {
      return textBody;
    }
    const htmlPart = payload.parts.find((part) => part.mimeType === "text/html");
    const htmlBody = htmlPart !== undefined ? extractBody(htmlPart) : null;
    if (htmlBody !== null) {
      return htmlBody;
    }
    for (const part of payload.parts) {
      const nested = extractBody(part);
      if (nested !== null) {
        return nested;
      }
    }
  }
  // Fallback when the payload is a single-part message with an unknown
  // MIME type: decode whatever body is present.
  return decoded;
}

function decodePart(payload: gmail_v1.Schema$MessagePart): string | null {
  const data = payload.body?.data;
  if (data === undefined || data === null || data.length === 0) {
    return null;
  }
  try {
    return Buffer.from(data, "base64url").toString("utf-8");
  } catch {
    return null;
  }
}

function stripHtml(html: string): string {
  return html
    .replace(/<script[\s\S]*?<\/script>/gi, "")
    .replace(/<style[\s\S]*?<\/style>/gi, "")
    .replace(/<[^>]+>/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}
