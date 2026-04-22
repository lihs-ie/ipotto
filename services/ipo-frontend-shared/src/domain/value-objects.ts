import { z } from "zod";

export const companyNameSchema = z.string().min(1).max(200).brand("CompanyName");
export type CompanyName = z.infer<typeof companyNameSchema>;

export const tickerSymbolSchema = z
  .string()
  .regex(/^\d{4,5}$/, "ticker symbol must be 4 or 5 digit characters")
  .brand("TickerSymbol");
export type TickerSymbol = z.infer<typeof tickerSymbolSchema>;

export const industrySchema = z.string().min(1).max(100).brand("Industry");
export type Industry = z.infer<typeof industrySchema>;

export const leadUnderwriterSchema = z
  .string()
  .min(1)
  .max(100)
  .brand("LeadUnderwriter");
export type LeadUnderwriter = z.infer<typeof leadUnderwriterSchema>;

export const exclusionReasonSchema = z
  .string()
  .min(1)
  .max(500)
  .brand("ExclusionReason");
export type ExclusionReason = z.infer<typeof exclusionReasonSchema>;

/// Integer yen amount. Rust side uses i64; we keep `number` since JSON numbers
/// in this range never lose precision for IPO prices (< 10 million yen).
export const yenSchema = z.number().int().nonnegative().brand("Yen");
export type Yen = z.infer<typeof yenSchema>;

export const sharesSchema = z.number().int().nonnegative().brand("Shares");
export type Shares = z.infer<typeof sharesSchema>;

export const isoDateSchema = z
  .string()
  .regex(/^\d{4}-\d{2}-\d{2}$/, "date must be ISO 8601 (YYYY-MM-DD)")
  .brand("IsoDate");
export type IsoDate = z.infer<typeof isoDateSchema>;

export const isoDateTimeSchema = z
  .string()
  .datetime({ offset: true })
  .brand("IsoDateTime");
export type IsoDateTime = z.infer<typeof isoDateTimeSchema>;

export const mailAddressSchema = z.string().email().max(254).brand("MailAddress");
export type MailAddress = z.infer<typeof mailAddressSchema>;

export const loginIdSchema = z.string().min(1).max(100).brand("LoginId");
export type LoginId = z.infer<typeof loginIdSchema>;

export const loginPasswordSchema = z.string().min(1).max(200).brand("LoginPassword");
export type LoginPassword = z.infer<typeof loginPasswordSchema>;

export const tradingPasswordSchema = z
  .string()
  .min(4)
  .max(20)
  .brand("TradingPassword");
export type TradingPassword = z.infer<typeof tradingPasswordSchema>;

export const mailPasswordSchema = z.string().min(1).max(200).brand("MailPassword");
export type MailPassword = z.infer<typeof mailPasswordSchema>;

export const imapHostSchema = z
  .string()
  .regex(/^[a-zA-Z0-9.-]+$/, "imap host must be a valid hostname")
  .max(253)
  .brand("ImapHost");
export type ImapHost = z.infer<typeof imapHostSchema>;

export const imapPortSchema = z.number().int().min(1).max(65535).brand("ImapPort");
export type ImapPort = z.infer<typeof imapPortSchema>;
