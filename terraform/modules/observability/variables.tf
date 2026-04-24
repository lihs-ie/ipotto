variable "project_id" {
  description = "GCP プロジェクト ID。Cloud Logging / Cloud Monitoring リソースの所属先。"
  type        = string
}

variable "alert_email_recipients" {
  description = "アラート通知先のメールアドレス一覧。最低 1 件必要。"
  type        = list(string)
  validation {
    condition     = length(var.alert_email_recipients) > 0
    error_message = "alert_email_recipients には最低 1 件のメールアドレスを指定してください。"
  }
}

variable "cloud_run_service_names" {
  description = "監視対象の Cloud Run サービス名。5xx 急増 / restart ループのアラート対象。"
  type        = list(string)
  default     = []
}

variable "dlq_topic_name" {
  description = "Pub/Sub Dead-Letter topic 名。未ack メッセージ残存アラートの対象。"
  type        = string
  default     = "ipo-dead-letter"
}

variable "labels" {
  description = "全 observability リソースに共通して付与するラベル。"
  type        = map(string)
  default     = {}
}

variable "error_rate_threshold" {
  description = "5 分間の 5xx 発生数がこの値以上になるとアラート発火。"
  type        = number
  default     = 10
}

variable "restart_threshold" {
  description = "1 分間のコンテナ再起動回数がこの値以上になるとアラート発火。"
  type        = number
  default     = 3
}

variable "audit_log_services" {
  description = "Cloud Audit Data Access logs を有効化する GCP サービス一覧。"
  type        = list(string)
  default = [
    "secretmanager.googleapis.com",
    "firestore.googleapis.com",
    "storage.googleapis.com",
  ]
}
