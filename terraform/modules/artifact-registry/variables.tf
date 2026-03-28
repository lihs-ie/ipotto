variable "project_id" {
  description = "Artifact Registry を作成する GCP プロジェクト ID。"
  type        = string
}

variable "region" {
  description = "Artifact Registry を作成するリージョン。"
  type        = string
}

variable "repository_id" {
  description = "Artifact Registry のリポジトリ ID。"
  type        = string
}

variable "format" {
  description = "Artifact Registry のフォーマット。"
  type        = string
  default     = "DOCKER"
}

variable "description" {
  description = "Artifact Registry の説明。"
  type        = string
  default     = "IPOtto container images"
}

variable "labels" {
  description = "Artifact Registry に付与するラベル。"
  type        = map(string)
  default     = {}
}
