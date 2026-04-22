locals {
  service_filter = length(var.cloud_run_service_names) == 0 ? "" : format(
    " AND (%s)",
    join(" OR ", [
      for service in var.cloud_run_service_names :
      "resource.labels.service_name=\"${service}\""
    ]),
  )
}

// Log-based metric: severity>=ERROR の発生数をサービス別に計上。
// Cloud Logging には既に `severity` が付いているので、追加ラベルなし。
resource "google_logging_metric" "service_error" {
  project = var.project_id
  name    = "ipotto/service_error"
  description = "severity が ERROR 以上のログ件数。ipo-api / ipo-info-fetcher / ipo-applier / ipo-result-checker / ipo-browser が対象。"

  filter = "severity>=ERROR AND resource.type=\"cloud_run_revision\"${local.service_filter}"

  metric_descriptor {
    metric_kind = "DELTA"
    value_type  = "INT64"
    unit        = "1"
    display_name = "IPOtto Cloud Run error logs"
    labels {
      key         = "service_name"
      value_type  = "STRING"
      description = "Cloud Run service name"
    }
  }

  label_extractors = {
    "service_name" = "EXTRACT(resource.labels.service_name)"
  }
}

// Pub/Sub DLQ メッセージ残留を検出するための log metric。
// subscription のデリバリ失敗 (ACK タイムアウト等) を拾う。
resource "google_logging_metric" "dlq_delivery_failed" {
  project     = var.project_id
  name        = "ipotto/dlq_delivery_failed"
  description = "Pub/Sub の delivery 失敗ログ件数。DLQ 到達の兆候として計上。"

  filter = "resource.type=\"pubsub_subscription\" AND severity>=WARNING AND textPayload=~\"(?i)delivery\""

  metric_descriptor {
    metric_kind = "DELTA"
    value_type  = "INT64"
    unit        = "1"
    display_name = "IPOtto Pub/Sub delivery failures"
  }
}

// Email 通知チャネル。
// 実アドレスは terraform.tfvars 経由で差し替える前提。
resource "google_monitoring_notification_channel" "email" {
  for_each = toset(var.alert_email_recipients)

  project      = var.project_id
  display_name = "IPOtto Alerts (${each.value})"
  type         = "email"

  labels = {
    email_address = each.value
  }
}

locals {
  notification_channel_names = [
    for channel in google_monitoring_notification_channel.email : channel.name
  ]
}

// 5xx エラー急増アラート (Cloud Run metric)。
resource "google_monitoring_alert_policy" "cloud_run_5xx" {
  project      = var.project_id
  display_name = "IPOtto Cloud Run 5xx surge"
  combiner     = "OR"
  user_labels  = var.labels

  conditions {
    display_name = "5xx responses >= ${var.error_rate_threshold} in 5 min"
    condition_threshold {
      filter = "resource.type=\"cloud_run_revision\" AND metric.type=\"run.googleapis.com/request_count\" AND metric.labels.response_code_class=\"5xx\""
      duration        = "300s"
      threshold_value = var.error_rate_threshold
      comparison      = "COMPARISON_GT"

      aggregations {
        alignment_period   = "60s"
        per_series_aligner = "ALIGN_RATE"
      }
    }
  }

  notification_channels = local.notification_channel_names
}

// Cloud Run コンテナ再起動ループアラート。
// instance_count が 1 分で大きく増減したら異常。
resource "google_monitoring_alert_policy" "cloud_run_restart_loop" {
  project      = var.project_id
  display_name = "IPOtto Cloud Run restart loop"
  combiner     = "OR"
  user_labels  = var.labels

  conditions {
    display_name = "container restarts >= ${var.restart_threshold} per minute"
    condition_threshold {
      filter = "resource.type=\"cloud_run_revision\" AND metric.type=\"run.googleapis.com/container/instance_count\""
      duration        = "60s"
      threshold_value = var.restart_threshold
      comparison      = "COMPARISON_GT"

      aggregations {
        alignment_period   = "60s"
        per_series_aligner = "ALIGN_DELTA"
      }
    }
  }

  notification_channels = local.notification_channel_names
}

// ERROR ログ流量アラート。
// log-based metric を使って severity>=ERROR が急増したら発火。
resource "google_monitoring_alert_policy" "service_error_logs" {
  project      = var.project_id
  display_name = "IPOtto service ERROR logs"
  combiner     = "OR"
  user_labels  = var.labels

  conditions {
    display_name = "ERROR log rate >= ${var.error_rate_threshold} / 5 min"
    condition_threshold {
      filter = "metric.type=\"logging.googleapis.com/user/ipotto/service_error\" AND resource.type=\"cloud_run_revision\""
      duration        = "300s"
      threshold_value = var.error_rate_threshold
      comparison      = "COMPARISON_GT"

      aggregations {
        alignment_period   = "60s"
        per_series_aligner = "ALIGN_RATE"
      }
    }
  }

  notification_channels = local.notification_channel_names
  depends_on            = [google_logging_metric.service_error]
}

// Pub/Sub DLQ 到達アラート。
resource "google_monitoring_alert_policy" "pubsub_dlq_activity" {
  project      = var.project_id
  display_name = "IPOtto Pub/Sub DLQ activity"
  combiner     = "OR"
  user_labels  = var.labels

  conditions {
    display_name = "DLQ delivery failures detected"
    condition_threshold {
      filter = "metric.type=\"logging.googleapis.com/user/ipotto/dlq_delivery_failed\""
      duration        = "300s"
      threshold_value = 0
      comparison      = "COMPARISON_GT"

      aggregations {
        alignment_period   = "60s"
        per_series_aligner = "ALIGN_RATE"
      }
    }
  }

  notification_channels = local.notification_channel_names
  depends_on            = [google_logging_metric.dlq_delivery_failed]
}
