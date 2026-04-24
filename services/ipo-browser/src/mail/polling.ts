import { parseImageAuthenticationKeyword } from "./keyword-extractor.js";
import {
  DEFAULT_MAIL_POLLING_CONFIG,
  MailRetrievalTimeoutError,
  type ImageAuthenticationKeyword,
  type MailPollingConfig,
} from "./types.js";

// Phase 3 Sprint 6.2 — polling loop that waits for the Rakuten 2FA mail
// to arrive. Mirrors the Rust implementation in
// `ipo-backend-shared::infrastructure::mail::imap_mail_reader`:
//   - supplier is invoked at most every `pollingIntervalMs` ms.
//   - Total elapsed time is capped at
//     `min(requestedTimeoutMs, config.maxTimeoutMs)`.
//   - Returns immediately when the supplier yields a body that the
//     keyword parser accepts.
//   - Throws `MailRetrievalTimeoutError` when the cap is reached.
//
// The supplier captures whatever credential / cutoff context the
// concrete reader needs in its closure — that keeps this module
// agnostic to IMAP vs Gmail-API identity models.

export type MailMessageSupplier = () => Promise<string | null>;

export type PollOptions = {
  readonly timeoutMs?: number;
  readonly config?: MailPollingConfig;
  readonly now?: () => number;
  readonly sleep?: (ms: number) => Promise<void>;
};

const defaultSleep = (ms: number): Promise<void> =>
  new Promise((resolve) => {
    setTimeout(resolve, ms);
  });

export async function pollForImageAuthenticationKeyword(
  supplier: MailMessageSupplier,
  options: PollOptions = {},
): Promise<ImageAuthenticationKeyword> {
  const config = options.config ?? DEFAULT_MAIL_POLLING_CONFIG;
  const now = options.now ?? Date.now;
  const sleep = options.sleep ?? defaultSleep;
  const startedAt = now();
  const requestedTimeout = options.timeoutMs ?? config.maxTimeoutMs;
  const cap = Math.min(requestedTimeout, config.maxTimeoutMs);
  const deadline = startedAt + cap;

  while (true) {
    const body = await supplier();
    if (body !== null) {
      const keyword = parseImageAuthenticationKeyword(body);
      if (keyword !== null) {
        return keyword;
      }
    }

    const current = now();
    if (current >= deadline) {
      throw new MailRetrievalTimeoutError();
    }
    const remaining = deadline - current;
    const sleepMs = Math.min(config.pollingIntervalMs, remaining);
    await sleep(sleepMs);
  }
}
