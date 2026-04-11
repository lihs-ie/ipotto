import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { detectApplicationResultFromText } from "./application-result-detector.js";

const currentDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(currentDirectory, "../../../../../");
const referenceRoot = resolve(repositoryRoot, "docs/reference");
const mockFixtureRoot = resolve(
  currentDirectory,
  "../../../tests/fixtures/html/rakuten",
);

/**
 * Reads a Rakuten reference HTML fixture.
 */
function readReferenceHtml(...segments: string[]): string {
  return readFileSync(resolve(referenceRoot, ...segments), "utf8");
}

/**
 * Reads a local mock Rakuten HTML fixture.
 */
function readMockHtml(fileName: string): string {
  return readFileSync(resolve(mockFixtureRoot, fileName), "utf8");
}

describe("Rakuten reference HTML contracts", () => {
  it("matches dashboard markers used for login success detection", () => {
    const html = readReferenceHtml(
      "楽天証券",
      "ログイン成功後のダッシュボード画面.html",
    );

    expect(html).toContain("<title>ホーム | 楽天証券[PC]</title>");
    expect(html).toContain('id="ratPageName" value="[member]/app/home.do"');
    expect(html).toContain("/app/home.do");
  });

  it("matches image authentication markers used for additional authentication detection", () => {
    const html = readReferenceHtml("楽天証券", "画像認証画面.html");

    expect(html).toContain("<title>多要素認証設定 | 楽天証券[PC]</title>");
    expect(html).toContain("ログイン追加認証");
    expect(html).toContain("認証コード画像選択");
    expect(html).toContain("pcmm_emoji-img");
  });

  it("matches resend markers used on the expired image authentication page", () => {
    const html = readReferenceHtml("楽天証券", "画像認証失敗時の画面.html");

    expect(html).toContain("認証コードが失効しました");
    expect(html).toContain("認証コードを再送信する");
    expect(html).toContain("pcmm_emoji-art");
  });

  it("matches selectors used on the IPO list page", () => {
    const html = readReferenceHtml("楽天証券", "IPO一覧画面.html");

    expect(html).toContain('class="pcmm_ipolt-ipo-block"');
    expect(html).toContain('class="pcmm_ipolt-ipo-block__stockname"');
    expect(html).toContain("ブックビルディング申込");
    expect(html).toContain("銘柄詳細を見る");
  });

  it("matches selectors used on the already-applied IPO list page", () => {
    const html = readReferenceHtml(
      "楽天証券",
      "IPO一覧画面（申込後が含まれる）.html",
    );

    expect(html).toContain("ブックビルディング申込済");
    expect(html).toContain("申込詳細を見る");
  });

  it("matches selectors used on the IPO application pages", () => {
    const cautionHtml = readReferenceHtml("楽天証券", "IPO申し込み入力画面", "1.html");
    const inputHtml = readReferenceHtml(
      "楽天証券",
      "IPO申し込み入力画面",
      "2. 同意して次へを押した後.html",
    );
    const confirmHtml = readReferenceHtml(
      "楽天証券",
      "IPO申し込み入力画面",
      "3. 申し込み内容を確認するを押した後.html",
    );

    expect(cautionHtml).toContain("同意して次へ");
    expect(inputHtml).toContain('id="orderValueInput"');
    expect(inputHtml).toContain('id="priceSpinnerComBox"');
    expect(inputHtml).toContain("申込内容を確認する");
    expect(confirmHtml).toContain('id="passwordInputText"');
    expect(confirmHtml).toContain("申し込む");
  });
});

describe("detectApplicationResultFromText", () => {
  it("detects success from real Rakuten HTML", () => {
    const html = readReferenceHtml("楽天証券", "IPO申し込み成功画面.html");

    expect(detectApplicationResultFromText(html, "application failed")).toEqual({
      status: "success",
    });
  });

  it("detects already applied from real Rakuten HTML", () => {
    const html = readReferenceHtml("楽天証券", "IPO申し込み済み詳細.html");

    expect(detectApplicationResultFromText(html, "application failed")).toEqual({
      status: "already_applied",
    });
  });

  it("detects insufficient balance from the mock HTML fixture", () => {
    const html = readMockHtml("ipo_application_insufficient.html");

    expect(detectApplicationResultFromText(html, "application failed")).toEqual({
      status: "insufficient_balance",
    });
  });

  it("returns a failure with the supplied reason when no markers match", () => {
    expect(
      detectApplicationResultFromText(
        "申し込みに失敗しました",
        "申し込みに失敗しました",
      ),
    ).toEqual({
      status: "failure",
      reason: "申し込みに失敗しました",
    });
  });
});
