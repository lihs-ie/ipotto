import { monotonicFactory } from "ulid";

const generate = monotonicFactory();

/**
 * Generates a ULID string.
 */
export function generateUlid(): string {
  return generate();
}
