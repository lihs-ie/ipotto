output "database_name" {
  description = "Firestore データベース名。"
  value       = google_firestore_database.this.name
}

output "index_ids" {
  description = "複合インデックス ID 一覧。"
  value = {
    for key, index in google_firestore_index.composite : key => index.id
  }
}
