output "secret_ids" {
  description = "シークレット ID の一覧。"
  value = {
    for key, secret in google_secret_manager_secret.this : key => secret.secret_id
  }
}

output "secret_names" {
  description = "シークレットのフルリソース名一覧。"
  value = {
    for key, secret in google_secret_manager_secret.this : key => secret.name
  }
}
