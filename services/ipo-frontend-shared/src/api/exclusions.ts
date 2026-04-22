import { z } from "zod";

import { exclusionIdentifierSchema } from "../domain/identifiers";
import {
  companyNameSchema,
  exclusionReasonSchema,
  isoDateTimeSchema,
} from "../domain/value-objects";

/// API-004: GET /api/v1/exclusions — list item.
export const exclusionSummarySchema = z.object({
  identifier: exclusionIdentifierSchema,
  companyName: z.string(),
  reason: z.string(),
  registeredAt: isoDateTimeSchema,
});
export type ExclusionSummary = z.infer<typeof exclusionSummarySchema>;

export const listExclusionsResponseSchema = z.object({
  items: z.array(exclusionSummarySchema),
  totalCount: z.number().int().nonnegative(),
});
export type ListExclusionsResponse = z.infer<
  typeof listExclusionsResponseSchema
>;

/// API-005: POST /api/v1/exclusions — request body.
export const createExclusionRequestSchema = z.object({
  companyName: companyNameSchema,
  reason: exclusionReasonSchema,
});
export type CreateExclusionRequest = z.infer<
  typeof createExclusionRequestSchema
>;

/// API-005 response — reuses `exclusionSummarySchema`.
export const createExclusionResponseSchema = exclusionSummarySchema;
export type CreateExclusionResponse = ExclusionSummary;
