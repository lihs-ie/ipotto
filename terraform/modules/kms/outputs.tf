output "key_ring_id" {
  description = "KeyRing のフルリソース名。"
  value       = google_kms_key_ring.this.id
}

output "crypto_key_id" {
  description = "CryptoKey のフルリソース名。"
  value       = google_kms_crypto_key.this.id
}

output "crypto_key_name" {
  description = "CryptoKey のリソースパス (projects/.../cryptoKeys/...)。環境変数で利用する。"
  value       = google_kms_crypto_key.this.id
}
