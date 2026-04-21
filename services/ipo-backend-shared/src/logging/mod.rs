//! Cross-service logging utilities, primarily secret masking helpers used by
//! the API, info fetcher, result checker, and browser services to keep
//! credentials and other sensitive fields out of structured logs.

mod secret_masking;

pub use secret_masking::{mask_email, mask_prefix, mask_secret};
