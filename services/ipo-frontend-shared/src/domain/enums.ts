import { z } from "zod";

export const stockStatusSchema = z.enum([
  "Fetched",
  "Eligible",
  "Applied",
  "Won",
  "Lost",
  "Alternate",
  "Purchased",
  "Declined",
  "Sold",
  "Excluded",
  "Failed",
]);
export type StockStatus = z.infer<typeof stockStatusSchema>;

export const marketSchema = z.enum(["Prime", "Standard", "Growth"]);
export type Market = z.infer<typeof marketSchema>;

export const applicationStatusSchema = z.enum([
  "Pending",
  "Applied",
  "ResultChecked",
]);
export type ApplicationStatus = z.infer<typeof applicationStatusSchema>;

export const lotteryResultSchema = z.enum(["Won", "Lost", "Alternate"]);
export type LotteryResult = z.infer<typeof lotteryResultSchema>;

export const channelTypeSchema = z.enum(["LINE", "Email", "Slack"]);
export type ChannelType = z.infer<typeof channelTypeSchema>;

export const fetchOriginSchema = z.enum(["ExternalSite", "SecuritiesSite"]);
export type FetchOrigin = z.infer<typeof fetchOriginSchema>;

export const notificationEventTypeSchema = z.enum([
  "ApplicationCompleted",
  "LotteryResultWon",
  "LotteryResultLost",
  "OperationError",
  "StockUpdated",
]);
export type NotificationEventType = z.infer<typeof notificationEventTypeSchema>;

export const operationEventTypeSchema = z.enum([
  "fetch_stocks",
  "apply_lottery",
  "check_lottery_result",
  "notification_dispatch",
  "connection_test",
  "other",
]);
export type OperationEventType = z.infer<typeof operationEventTypeSchema>;

export const operationStatusSchema = z.enum(["succeeded", "failed"]);
export type OperationStatus = z.infer<typeof operationStatusSchema>;

export const securitiesCompanySchema = z.enum(["Rakuten"]);
export type SecuritiesCompany = z.infer<typeof securitiesCompanySchema>;
