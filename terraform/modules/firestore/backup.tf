resource "google_firestore_backup_schedule" "daily" {
  provider = google-beta

  project  = var.project_id
  database = google_firestore_database.this.name

  retention = var.backup_daily_retention

  daily_recurrence {}

  depends_on = [google_firestore_database.this]
}

resource "google_firestore_backup_schedule" "weekly" {
  provider = google-beta

  project  = var.project_id
  database = google_firestore_database.this.name

  retention = var.backup_weekly_retention

  weekly_recurrence {
    day = "SUNDAY"
  }

  depends_on = [google_firestore_database.this]
}
