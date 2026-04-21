// Translates Rakuten Securities IPO apply result page text into the
// ApplicationResult discriminated union expected by the ipo-api backend.
//
// The keyword table mirrors `translate_application_result` in
// `services/ipo-backend-shared/src/acl/browser/application_result.rs`
// and `docs/03-detailed-design/acl.md` §3.5. Keeping the two in
// lockstep is required so the Rust BrowserServiceClient and TypeScript
// route agree on semantics for every response page reachable in the
// html-mock-server fixtures and on the real broker site.

export type ApplyResult =
  | { readonly status: "success" }
  | { readonly status: "failure"; readonly reason: string }
  | { readonly status: "already_applied" }
  | { readonly status: "insufficient_balance" };

export function translateApplyResultText(raw: string): ApplyResult {
  const trimmed = raw.trim();
  if (trimmed.includes("受け付けました") || trimmed.includes("完了")) {
    return { status: "success" };
  }
  if (trimmed.includes("既に申込済み")) {
    return { status: "already_applied" };
  }
  if (trimmed.includes("残高") && trimmed.includes("不足")) {
    return { status: "insufficient_balance" };
  }
  return { status: "failure", reason: trimmed };
}
