resource "google_secret_manager_secret" "this" {
  for_each = var.secrets

  project   = var.project_id
  secret_id = each.value.secret_id
  labels    = each.value.labels

  replication {
    auto {}
  }
}
