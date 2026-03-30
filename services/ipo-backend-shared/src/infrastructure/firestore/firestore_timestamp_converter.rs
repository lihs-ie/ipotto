use chrono::{DateTime, Utc};

/// Converts chrono timestamps to a Firestore-friendly representation.
pub fn to_rfc3339(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

/// Parses a Firestore-friendly timestamp representation.
pub fn from_rfc3339(value: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(value).map(|datetime| datetime.with_timezone(&Utc))
}
