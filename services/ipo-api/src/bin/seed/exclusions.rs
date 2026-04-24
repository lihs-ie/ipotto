//! Seed data for `exclusions`. Registers two well-known exclusion rules
//! that match the `Excluded` / `Failed` stocks in the stock seed set.

use chrono::{Duration, Utc};
use ipo_backend_shared::{
    domain::{
        exclusion::{Exclusion, ExclusionIdentifier, ExclusionReason},
        stock::CompanyName,
    },
    errors::DomainError,
};
use ulid::Ulid;

pub fn build_exclusions() -> Result<Vec<Exclusion>, DomainError> {
    let now = Utc::now();
    Ok(vec![
        Exclusion::reconstruct(
            exclusion_identifier(1)?,
            CompanyName::new("怪しいバイブコイン株式会社")?,
            ExclusionReason::new("仕手株の噂があるため除外")?,
            now - Duration::days(6),
        )?,
        Exclusion::reconstruct(
            exclusion_identifier(2)?,
            CompanyName::new("フェイクニュース株式会社")?,
            ExclusionReason::new("投資対象として不適格と判断")?,
            now - Duration::days(2),
        )?,
    ])
}

fn exclusion_identifier(index: u8) -> Result<ExclusionIdentifier, DomainError> {
    let ulid = Ulid::from_parts(1_700_000_010_000, u128::from(index));
    ExclusionIdentifier::new(ulid.to_string())
}
