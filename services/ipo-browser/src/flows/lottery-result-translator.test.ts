import { describe, expect, it } from "vitest";

import { translateLotteryResultText } from "./lottery-result-translator.js";

describe("translateLotteryResultText", () => {
  it("maps '当選' to Won", () => {
    expect(translateLotteryResultText("当選")).toBe("Won");
  });

  it("maps '落選' to Lost", () => {
    expect(translateLotteryResultText("落選")).toBe("Lost");
  });

  it("maps '補欠' to Alternate", () => {
    expect(translateLotteryResultText("補欠")).toBe("Alternate");
  });

  it("maps '補欠当選' to Alternate (not Won)", () => {
    expect(translateLotteryResultText("補欠当選")).toBe("Alternate");
  });

  it("maps '未発表' to null", () => {
    expect(translateLotteryResultText("未発表")).toBeNull();
  });

  it("maps '抽選前' to null", () => {
    expect(translateLotteryResultText("抽選前")).toBeNull();
  });

  it("returns null when input is null, undefined or empty", () => {
    expect(translateLotteryResultText(null)).toBeNull();
    expect(translateLotteryResultText(undefined)).toBeNull();
    expect(translateLotteryResultText("")).toBeNull();
    expect(translateLotteryResultText("   ")).toBeNull();
  });

  it("returns null for unknown text rather than throwing", () => {
    expect(translateLotteryResultText("その他")).toBeNull();
  });
});
