import { z } from "zod";

export const stockIdentifierSchema = z.string().min(1).brand("StockIdentifier");
export type StockIdentifier = z.infer<typeof stockIdentifierSchema>;

export const applicationIdentifierSchema = z
  .string()
  .min(1)
  .brand("ApplicationIdentifier");
export type ApplicationIdentifier = z.infer<typeof applicationIdentifierSchema>;

export const exclusionIdentifierSchema = z
  .string()
  .min(1)
  .brand("ExclusionIdentifier");
export type ExclusionIdentifier = z.infer<typeof exclusionIdentifierSchema>;

export const securitiesAccountIdentifierSchema = z
  .string()
  .min(1)
  .brand("SecuritiesAccountIdentifier");
export type SecuritiesAccountIdentifier = z.infer<
  typeof securitiesAccountIdentifierSchema
>;

export const notificationSettingIdentifierSchema = z
  .string()
  .min(1)
  .brand("NotificationSettingIdentifier");
export type NotificationSettingIdentifier = z.infer<
  typeof notificationSettingIdentifierSchema
>;

export const channelIdentifierSchema = z
  .string()
  .min(1)
  .brand("ChannelIdentifier");
export type ChannelIdentifier = z.infer<typeof channelIdentifierSchema>;

export const operationLogIdentifierSchema = z
  .string()
  .min(1)
  .brand("OperationLogIdentifier");
export type OperationLogIdentifier = z.infer<
  typeof operationLogIdentifierSchema
>;
