import { describe, expect, it } from "vitest";

import {
  extractImageAuthenticationKeywords,
  RakutenAuthMailParseError,
} from "./rakuten-auth-mail-parser.js";

describe("extractImageAuthenticationKeywords", () => {
  it("extracts two keywords from a mail body", () => {
    const result = extractImageAuthenticationKeywords(`
楽天証券からのお知らせ
さくら + みかん
有効期限は2分です
`);

    expect(result).toEqual({
      firstKeyword: "さくら",
      secondKeyword: "みかん",
    });
  });

  it("throws when the keyword line is not found", () => {
    expect(() =>
      extractImageAuthenticationKeywords("楽天証券からのお知らせ"),
    ).toThrow(RakutenAuthMailParseError);
  });

  it("throws when only one keyword exists", () => {
    expect(() => extractImageAuthenticationKeywords("さくら")).toThrow(
      RakutenAuthMailParseError,
    );
  });

  it("throws when more than two keywords exist", () => {
    expect(() =>
      extractImageAuthenticationKeywords("さくら + みかん + りんご"),
    ).toThrow(RakutenAuthMailParseError);
  });
});
