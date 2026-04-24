output "artifact_registry_repository_url" {
  description = "Artifact Registry のベース URL。"
  value       = module.artifact_registry.repository_url
}

output "service_account_emails" {
  description = "サービスアカウントのメールアドレス一覧。"
  value       = module.iam.service_account_emails
}

output "cloud_run_service_urls" {
  description = "Cloud Run サービス URL 一覧。"
  value = {
    for key, service in module.cloud_run : key => service.service_url
  }
}

output "pubsub_topics" {
  description = "Pub/Sub トピック名一覧。"
  value       = module.pubsub.topic_names
}

output "secret_names" {
  description = "Secret Manager シークレット名一覧。"
  value       = module.secret_manager.secret_names
}

output "firestore_database_name" {
  description = "Firestore データベース名。"
  value       = var.enable_firestore ? module.firestore[0].database_name : null
}

output "credential_kek_name" {
  description = "証券口座クレデンシャル envelope 暗号化用 KEK のフルリソース名。Cloud Run に IPOTTO_CREDENTIAL_KEK_NAME として配線する。"
  value       = module.credential_kms.crypto_key_name
}
