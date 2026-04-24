import { z } from "zod";

import {
  operationLogIdentifierSchema,
  stockIdentifierSchema,
} from "../domain/identifiers";
import {
  operationEventTypeSchema,
  operationStatusSchema,
} from "../domain/enums";
import { isoDateTimeSchema } from "../domain/value-objects";

export const operationLogSummarySchema = z.object({
  identifier: operationLogIdentifierSchema,
  stock: stockIdentifierSchema.nullable(),
  eventType: operationEventTypeSchema,
  serviceName: z.string(),
  status: operationStatusSchema,
  message: z.string(),
  errorDetails: z.string().nullable(),
  executedAt: isoDateTimeSchema,
});
export type OperationLogSummary = z.infer<typeof operationLogSummarySchema>;

/// API-014: GET /api/v1/logs — cursor-based pagination.
export const listOperationLogsQuerySchema = z.object({
  eventType: operationEventTypeSchema.optional(),
  from: isoDateTimeSchema.optional(),
  to: isoDateTimeSchema.optional(),
  limit: z.number().int().positive().max(200).optional(),
  cursor: z.string().optional(),
});
export type ListOperationLogsQuery = z.infer<
  typeof listOperationLogsQuerySchema
>;

export const listOperationLogsResponseSchema = z.object({
  items: z.array(operationLogSummarySchema),
  nextCursor: z.string().nullable(),
  hasMore: z.boolean(),
});
export type ListOperationLogsResponse = z.infer<
  typeof listOperationLogsResponseSchema
>;
