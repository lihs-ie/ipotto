variable "project_id" {
  description = "Cloud Run サービスを作成する GCP プロジェクト ID。"
  type        = string
}

variable "region" {
  description = "Cloud Run サービスを作成するリージョン。"
  type        = string
}

variable "service_name" {
  description = "Cloud Run サービス名。"
  type        = string
}

variable "image" {
  description = "デプロイするコンテナイメージ。"
  type        = string
}

variable "cpu" {
  description = "Cloud Run に割り当てる CPU。"
  type        = string
}

variable "memory" {
  description = "Cloud Run に割り当てるメモリ。"
  type        = string
}

variable "min_instance_count" {
  description = "Cloud Run の最小インスタンス数。"
  type        = number
}

variable "max_instance_count" {
  description = "Cloud Run の最大インスタンス数。"
  type        = number
}

variable "timeout_seconds" {
  description = "Cloud Run のタイムアウト秒数。"
  type        = number
}

variable "max_concurrent_requests" {
  description = "Cloud Run の最大同時リクエスト数。"
  type        = number
}

variable "ingress" {
  description = "Cloud Run の ingress 設定。"
  type        = string

  validation {
    condition     = contains(["all", "internal", "internal-and-cloud-load-balancing"], var.ingress)
    error_message = "ingress は all / internal / internal-and-cloud-load-balancing のいずれかを指定してください。"
  }
}

variable "service_account_email" {
  description = "Cloud Run サービスに割り当てるサービスアカウント。"
  type        = string
}

variable "container_port" {
  description = "コンテナが待ち受けるポート。"
  type        = number
}

variable "environment_variables" {
  description = "平文の環境変数。"
  type        = map(string)
  default     = {}
}

variable "secret_environment_variables" {
  description = "Secret Manager 経由で注入する環境変数。"
  type = map(object({
    secret_id = string
    version   = optional(string, "latest")
  }))
  default = {}
}

variable "invoker_members" {
  description = "roles/run.invoker を付与する principal 一覧。"
  type        = set(string)
  default     = []
}

variable "labels" {
  description = "Cloud Run に付与するラベル。"
  type        = map(string)
  default     = {}
}

variable "cpu_idle" {
  description = "アイドル時に CPU を割り当て続けるかどうか。"
  type        = bool
  default     = true
}

variable "deletion_protection" {
  description = "Cloud Run の削除保護を有効にするかどうか。"
  type        = bool
  default     = false
}
