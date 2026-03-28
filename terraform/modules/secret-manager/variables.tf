variable "project_id" {
  description = "Secret Manager を作成する GCP プロジェクト ID。"
  type        = string
}

variable "secrets" {
  description = "作成するシークレットコンテナ定義。"
  type = map(object({
    secret_id = string
    labels    = optional(map(string), {})
  }))
  default = {}
}
