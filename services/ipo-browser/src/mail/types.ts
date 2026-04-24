// Phase 3 Sprint 6.1 / 6.2 — shared types for the Rakuten 2FA mail OTP
// pipeline. Mirrors the shape of the Rust-side
// `ipo-backend-shared::infrastructure::mail` module so callers can
// freely swap between TypeScript and Rust implementations without
// changing the consumer contract.

export type MailCredential = {
  readonly mailAddress: string;
  readonly mailPassword: string;
  readonly imapHost: string;
  readonly imapPort: number;
};

export type ImageAuthenticationKeyword = {
  readonly first: string;
  readonly second: string;
};

export type MailPollingConfig = {
  readonly pollingIntervalMs: number;
  readonly maxTimeoutMs: number;
};

export const DEFAULT_MAIL_POLLING_CONFIG: MailPollingConfig = Object.freeze({
  // Keep identical to Rust defaults (MailPollingConfig::default()):
  // 3 second polling interval, 120 second total timeout driven by
  // Rakuten's 2-minute OTP validity window.
  pollingIntervalMs: 3_000,
  maxTimeoutMs: 120_000,
});

export class MailRetrievalTimeoutError extends Error {
  constructor(
    message = "Mail retrieval timed out before an image-auth keyword arrived",
  ) {
    super(message);
    this.name = "MailRetrievalTimeoutError";
  }
}
