variable "project_id" {
  description = "Firestore データベースを作成する GCP プロジェクト ID。"
  type        = string
}

variable "database_name" {
  description = "Firestore データベース名。"
  type        = string
  default     = "(default)"
}

variable "location_id" {
  description = "Firestore データベースのリージョン。"
  type        = string
}

variable "database_type" {
  description = "Firestore データベースタイプ。"
  type        = string
  default     = "FIRESTORE_NATIVE"
}

variable "concurrency_mode" {
  description = "Firestore の同時実行制御モード。"
  type        = string
  default     = "OPTIMISTIC"
}

variable "composite_indexes" {
  description = "作成する複合インデックス定義。"
  type = map(object({
    collection  = string
    query_scope = optional(string, "COLLECTION")
    fields = list(object({
      field_path   = string
      order        = optional(string)
      array_config = optional(string)
    }))
  }))
  default = {}
}
