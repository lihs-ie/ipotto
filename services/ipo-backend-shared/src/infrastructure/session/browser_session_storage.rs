use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use crate::{domain::account::SecuritiesAccountIdentifier, errors::DomainError};

/// Manages persistent browser session directories.
#[derive(Debug, Clone)]
pub struct BrowserSessionStorage {
    base_directory: PathBuf,
}

impl BrowserSessionStorage {
    /// Creates browser session storage.
    pub fn new(base_directory: impl Into<PathBuf>) -> Self {
        Self {
            base_directory: base_directory.into(),
        }
    }

    /// Returns the user data directory for an account.
    pub fn get_user_data_directory(&self, account_id: &SecuritiesAccountIdentifier) -> PathBuf {
        self.base_directory.join(account_id.value())
    }

    /// Returns whether a session directory exists for the account.
    pub fn session_exists(&self, account_id: &SecuritiesAccountIdentifier) -> bool {
        self.get_user_data_directory(account_id).exists()
    }

    /// Deletes expired session directories and returns the deletion count.
    pub fn cleanup_expired_sessions(&self, max_age_hours: u32) -> Result<u32, DomainError> {
        let threshold = Duration::from_secs(u64::from(max_age_hours) * 60 * 60);
        let now = SystemTime::now();
        let mut deleted = 0;
        if !self.base_directory.exists() {
            return Ok(0);
        }

        for entry in
            fs::read_dir(&self.base_directory).map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?
        {
            let entry = entry.map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?;
            let metadata = entry
                .metadata()
                .map_err(|error| DomainError::SecretPayloadError {
                    reason: error.to_string(),
                })?;
            let modified =
                metadata
                    .modified()
                    .map_err(|error| DomainError::SecretPayloadError {
                        reason: error.to_string(),
                    })?;
            if now.duration_since(modified).unwrap_or_default() > threshold {
                fs::remove_dir_all(entry.path()).map_err(|error| {
                    DomainError::SecretPayloadError {
                        reason: error.to_string(),
                    }
                })?;
                deleted += 1;
            }
        }
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::BrowserSessionStorage;
    use crate::domain::account::SecuritiesAccountIdentifier;

    #[test]
    fn builds_account_specific_paths() {
        let temp = tempdir().expect("tempdir");
        let storage = BrowserSessionStorage::new(temp.path());
        let account_id = SecuritiesAccountIdentifier::generate();
        let path = storage.get_user_data_directory(&account_id);
        assert!(path.ends_with(account_id.value()));
    }

    #[test]
    fn detects_existing_session() {
        let temp = tempdir().expect("tempdir");
        let storage = BrowserSessionStorage::new(temp.path());
        let account_id = SecuritiesAccountIdentifier::generate();
        fs::create_dir_all(storage.get_user_data_directory(&account_id)).expect("mkdir");
        assert!(storage.session_exists(&account_id));
    }
}
