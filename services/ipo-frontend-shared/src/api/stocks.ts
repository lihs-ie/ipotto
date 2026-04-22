import { z } from "zod";

import {
  applicationIdentifierSchema,
  stockIdentifierSchema,
} from "../domain/identifiers";
import {
  applicationStatusSchema,
  fetchOriginSchema,
  lotteryResultSchema,
  stockStatusSchema,
} from "../domain/enums";
import {
  isoDateSchema,
  isoDateTimeSchema,
  sharesSchema,
  yenSchema,
} from "../domain/value-objects";

/// API-001: GET /api/v1/stocks — summary list item.
export const ipoStockSummarySchema = z.object({
  identifier: stockIdentifierSchema,
  companyName: z.string(),
  tickerSymbol: z.string().nullable(),
  market: z.string(),
  industry: z.string(),
  bookBuildingStartDate: isoDateSchema,
  bookBuildingEndDate: isoDateSchema,
  lotteryDate: isoDateSchema,
  listingDate: isoDateSchema,
  priceRangeMin: yenSchema,
  priceRangeMax: yenSchema,
  offerPrice: yenSchema.nullable(),
  leadUnderwriter: z.string(),
  numberOfOfferedShares: sharesSchema,
  status: stockStatusSchema,
});
export type IpoStockSummary = z.infer<typeof ipoStockSummarySchema>;

export const listIpoStocksResponseSchema = z.object({
  items: z.array(ipoStockSummarySchema),
  totalCount: z.number().int().nonnegative(),
});
export type ListIpoStocksResponse = z.infer<typeof listIpoStocksResponseSchema>;

/// API-001 query params.
export const listIpoStocksQuerySchema = z.object({
  status: stockStatusSchema.optional(),
});
export type ListIpoStocksQuery = z.infer<typeof listIpoStocksQuerySchema>;

/// API-002 — sub-schemas.
export const companyProfileSchema = z.object({
  companyName: z.string(),
  tickerSymbol: z.string().nullable(),
  market: z.string(),
  industry: z.string(),
});
export type CompanyProfile = z.infer<typeof companyProfileSchema>;

export const scheduleSchema = z.object({
  bookBuildingStartDate: isoDateSchema,
  bookBuildingEndDate: isoDateSchema,
  lotteryDate: isoDateSchema,
  listingDate: isoDateSchema,
});
export type Schedule = z.infer<typeof scheduleSchema>;

export const pricingSchema = z.object({
  priceRangeMin: yenSchema,
  priceRangeMax: yenSchema,
  offerPrice: yenSchema.nullable(),
});
export type Pricing = z.infer<typeof pricingSchema>;

export const offeringSchema = z.object({
  leadUnderwriter: z.string(),
  numberOfOfferedShares: sharesSchema,
});
export type Offering = z.infer<typeof offeringSchema>;

export const metaSourceSchema = z.object({
  source: fetchOriginSchema,
  fetchedAt: isoDateTimeSchema,
});
export type MetaSource = z.infer<typeof metaSourceSchema>;

export const stockApplicationSchema = z.object({
  identifier: applicationIdentifierSchema,
  securitiesCompany: z.string(),
  appliedShares: sharesSchema,
  appliedPrice: yenSchema,
  appliedAt: isoDateTimeSchema,
  lotteryOutcome: lotteryResultSchema.nullable(),
  status: applicationStatusSchema,
});
export type StockApplication = z.infer<typeof stockApplicationSchema>;

/// API-002: GET /api/v1/stocks/{stockIdentifier}.
export const getIpoStockResponseSchema = z.object({
  identifier: stockIdentifierSchema,
  companyProfile: companyProfileSchema,
  schedule: scheduleSchema,
  pricing: pricingSchema,
  offering: offeringSchema,
  status: stockStatusSchema,
  metaSource: metaSourceSchema,
  applications: z.array(stockApplicationSchema),
});
export type GetIpoStockResponse = z.infer<typeof getIpoStockResponseSchema>;
