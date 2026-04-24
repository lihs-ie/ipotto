variable "project_id" {
  description = "Cloud Scheduler ジョブを作成する GCP プロジェクト ID。"
  type        = string
}

variable "region" {
  description = "Cloud Scheduler ジョブを作成するリージョン。"
  type        = string
}

variable "jobs" {
  description = "Cloud Scheduler ジョブ定義。"
  type = map(object({
    name             = string
    description      = optional(string, "")
    schedule         = string
    time_zone        = string
    topic_id         = string
    payload          = string
    attributes       = optional(map(string), {})
    attempt_deadline = optional(string, "320s")
  }))
  default = {}
}
