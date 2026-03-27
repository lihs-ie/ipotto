output "service_account_emails" {
  description = "サービスアカウントのメールアドレス一覧。"
  value = {
    for key, account in google_service_account.this : key => account.email
  }
}

output "service_account_names" {
  description = "サービスアカウントのリソース名一覧。"
  value = {
    for key, account in google_service_account.this : key => account.name
  }
}
