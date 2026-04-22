import { z } from "zod";

import {
  channelIdentifierSchema,
  notificationSettingIdentifierSchema,
} from "../domain/identifiers";
import {
  channelTypeSchema,
  notificationEventTypeSchema,
} from "../domain/enums";

/// Channel destination payload — the value shape depends on `channelType`
/// but the API keeps it as a string → string map so the frontend can
/// render dynamic forms. Validation is performed by the backend.
export const channelDestinationSchema = z.record(z.string(), z.string());
export type ChannelDestination = z.infer<typeof channelDestinationSchema>;

export const channelSubscriptionsSchema = z.record(
  notificationEventTypeSchema,
  z.boolean(),
);
export type ChannelSubscriptions = z.infer<typeof channelSubscriptionsSchema>;

export const notificationChannelSchema = z.object({
  identifier: channelIdentifierSchema,
  channelType: channelTypeSchema,
  destination: channelDestinationSchema,
  enabled: z.boolean(),
  subscriptions: channelSubscriptionsSchema,
});
export type NotificationChannel = z.infer<typeof notificationChannelSchema>;

/// API-007: GET /api/v1/notifications/settings.
export const getNotificationSettingResponseSchema = z.object({
  identifier: notificationSettingIdentifierSchema,
  enabled: z.boolean(),
  channels: z.array(notificationChannelSchema),
});
export type GetNotificationSettingResponse = z.infer<
  typeof getNotificationSettingResponseSchema
>;

/// API-008: PUT /api/v1/notifications/settings — request body.
export const updateNotificationChannelInputSchema = z.object({
  channelType: channelTypeSchema,
  destination: channelDestinationSchema,
  enabled: z.boolean(),
  subscriptions: channelSubscriptionsSchema,
});
export type UpdateNotificationChannelInput = z.infer<
  typeof updateNotificationChannelInputSchema
>;

export const updateNotificationSettingRequestSchema = z.object({
  enabled: z.boolean(),
  channels: z.array(updateNotificationChannelInputSchema),
});
export type UpdateNotificationSettingRequest = z.infer<
  typeof updateNotificationSettingRequestSchema
>;

export const updateNotificationSettingResponseSchema =
  getNotificationSettingResponseSchema;
export type UpdateNotificationSettingResponse = GetNotificationSettingResponse;
