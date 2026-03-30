use crate::domain::account::SecuritiesAccountIdentifier;

/// Returns the secret name for securities account credentials.
pub fn account_credential_secret_name(account_id: &SecuritiesAccountIdentifier) -> String {
    format!("ipo-account-{}", account_id.value())
}

/// Returns the secret name for Gmail OAuth refresh payloads.
pub fn gmail_oauth_secret_name(account_id: &SecuritiesAccountIdentifier) -> String {
    format!("ipo-gmail-oauth-{}", account_id.value())
}

/// Returns the shared SendGrid API key secret name.
pub const fn sendgrid_api_key_secret_name() -> &'static str {
    "ipo-sendgrid-api-key"
}

#[cfg(test)]
mod tests {
    use crate::domain::account::SecuritiesAccountIdentifier;

    use super::{
        account_credential_secret_name, gmail_oauth_secret_name, sendgrid_api_key_secret_name,
    };

    #[test]
    fn builds_expected_secret_names() {
        let account_id = SecuritiesAccountIdentifier::generate();
        assert!(account_credential_secret_name(&account_id).starts_with("ipo-account-"));
        assert!(gmail_oauth_secret_name(&account_id).starts_with("ipo-gmail-oauth-"));
        assert_eq!(sendgrid_api_key_secret_name(), "ipo-sendgrid-api-key");
    }
}
