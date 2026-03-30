use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        exclusion::{Exclusion, ExclusionIdentifier, ExclusionReason},
        stock::CompanyName,
    },
    errors::DomainError,
};

/// Firestore document model for `Exclusion`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExclusionDocument {
    pub identifier: String,
    pub company_name: String,
    pub reason: String,
    pub registered_at: DateTime<Utc>,
}

impl ExclusionDocument {
    pub fn from_domain(exclusion: &Exclusion) -> Self {
        Self {
            identifier: exclusion.identifier().value().to_string(),
            company_name: exclusion.company_name().value().to_string(),
            reason: exclusion.reason().value().to_string(),
            registered_at: exclusion.registered_at(),
        }
    }

    pub fn to_domain(&self) -> Result<Exclusion, DomainError> {
        Exclusion::reconstruct(
            ExclusionIdentifier::new(self.identifier.clone())?,
            CompanyName::new(self.company_name.clone())?,
            ExclusionReason::new(self.reason.clone())?,
            self.registered_at,
        )
    }
}
