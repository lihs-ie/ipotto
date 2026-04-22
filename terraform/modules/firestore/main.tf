resource "google_firestore_database" "this" {
  provider = google-beta

  project                     = var.project_id
  name                        = var.database_name
  location_id                 = var.location_id
  type                        = var.database_type
  concurrency_mode            = var.concurrency_mode
  app_engine_integration_mode = "DISABLED"
  delete_protection_state     = "DELETE_PROTECTION_DISABLED"
}

resource "google_firebaserules_ruleset" "firestore" {
  count   = var.firestore_rules_path != null ? 1 : 0
  provider = google-beta

  project = var.project_id
  source {
    files {
      name    = "firestore.rules"
      content = file(var.firestore_rules_path)
    }
  }

  depends_on = [google_firestore_database.this]
}

resource "google_firebaserules_release" "firestore" {
  count   = var.firestore_rules_path != null ? 1 : 0
  provider = google-beta

  project      = var.project_id
  name         = "cloud.firestore/${google_firestore_database.this.name}"
  ruleset_name = google_firebaserules_ruleset.firestore[0].name

  depends_on = [google_firebaserules_ruleset.firestore]
}

resource "google_firestore_index" "composite" {
  provider = google-beta

  for_each = var.composite_indexes

  project     = var.project_id
  database    = var.database_name
  collection  = each.value.collection
  query_scope = each.value.query_scope

  dynamic "fields" {
    for_each = each.value.fields

    content {
      field_path   = fields.value.field_path
      order        = try(fields.value.order, null)
      array_config = try(fields.value.array_config, null)
    }
  }

  depends_on = [google_firestore_database.this]
}
