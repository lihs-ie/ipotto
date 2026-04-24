import { z } from "zod";

export const apiErrorDetailSchema = z.object({
  field: z.string(),
  message: z.string(),
});
export type ApiErrorDetail = z.infer<typeof apiErrorDetailSchema>;

export const apiErrorSchema = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
    details: z.array(apiErrorDetailSchema).optional(),
  }),
});
export type ApiErrorResponse = z.infer<typeof apiErrorSchema>;
