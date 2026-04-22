import { describe, expect, it } from "vitest";

import {
  companyNameSchema,
  exclusionReasonSchema,
  imapHostSchema,
  imapPortSchema,
  industrySchema,
  isoDateSchema,
  isoDateTimeSchema,
  leadUnderwriterSchema,
  loginIdSchema,
  mailAddressSchema,
  sharesSchema,
  tickerSymbolSchema,
  tradingPasswordSchema,
  yenSchema,
} from "./value-objects";

describe("value object schemas", () => {
  it("companyName accepts 1-200 chars and brands", () => {
    expect(companyNameSchema.parse("○○株式会社")).toBe("○○株式会社");
  });

  it("companyName rejects empty and oversized", () => {
    expect(() => companyNameSchema.parse("")).toThrow();
    expect(() => companyNameSchema.parse("a".repeat(201))).toThrow();
  });

  it("tickerSymbol requires 4-5 digit string", () => {
    expect(tickerSymbolSchema.parse("1234")).toBe("1234");
    expect(tickerSymbolSchema.parse("12345")).toBe("12345");
    expect(() => tickerSymbolSchema.parse("12")).toThrow();
    expect(() => tickerSymbolSchema.parse("abcd")).toThrow();
  });

  it("industry accepts 1-100 chars", () => {
    expect(industrySchema.parse("情報・通信業")).toBe("情報・通信業");
    expect(() => industrySchema.parse("")).toThrow();
  });

  it("leadUnderwriter accepts 1-100 chars", () => {
    expect(leadUnderwriterSchema.parse("楽天証券")).toBe("楽天証券");
  });

  it("exclusionReason enforces 500 char limit", () => {
    expect(exclusionReasonSchema.parse("理由")).toBe("理由");
    expect(() => exclusionReasonSchema.parse("a".repeat(501))).toThrow();
  });

  it("yen rejects negative and non-integer", () => {
    expect(yenSchema.parse(1400)).toBe(1400);
    expect(() => yenSchema.parse(-1)).toThrow();
    expect(() => yenSchema.parse(1.5)).toThrow();
  });

  it("shares rejects negative and non-integer", () => {
    expect(sharesSchema.parse(100_000)).toBe(100_000);
    expect(() => sharesSchema.parse(-1)).toThrow();
  });

  it("isoDate enforces YYYY-MM-DD", () => {
    expect(isoDateSchema.parse("2026-04-15")).toBe("2026-04-15");
    expect(() => isoDateSchema.parse("2026/04/15")).toThrow();
    expect(() => isoDateSchema.parse("2026-4-15")).toThrow();
  });

  it("isoDateTime enforces ISO 8601 with offset", () => {
    expect(isoDateTimeSchema.parse("2026-04-15T10:00:00Z")).toBe(
      "2026-04-15T10:00:00Z",
    );
    expect(() => isoDateTimeSchema.parse("2026-04-15 10:00:00")).toThrow();
  });

  it("mailAddress enforces RFC email", () => {
    expect(mailAddressSchema.parse("user@example.com")).toBe(
      "user@example.com",
    );
    expect(() => mailAddressSchema.parse("not-an-email")).toThrow();
  });

  it("loginId accepts 1-100 chars", () => {
    expect(loginIdSchema.parse("login")).toBe("login");
    expect(() => loginIdSchema.parse("")).toThrow();
  });

  it("tradingPassword enforces 4-20 length", () => {
    expect(tradingPasswordSchema.parse("1234")).toBe("1234");
    expect(() => tradingPasswordSchema.parse("123")).toThrow();
    expect(() => tradingPasswordSchema.parse("a".repeat(21))).toThrow();
  });

  it("imapHost accepts valid hostname characters", () => {
    expect(imapHostSchema.parse("imap.example.com")).toBe("imap.example.com");
    expect(() => imapHostSchema.parse("imap example.com")).toThrow();
  });

  it("imapPort enforces 1-65535", () => {
    expect(imapPortSchema.parse(993)).toBe(993);
    expect(() => imapPortSchema.parse(0)).toThrow();
    expect(() => imapPortSchema.parse(65536)).toThrow();
  });
});
