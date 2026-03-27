output "service_name" {
  description = "Cloud Run サービス名。"
  value       = google_cloud_run_v2_service.this.name
}

output "service_url" {
  description = "Cloud Run サービス URL。"
  value       = google_cloud_run_v2_service.this.uri
}

output "service_id" {
  description = "Cloud Run サービス ID。"
  value       = google_cloud_run_v2_service.this.id
}
