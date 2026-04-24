import { describe, expect, it } from "vitest";

import {
  applicationIdentifierSchema,
  channelIdentifierSchema,
  exclusionIdentifierSchema,
  notificationSettingIdentifierSchema,
  operationLogIdentifierSchema,
  securitiesAccountIdentifierSchema,
  stockIdentifierSchema,
} from "./identifiers";

describe("identifier schemas", () => {
  const schemas = [
    { name: "stock", schema: stockIdentifierSchema },
    { name: "application", schema: applicationIdentifierSchema },
    { name: "exclusion", schema: exclusionIdentifierSchema },
    {
      name: "securitiesAccount",
      schema: securitiesAccountIdentifierSchema,
    },
    {
      name: "notificationSetting",
      schema: notificationSettingIdentifierSchema,
    },
    { name: "channel", schema: channelIdentifierSchema },
    { name: "operationLog", schema: operationLogIdentifierSchema },
  ];

  for (const { name, schema } of schemas) {
    it(`${name} identifier accepts non-empty string`, () => {
      expect(schema.parse("abc_123")).toBe("abc_123");
    });

    it(`${name} identifier rejects empty string`, () => {
      expect(() => schema.parse("")).toThrow();
    });
  }
});
