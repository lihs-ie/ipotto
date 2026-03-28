variable "project_id" {
  description = "Pub/Sub リソースを作成する GCP プロジェクト ID。"
  type        = string
}

variable "topics" {
  description = "トピックとサブスクリプションの定義。"
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
