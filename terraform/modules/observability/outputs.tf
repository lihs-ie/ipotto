output "service_error_metric_id" {
  description = "Log-based metric のリソース ID (severity>=ERROR 件数)。"
  value       = google_logging_metric.service_error.id
}

output "dlq_delivery_failed_metric_id" {
  description = "Log-based metric のリソース ID (Pub/Sub DLQ delivery failure)。"
  value       = google_logging_metric.dlq_delivery_failed.id
}

output "notification_channel_names" {
  description = "作成された Email 通知チャネルのフルリソース名。"
  value       = local.notification_channel_names
}

output "alert_policy_ids" {
  description = "アラートポリシーの ID マップ (policy 名 → policy id)。"
  value = {
    cloud_run_5xx         = google_monitoring_alert_policy.cloud_run_5xx.id
    cloud_run_restart_loop = google_monitoring_alert_policy.cloud_run_restart_loop.id
    service_error_logs    = google_monitoring_alert_policy.service_error_logs.id
    pubsub_dlq_activity   = google_monitoring_alert_policy.pubsub_dlq_activity.id
  }
}
