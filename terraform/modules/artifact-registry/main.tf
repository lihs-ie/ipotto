resource "google_artifact_registry_repository" "this" {
  project       = var.project_id
  location      = var.region
  repository_id = var.repository_id
  format        = var.format
  description   = var.description
  labels        = var.labels
}
