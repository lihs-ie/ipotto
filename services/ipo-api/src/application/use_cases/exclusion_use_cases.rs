use std::sync::Arc;

use chrono::Utc;
use ipo_backend_shared::{
    domain::{
        exclusion::{Exclusion, ExclusionIdentifier, ExclusionReason, ExclusionRepository},
        stock::CompanyName,
    },
    errors::DomainError,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterExclusionInput {
    pub company_name: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterExclusionOutput {
    pub identifier: String,
    pub company_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusionOutput {
    pub identifier: String,
    pub company_name: String,
    pub reason: String,
    pub registered_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListExclusionsOutput {
    pub items: Vec<ExclusionOutput>,
    pub total_count: usize,
}

pub struct RegisterExclusionUseCase {
    repository: Arc<dyn ExclusionRepository + Send + Sync>,
}

impl RegisterExclusionUseCase {
    pub fn new(repository: Arc<dyn ExclusionRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        input: RegisterExclusionInput,
    ) -> Result<RegisterExclusionOutput, DomainError> {
        let company_name = CompanyName::new(input.company_name)?;
        if self.repository.exists_by_company_name(&company_name)? {
            return Err(DomainError::DuplicateExclusion {
                company_name: company_name.value().to_string(),
            });
        }
        let exclusion = Exclusion::create(
            company_name.clone(),
            ExclusionReason::new(input.reason)?,
            Utc::now(),
        )?;
        self.repository.save(&exclusion)?;
        Ok(RegisterExclusionOutput {
            identifier: exclusion.identifier().value().to_string(),
            company_name: company_name.value().to_string(),
        })
    }
}

pub struct RemoveExclusionUseCase {
    repository: Arc<dyn ExclusionRepository + Send + Sync>,
}

impl RemoveExclusionUseCase {
    pub fn new(repository: Arc<dyn ExclusionRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub fn execute(&self, identifier: &str) -> Result<(), DomainError> {
        let identifier = ExclusionIdentifier::new(identifier)?;
        let exclusion =
            self.repository
                .find_by_id(&identifier)?
                .ok_or_else(|| DomainError::NotFound {
                    resource: "exclusion".to_string(),
                    identifier: identifier.value().to_string(),
                })?;
        self.repository.delete(exclusion.identifier())
    }
}

pub struct ListExclusionsUseCase {
    repository: Arc<dyn ExclusionRepository + Send + Sync>,
}

impl ListExclusionsUseCase {
    pub fn new(repository: Arc<dyn ExclusionRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<ListExclusionsOutput, DomainError> {
        let mut exclusions = self.repository.find_all()?;
        exclusions.sort_by_key(|exclusion| std::cmp::Reverse(exclusion.registered_at()));
        let items = exclusions
            .into_iter()
            .map(|exclusion| ExclusionOutput {
                identifier: exclusion.identifier().value().to_string(),
                company_name: exclusion.company_name().value().to_string(),
                reason: exclusion.reason().value().to_string(),
                registered_at: exclusion.registered_at().to_rfc3339(),
            })
            .collect::<Vec<_>>();

        Ok(ListExclusionsOutput {
            total_count: items.len(),
            items,
        })
    }
}
