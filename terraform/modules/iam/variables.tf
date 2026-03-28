variable "project_id" {
  description = "IAM リソースを作成する GCP プロジェクト ID。"
  type        = string
}

variable "service_accounts" {
  description = "作成するサービスアカウント定義。"
  type = map(object({
    account_id    = string
    display_name  = string
    description   = optional(string, null)
    project_roles = set(string)
  }))
}
