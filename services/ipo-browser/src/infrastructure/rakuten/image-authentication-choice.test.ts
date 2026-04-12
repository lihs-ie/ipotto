import { describe, expect, it } from "vitest";

import {
  buildImageAuthenticationChoice,
  extractAltLabelFromOnclick,
  extractImageCodeFromSource,
  findMatchingImageAuthenticationChoice,
} from "./image-authentication-choice.js";

describe("image-authentication-choice", () => {
  it("extracts an alt label from emojiAltClick handlers", () => {
    expect(
      extractAltLabelFromOnclick(
        "emojiAltClick('emoji_1','1','/emoji/1F62C.gif','さくら')",
      ),
    ).toBe("さくら");
  });

  it("extracts an image code from the asset path", () => {
    expect(
      extractImageCodeFromSource(
        "/sotp/v200/assets/emoji/conv/20260322/gif/1F97D.gif",
      ),
    ).toBe("1F97D");
  });

  it("falls back to the asset code when the real page has no alt text", () => {
    const choice = buildImageAuthenticationChoice({
      buttonId: "emoji_0",
      buttonIndex: 0,
      onclick:
        "emojiClick('emoji_0','0','/sotp/v200/assets/emoji/conv/20260322/gif/1F97D.gif')",
      imageAlt: null,
      imageSrc: "/sotp/v200/assets/emoji/conv/20260322/gif/1F97D.gif",
    });

    expect(choice.labels).toEqual(["1F97D"]);
  });

  it("matches by alt text when available", () => {
    const choice = buildImageAuthenticationChoice({
      buttonId: null,
      buttonIndex: 0,
      onclick: null,
      imageAlt: "さくら",
      imageSrc: "#",
    });

    expect(
      findMatchingImageAuthenticationChoice([choice], "さくら"),
    ).toEqual(choice);
  });

  it("normalizes an empty button id to null", () => {
    const choice = buildImageAuthenticationChoice({
      buttonId: "",
      buttonIndex: 0,
      onclick: null,
      imageAlt: "さくら",
      imageSrc: "#",
    });

    expect(choice.buttonId).toBeNull();
  });

  it("matches by asset code for the real Rakuten page", () => {
    const choice = buildImageAuthenticationChoice({
      buttonId: "emoji_0",
      buttonIndex: 0,
      onclick:
        "emojiClick('emoji_0','0','/sotp/v200/assets/emoji/conv/20260322/gif/1F97D.gif')",
      imageAlt: null,
      imageSrc: "/sotp/v200/assets/emoji/conv/20260322/gif/1F97D.gif",
    });

    expect(
      findMatchingImageAuthenticationChoice([choice], "1f97d"),
    ).toEqual(choice);
  });
});
