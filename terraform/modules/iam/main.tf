locals {
  service_account_roles = flatten([
    for account_key, definition in var.service_accounts : [
      for role in definition.project_roles : {
        key        = "${account_key}-${replace(role, "/", "_")}"
        account_id = google_service_account.this[account_key].email
        role       = role
      }
    ]
  ])
}

resource "google_service_account" "this" {
  for_each = var.service_accounts

  project      = var.project_id
  account_id   = each.value.account_id
  display_name = each.value.display_name
  description  = each.value.description
}

resource "google_project_iam_member" "service_account_roles" {
  for_each = {
    for item in local.service_account_roles : item.key => item
  }

  project = var.project_id
  role    = each.value.role
  member  = "serviceAccount:${each.value.account_id}"
}
