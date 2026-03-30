/// Firestore client configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirestoreClientConfig {
    project_id: String,
}

impl FirestoreClientConfig {
    /// Creates a Firestore client configuration.
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
        }
    }

    /// Returns the project identifier.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
}
