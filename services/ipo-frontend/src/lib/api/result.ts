/// Rust-style `Result<T, E>` sum type. The API layer returns these so
/// callers can discriminate between success and typed errors without
/// relying on `throw` / `try-catch`.
export type Result<T, E> =
  | { ok: true; value: T }
  | { ok: false; error: E };

export type AsyncResult<T, E> = Promise<Result<T, E>>;

export const ok = <T,>(value: T): Result<T, never> => ({ ok: true, value });

export const err = <E,>(error: E): Result<never, E> => ({ ok: false, error });
