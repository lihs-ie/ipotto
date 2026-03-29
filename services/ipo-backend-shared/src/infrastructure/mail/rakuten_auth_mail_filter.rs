use chrono::{DateTime, Utc};

/// Returns whether the mail matches Rakuten image authentication mail conditions.
pub fn is_rakuten_auth_mail(
    from: &str,
    subject: &str,
    received_at: DateTime<Utc>,
    received_after: DateTime<Utc>,
) -> bool {
    from.contains("rakuten") && subject.contains("認証") && received_at >= received_after
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::is_rakuten_auth_mail;

    #[test]
    fn matches_rakuten_auth_mail() {
        let now = Utc::now();
        assert!(is_rakuten_auth_mail(
            "notice@rakuten-sec.co.jp",
            "画像認証のお知らせ",
            now,
            now - Duration::seconds(1)
        ));
    }
}
