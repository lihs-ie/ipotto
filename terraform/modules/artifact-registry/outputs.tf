output "repository_id" {
  description = "Artifact Registry リポジトリ ID。"
  value       = google_artifact_registry_repository.this.repository_id
}

output "repository_name" {
  description = "Artifact Registry リポジトリ名。"
  value       = google_artifact_registry_repository.this.name
}

output "repository_url" {
  description = "Docker イメージ参照に使うベース URL。"
  value       = "${var.region}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.this.repository_id}"
}
