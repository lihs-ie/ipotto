use serde::{Deserialize, Serialize};

/// Result reported by the browser service after attempting an IPO lottery
/// application on a broker site.
///
/// The variants mirror `docs/03-detailed-design/acl.md` §3.5
/// `translate_application_result`; keep the JSON tag / payload shape in
/// lockstep with `BrowserApplyResponse` in `ipo-api` and the
/// `ApplyResult` union in `services/ipo-browser/src/flows/apply.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApplicationResult {
    Success,
    Failure { reason: String },
    AlreadyApplied,
    InsufficientBalance,
}

/// Parses a raw result page message produced by the broker site and maps it to
/// an `ApplicationResult`. The keyword table is the canonical
/// `translate_application_result` defined in `docs/03-detailed-design/acl.md`
/// §3.5 and must stay in sync with the TypeScript translator in
/// `services/ipo-browser/src/flows/apply-result-translator.ts`.
pub fn translate_application_result(raw_message: &str) -> ApplicationResult {
    if raw_message.contains("受け付けました") || raw_message.contains("完了") {
        ApplicationResult::Success
    } else if raw_message.contains("既に申込済み") {
        ApplicationResult::AlreadyApplied
    } else if raw_message.contains("残高") && raw_message.contains("不足") {
        ApplicationResult::InsufficientBalance
    } else {
        ApplicationResult::Failure {
            reason: raw_message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_success_keyword() {
        assert_eq!(
            translate_application_result("お申込みを受け付けました"),
            ApplicationResult::Success
        );
        assert_eq!(
            translate_application_result("申込完了しました"),
            ApplicationResult::Success
        );
    }

    #[test]
    fn translates_already_applied_keyword() {
        assert_eq!(
            translate_application_result("既に申込済みです"),
            ApplicationResult::AlreadyApplied
        );
    }

    #[test]
    fn translates_insufficient_balance_keyword() {
        assert_eq!(
            translate_application_result("残高が不足しています"),
            ApplicationResult::InsufficientBalance
        );
    }

    #[test]
    fn falls_back_to_failure_for_unknown_text() {
        assert_eq!(
            translate_application_result("システムエラーが発生しました"),
            ApplicationResult::Failure {
                reason: "システムエラーが発生しました".to_string(),
            }
        );
    }
}
