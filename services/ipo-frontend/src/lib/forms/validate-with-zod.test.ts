import { z } from "zod";
import { describe, expect, it } from "vitest";

import { validateWithZod } from "./validate-with-zod";

describe("validateWithZod", () => {
  const schema = z.object({
    email: z.string().email("有効なメールアドレスを入力してください"),
    age: z.number().int().positive("年齢は正の整数で入力してください"),
  });

  it("returns ok + value on success", () => {
    const result = validateWithZod(schema, {
      email: "user@example.com",
      age: 30,
    });
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.email).toBe("user@example.com");
    }
  });

  it("returns field-keyed error map on failure", () => {
    const result = validateWithZod(schema, { email: "not-email", age: -1 });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.errors.email).toBeTypeOf("string");
      expect(result.errors.age).toBeTypeOf("string");
    }
  });

  it("groups root-level errors under _root", () => {
    const rootSchema = z.string().min(1, "空不可");
    const result = validateWithZod(rootSchema, "");
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.errors._root).toBe("空不可");
    }
  });
});
