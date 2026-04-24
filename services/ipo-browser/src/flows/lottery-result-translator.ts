// Translates Rakuten Securities lottery result page text into the
// LotteryResult enum expected by ipo-api/ipo-result-checker.
//
// The Rust side mirror is `translate_lottery_result` in
// `docs/03-detailed-design/acl.md` §3.5 (see also
// `ipo_backend_shared::domain::application::LotteryResult`).

export type LotteryOutcome = "Won" | "Lost" | "Alternate" | null;

export function translateLotteryResultText(raw: string | null | undefined): LotteryOutcome {
  if (raw === null || raw === undefined) {
    return null;
  }
  const trimmed = raw.trim();
  if (trimmed === "") {
    return null;
  }
  if (trimmed.includes("補欠")) {
    return "Alternate";
  }
  if (trimmed.includes("当選")) {
    return "Won";
  }
  if (trimmed.includes("落選")) {
    return "Lost";
  }
  if (trimmed.includes("未発表") || trimmed.includes("抽選前")) {
    return null;
  }
  return null;
}
