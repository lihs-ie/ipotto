//! Seed data helpers for `securities_accounts`.
//!
//! The seed binary does not persist a `SecuritiesAccount` aggregate.
//! `SecuritiesAccount`'s credential payload must live in the credential
//! store (Secret Manager in production, `InMemoryCredentialStore` in
//! dev), and the in-memory store is process-local — seeding from a
//! separate process would leave the running ipo-api without access to
//! the credential bytes, making every `find_all` / `find_active` call
//! fail with `secret not found`.
//!
//! To preserve lottery application linkage, this module exposes a
//! stable placeholder identifier that seeded applications reference;
//! registering a real account from the UI creates a fresh identifier
//! without affecting the seeded history.

use ipo_backend_shared::{domain::account::SecuritiesAccountIdentifier, errors::DomainError};
use ulid::Ulid;

pub fn placeholder_account_identifier() -> Result<SecuritiesAccountIdentifier, DomainError> {
    let ulid = Ulid::from_parts(1_700_000_020_000, 1);
    SecuritiesAccountIdentifier::new(ulid.to_string())
}
