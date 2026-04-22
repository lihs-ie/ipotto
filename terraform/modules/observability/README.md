# terraform/modules/observability

IPOtto の Cloud Monitoring / Cloud Logging を宣言的に管理するモジュール。

## 提供リソース

| 種類 | 名称 | 用途 |
|---|---|---|
| `google_logging_metric` | `ipotto/service_error` | Cloud Run サービスの `severity>=ERROR` ログ件数を計上 |
| `google_logging_metric` | `ipotto/dlq_delivery_failed` | Pub/Sub の delivery 失敗ログを計上 (DLQ 到達の兆候) |
| `google_monitoring_notification_channel` | `email` | 運用アラートの通知先 (メール) |
| `google_monitoring_alert_policy` | `IPOtto Cloud Run 5xx surge` | 5 分間の 5xx レスポンスが閾値超で通知 |
| `google_monitoring_alert_policy` | `IPOtto Cloud Run restart loop` | コンテナ再起動回数が 1 分で閾値超で通知 |
| `google_monitoring_alert_policy` | `IPOtto service ERROR logs` | `ipotto/service_error` 発生率が閾値超で通知 |
| `google_monitoring_alert_policy` | `IPOtto Pub/Sub DLQ activity` | `ipotto/dlq_delivery_failed` が発生したら通知 |

## 入力変数

| 変数 | 既定値 | 説明 |
|---|---|---|
| `project_id` | 必須 | GCP プロジェクト ID |
| `alert_email_recipients` | 必須 (1 件以上) | アラート通知先メール |
| `cloud_run_service_names` | `[]` | log metric のサービス絞り込み (空なら全サービス対象) |
| `dlq_topic_name` | `ipo-dead-letter` | DLQ topic (今後のサブスクリプション監視に利用) |
| `error_rate_threshold` | `10` | 5xx / ERROR ログ件数の 5 分間閾値 |
| `restart_threshold` | `3` | コンテナ再起動回数の 1 分間閾値 |
| `labels` | `{}` | 全ポリシーに付与するラベル |

## 使い方

```hcl
module "observability" {
  source = "../../modules/observability"

  project_id              = var.project_id
  alert_email_recipients  = var.observability.alert_email_recipients
  cloud_run_service_names = [for key, service in var.cloud_run_services : service.service_name]
  dlq_topic_name          = "ipo-dead-letter"
  labels                  = local.common_labels
}
```

## 運用ノート

- 通知チャネルは `email_address` ラベル単位で作成される。誤送信を避けるため、実運用アドレスは `terraform.tfvars` で一元管理する。
- Log-based metric は発生時 0〜数秒で集計されるが、alert が fire するまで最低 1 minute 程度 lag がある。
- Pub/Sub DLQ アラートは現状 "delivery" を含む WARN ログをトリガにしている。将来的には DLQ subscription の `num_undelivered_messages` 直接参照に置き換える。
