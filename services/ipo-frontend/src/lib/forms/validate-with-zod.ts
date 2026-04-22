import { type ZodTypeAny, type z } from "zod";

export type ValidationResult<T> =
  | { ok: true; value: T }
  | { ok: false; errors: Record<string, string> };

/// Runs `schema.safeParse(input)` and, on failure, flattens every
/// Zod issue into a `{ fieldName: errorMessage }` map so form
/// components can render per-field validation text without re-
/// walking `ZodError`.
export const validateWithZod = <S extends ZodTypeAny>(
  schema: S,
  input: unknown,
): ValidationResult<z.output<S>> => {
  const parsed = schema.safeParse(input);
  if (parsed.success) {
    return { ok: true, value: parsed.data };
  }
  const errors: Record<string, string> = {};
  for (const issue of parsed.error.issues) {
    const path = issue.path.map((segment) => String(segment)).join(".");
    const key = path === "" ? "_root" : path;
    if (!(key in errors)) {
      errors[key] = issue.message;
    }
  }
  return { ok: false, errors };
};
