import { describe, expect, it } from "vitest";

import { parseImageAuthenticationKeyword } from "./keyword-extractor.js";

describe("parseImageAuthenticationKeyword", () => {
  it("extracts the keyword pair around an ASCII plus", () => {
    expect(parseImageAuthenticationKeyword("みかん + りんご を選択")).toEqual({
      first: "みかん",
      second: "りんご",
    });
  });

  it("tolerates leading prose in the mail body", () => {
    const body = [
      "平素は楽天証券をご利用いただきありがとうございます。",
      "以下のキーワードに対応する画像をタップしてください。",
      "",
      "ぶどう + いちご",
      "",
      "なお、このコードの有効期限は 2 分です。",
    ].join("\n");
    expect(parseImageAuthenticationKeyword(body)).toEqual({
      first: "ぶどう",
      second: "いちご",
    });
  });

  it("accepts multiple whitespace characters around the plus sign", () => {
    expect(parseImageAuthenticationKeyword("ばなな  +\tすいか")).toEqual({
      first: "ばなな",
      second: "すいか",
    });
  });

  it("returns null when the body does not contain the pattern", () => {
    expect(parseImageAuthenticationKeyword("no keyword here")).toBeNull();
  });

  it("returns null when only one side of the pair is present", () => {
    expect(parseImageAuthenticationKeyword("only one +")).toBeNull();
    expect(parseImageAuthenticationKeyword("+ second")).toBeNull();
  });

  it("picks up the first occurrence when the body contains multiple pairs", () => {
    expect(
      parseImageAuthenticationKeyword("みかん + りんご\nぶどう + いちご"),
    ).toEqual({ first: "みかん", second: "りんご" });
  });
});
