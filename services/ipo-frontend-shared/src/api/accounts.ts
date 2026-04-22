import { z } from "zod";

import { securitiesAccountIdentifierSchema } from "../domain/identifiers";
import { securitiesCompanySchema } from "../domain/enums";
import {
  imapHostSchema,
  imapPortSchema,
  isoDateTimeSchema,
  loginIdSchema,
  loginPasswordSchema,
  mailAddressSchema,
  mailPasswordSchema,
  tradingPasswordSchema,
} from "../domain/value-objects";

/// API-009: GET /api/v1/accounts — credentials are masked in responses.
export const securitiesAccountSummarySchema = z.object({
  identifier: securitiesAccountIdentifierSchema,
  securitiesCompany: z.string(),
  loginIdMasked: z.string(),
  mailAddressMasked: z.string(),
  imapHost: z.string(),
  imapPort: z.number().int(),
  active: z.boolean(),
  registeredAt: isoDateTimeSchema,
  lastTestedAt: isoDateTimeSchema.nullable(),
  lastTestedStatus: z.string().nullable(),
});
export type SecuritiesAccountSummary = z.infer<
  typeof securitiesAccountSummarySchema
>;

export const listSecuritiesAccountsResponseSchema = z.object({
  items: z.array(securitiesAccountSummarySchema),
  totalCount: z.number().int().nonnegative(),
});
export type ListSecuritiesAccountsResponse = z.infer<
  typeof listSecuritiesAccountsResponseSchema
>;

/// API-010: POST /api/v1/accounts — request body. Stored encrypted.
export const createSecuritiesAccountRequestSchema = z.object({
  securitiesCompany: securitiesCompanySchema,
  loginId: loginIdSchema,
  loginPassword: loginPasswordSchema,
  tradingPassword: tradingPasswordSchema,
  mailAddress: mailAddressSchema,
  mailPassword: mailPasswordSchema,
  imapHost: imapHostSchema,
  imapPort: imapPortSchema,
});
export type CreateSecuritiesAccountRequest = z.infer<
  typeof createSecuritiesAccountRequestSchema
>;

export const createSecuritiesAccountResponseSchema =
  securitiesAccountSummarySchema;
export type CreateSecuritiesAccountResponse = SecuritiesAccountSummary;

/// API-011: PUT /api/v1/accounts/{accountIdentifier} — partial update.
export const updateSecuritiesAccountRequestSchema = z.object({
  loginId: loginIdSchema.optional(),
  loginPassword: loginPasswordSchema.optional(),
  tradingPassword: tradingPasswordSchema.optional(),
  mailAddress: mailAddressSchema.optional(),
  mailPassword: mailPasswordSchema.optional(),
  imapHost: imapHostSchema.optional(),
  imapPort: imapPortSchema.optional(),
  active: z.boolean().optional(),
});
export type UpdateSecuritiesAccountRequest = z.infer<
  typeof updateSecuritiesAccountRequestSchema
>;

export const updateSecuritiesAccountResponseSchema =
  securitiesAccountSummarySchema;
export type UpdateSecuritiesAccountResponse = SecuritiesAccountSummary;

/// API-013: POST /api/v1/accounts/{accountIdentifier}/test — body is empty.
export const testSecuritiesAccountConnectionResponseSchema = z.object({
  success: z.boolean(),
  message: z.string(),
  testedAt: isoDateTimeSchema,
});
export type TestSecuritiesAccountConnectionResponse = z.infer<
  typeof testSecuritiesAccountConnectionResponseSchema
>;
