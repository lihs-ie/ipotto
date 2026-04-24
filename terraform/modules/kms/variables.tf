variable "project_id" {
  description = "KMS KeyRing を作成する GCP プロジェクト ID。"
  type        = string
}

variable "location" {
  description = "KeyRing を配置するロケーション。"
  type        = string
}

variable "key_ring_name" {
  description = "KeyRing 名。"
  type        = string
}

variable "crypto_key_name" {
  description = "CryptoKey 名。"
  type        = string
}

variable "rotation_period" {
  description = "CryptoKey の自動ローテーション間隔 (例: 7776000s = 90 日)。"
  type        = string
  default     = "7776000s"
}

variable "labels" {
  description = "CryptoKey に付与するラベル。"
  type        = map(string)
  default     = {}
}

variable "encrypter_decrypter_members" {
  description = "cryptoKeyEncrypterDecrypter ロールを付与するメンバー一覧。"
  type        = set(string)
  default     = []
}
