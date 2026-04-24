variable "project_id" {
  description = "デプロイ先の GCP プロジェクト ID。"
  type        = string
}

variable "region" {
  description = "デプロイ先のリージョン。"
  type        = string
}

variable "environment" {
  description = "環境名。"
  type        = string
}

variable "enabled_apis" {
  description = "有効化する Google API 一覧。"
  type        = set(string)
  default = [
    "artifactregistry.googleapis.com",
    "cloudbuild.googleapis.com",
    "cloudkms.googleapis.com",
    "cloudscheduler.googleapis.com",
    "firebase.googleapis.com",
    "firestore.googleapis.com",
    "iam.googleapis.com",
    "logging.googleapis.com",
    "monitoring.googleapis.com",
    "pubsub.googleapis.com",
    "run.googleapis.com",
    "secretmanager.googleapis.com",
  ]
}

variable "credential_kms" {
  description = "Secret Manager に保存する証券口座クレデンシャルを envelope 暗号化するための KMS 設定。"
  type = object({
    location                         = string
    key_ring_name                    = string
    crypto_key_name                  = string
    rotation_period                  = optional(string, "7776000s")
    encrypter_decrypter_account_keys = set(string)
  })
}

variable "artifact_registry" {
  description = "Artifact Registry の設定。"
  type = object({
    repository_id = string
    format        = optional(string, "DOCKER")
    description   = optional(string, "IPOtto container images")
  })
}

variable "service_accounts" {
  description = "サービスアカウント定義。"
  type = map(object({
    account_id    = string
    display_name  = string
    description   = optional(string, null)
    project_roles = set(string)
  }))
}

variable "cloud_run_services" {
  description = "Cloud Run サービス定義。"
  type = map(object({
    service_name            = string
    service_account_key     = string
    container_port          = number
    cpu                     = string
    memory                  = string
    min_instance_count      = number
    max_instance_count      = number
    timeout_seconds         = number
    max_concurrent_requests = number
    ingress                 = string
    image                   = optional(string)
    environment_variables   = optional(map(string), {})
    secret_environment_variables = optional(map(object({
      secret_id = string
      version   = optional(string, "latest")
    })), {})
    invoker_members = optional(set(string), [])
  }))
}

variable "pubsub_topics" {
  description = "Pub/Sub トピックとサブスクリプション定義。"
  type = map(object({
    name   = string
    labels = optional(map(string), {})
    subscriptions = optional(map(object({
      name                       = string
      ack_deadline_seconds       = optional(number, 10)
      message_retention_duration = optional(string, "604800s")
      retain_acked_messages      = optional(bool, false)
      enable_message_ordering    = optional(bool, false)
      expiration_policy_ttl      = optional(string)
      dead_letter_topic          = optional(string)
      max_delivery_attempts      = optional(number, 5)
      filter                     = optional(string)
    })), {})
  }))
  default = {}
}

variable "scheduler_jobs" {
  description = "Cloud Scheduler ジョブ定義。"
  type = map(object({
    name             = string
    description      = optional(string, "")
    schedule         = string
    time_zone        = string
    topic_key        = string
    payload          = string
    attributes       = optional(map(string), {})
    attempt_deadline = optional(string, "320s")
  }))
  default = {}
}

variable "secret_containers" {
  description = "作成する Secret Manager のシークレットコンテナ。"
  type = map(object({
    secret_id = string
    labels    = optional(map(string), {})
  }))
  default = {}
}

variable "enable_firestore" {
  description = "Firestore データベースを作成するかどうか。"
  type        = bool
  default     = true
}

variable "firestore" {
  description = "Firestore 設定。"
  type = object({
    location_id      = string
    database_name    = optional(string, "(default)")
    database_type    = optional(string, "FIRESTORE_NATIVE")
    concurrency_mode = optional(string, "OPTIMISTIC")
    composite_indexes = optional(map(object({
      collection  = string
      query_scope = optional(string, "COLLECTION")
      fields = list(object({
        field_path   = string
        order        = optional(string)
        array_config = optional(string)
      }))
    })), {})
  })
}

variable "observability" {
  description = "Cloud Monitoring / Logging 設定。"
  type = object({
    alert_email_recipients = list(string)
    error_rate_threshold   = optional(number, 10)
    restart_threshold      = optional(number, 3)
  })
}
