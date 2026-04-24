import { z } from "zod";

import { stockIdentifierSchema } from "../domain/identifiers";
import {
  dashboardActivityEventTypeSchema,
  stockStatusSchema,
} from "../domain/enums";
import { isoDateSchema, isoDateTimeSchema } from "../domain/value-objects";

export const dashboardRecentActivitySchema = z.object({
  stock: stockIdentifierSchema,
  companyName: z.string(),
  securitiesCompany: z.string(),
  eventType: dashboardActivityEventTypeSchema,
  occurredAt: isoDateTimeSchema,
});
export type DashboardRecentActivity = z.infer<
  typeof dashboardRecentActivitySchema
>;

export const dashboardUpcomingStockSchema = z.object({
  stock: stockIdentifierSchema,
  companyName: z.string(),
  bookBuildingStartDate: isoDateSchema,
  bookBuildingEndDate: isoDateSchema,
  lotteryDate: isoDateSchema,
});
export type DashboardUpcomingStock = z.infer<
  typeof dashboardUpcomingStockSchema
>;

export const dashboardAccountStatusSchema = z.object({
  securitiesCompany: z.string(),
  connectionStatus: z.string(),
  lastTestedAt: isoDateTimeSchema.nullable(),
});
export type DashboardAccountStatus = z.infer<
  typeof dashboardAccountStatusSchema
>;

export const dashboardSystemStatusSchema = z.object({
  nextJobScheduledAt: isoDateTimeSchema,
  accounts: z.array(dashboardAccountStatusSchema),
});
export type DashboardSystemStatus = z.infer<typeof dashboardSystemStatusSchema>;

/// API-003: GET /api/v1/dashboard.
export const dashboardSummaryResponseSchema = z.object({
  statusCounts: z.record(stockStatusSchema, z.number().int().nonnegative()),
  recentActivities: z.array(dashboardRecentActivitySchema),
  upcomingStocks: z.array(dashboardUpcomingStockSchema),
  systemStatus: dashboardSystemStatusSchema,
});
export type DashboardSummaryResponse = z.infer<
  typeof dashboardSummaryResponseSchema
>;
