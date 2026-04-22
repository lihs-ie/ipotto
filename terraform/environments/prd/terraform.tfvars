project_id  = "ipotto-prd"
region      = "asia-northeast1"
environment = "prd"

artifact_registry = {
  repository_id = "ipotto"
}

service_accounts = {
  frontend = {
    account_id   = "ipo-frontend"
    display_name = "IPOtto Frontend"
    project_roles = [
      "roles/logging.logWriter",
      "roles/monitoring.metricWriter",
    ]
  }
  api = {
    account_id   = "ipo-api"
    display_name = "IPOtto API"
    project_roles = [
      "roles/datastore.user",
      "roles/logging.logWriter",
      "roles/monitoring.metricWriter",
      "roles/secretmanager.secretAccessor",
    ]
  }
  browser = {
    account_id   = "ipo-browser"
    display_name = "IPOtto Browser"
    project_roles = [
      "roles/datastore.user",
      "roles/logging.logWriter",
      "roles/monitoring.metricWriter",
      "roles/pubsub.publisher",
      "roles/secretmanager.secretAccessor",
    ]
  }
  info_fetcher = {
    account_id   = "ipo-info-fetcher"
    display_name = "IPOtto Info Fetcher"
    project_roles = [
      "roles/datastore.user",
      "roles/logging.logWriter",
      "roles/monitoring.metricWriter",
      "roles/pubsub.publisher",
    ]
  }
  result_checker = {
    account_id   = "ipo-result-checker"
    display_name = "IPOtto Result Checker"
    project_roles = [
      "roles/datastore.user",
      "roles/logging.logWriter",
      "roles/monitoring.metricWriter",
      "roles/pubsub.publisher",
      "roles/secretmanager.secretAccessor",
    ]
  }
}

cloud_run_services = {
  frontend = {
    service_name            = "ipo-frontend"
    service_account_key     = "frontend"
    container_port          = 3000
    cpu                     = "1"
    memory                  = "256Mi"
    min_instance_count      = 0
    max_instance_count      = 1
    timeout_seconds         = 300
    max_concurrent_requests = 80
    ingress                 = "all"
    environment_variables   = {}
    invoker_members         = ["allUsers"]
  }
  api = {
    service_name            = "ipo-api"
    service_account_key     = "api"
    container_port          = 8080
    cpu                     = "1"
    memory                  = "256Mi"
    min_instance_count      = 0
    max_instance_count      = 2
    timeout_seconds         = 300
    max_concurrent_requests = 80
    ingress                 = "internal-and-cloud-load-balancing"
    environment_variables = {
      RUST_LOG = "info"
    }
    secret_environment_variables = {
      ALLOWED_EMAIL = {
        secret_id = "allowed-email"
      }
    }
  }
  browser = {
    service_name            = "ipo-browser"
    service_account_key     = "browser"
    container_port          = 8081
    cpu                     = "2"
    memory                  = "1Gi"
    min_instance_count      = 0
    max_instance_count      = 1
    timeout_seconds         = 600
    max_concurrent_requests = 1
    ingress                 = "internal"
    environment_variables   = {}
  }
  info_fetcher = {
    service_name            = "ipo-info-fetcher"
    service_account_key     = "info_fetcher"
    container_port          = 8082
    cpu                     = "1"
    memory                  = "256Mi"
    min_instance_count      = 0
    max_instance_count      = 1
    timeout_seconds         = 300
    max_concurrent_requests = 1
    ingress                 = "internal"
    environment_variables = {
      RUST_LOG = "info"
    }
  }
  result_checker = {
    service_name            = "ipo-result-checker"
    service_account_key     = "result_checker"
    container_port          = 8083
    cpu                     = "1"
    memory                  = "256Mi"
    min_instance_count      = 0
    max_instance_count      = 1
    timeout_seconds         = 300
    max_concurrent_requests = 1
    ingress                 = "internal"
    environment_variables = {
      RUST_LOG = "info"
    }
  }
}

pubsub_topics = {
  ipo_job_trigger = {
    name = "ipo-job-trigger"
    subscriptions = {
      info_fetch_sub = {
        name                  = "ipo-info-fetch-sub"
        dead_letter_topic     = "ipo-job-trigger-dlq"
        max_delivery_attempts = 3
      }
      apply_sub = {
        name                  = "ipo-apply-sub"
        dead_letter_topic     = "ipo-job-trigger-dlq"
        max_delivery_attempts = 3
      }
      result_check_sub = {
        name                  = "ipo-result-check-sub"
        dead_letter_topic     = "ipo-job-trigger-dlq"
        max_delivery_attempts = 3
      }
    }
  }
  ipo_info_updated = {
    name = "ipo-info-updated"
    subscriptions = {
      notify_sub = {
        name                  = "ipo-info-updated-notify-sub"
        dead_letter_topic     = "ipo-info-updated-dlq"
        max_delivery_attempts = 3
      }
    }
  }
  ipo_result_updated = {
    name = "ipo-result-updated"
    subscriptions = {
      notify_sub = {
        name                  = "ipo-result-updated-notify-sub"
        dead_letter_topic     = "ipo-result-updated-dlq"
        max_delivery_attempts = 3
      }
    }
  }
  ipo_notification = {
    name = "ipo-notification"
    subscriptions = {
      notify_sub = {
        name                  = "ipo-notification-sub"
        dead_letter_topic     = "ipo-notification-dlq"
        max_delivery_attempts = 5
      }
    }
  }
}

scheduler_jobs = {
  ipo_daily_trigger = {
    name        = "ipo-daily-trigger"
    description = "毎日9時にIPO関連ジョブをトリガーする"
    schedule    = "0 9 * * *"
    time_zone   = "Asia/Tokyo"
    topic_key   = "ipo_job_trigger"
    payload     = "{\"trigger\":\"daily\"}"
    attributes = {
      source = "cloud-scheduler"
    }
  }
}

secret_containers = {
  allowed_email = {
    secret_id = "allowed-email"
  }
  sendgrid_api_key = {
    secret_id = "sendgrid-api-key"
  }
  slack_webhook_url = {
    secret_id = "slack-webhook-url"
  }
  line_notify_token = {
    secret_id = "line-notify-token"
  }
  imap_password = {
    secret_id = "imap-password"
  }
}

observability = {
  alert_email_recipients = ["ops@example.com"]
  error_rate_threshold   = 10
  restart_threshold      = 3
}

enable_firestore = true

firestore = {
  location_id = "asia-northeast1"
  composite_indexes = {
    ipo_stocks_status_updated_at = {
      collection = "ipo_stocks"
      fields = [
        {
          field_path = "status"
          order      = "ASCENDING"
        },
        {
          field_path = "updatedAt"
          order      = "DESCENDING"
        },
      ]
    }
    ipo_stocks_bb_period = {
      collection = "ipo_stocks"
      fields = [
        {
          field_path = "schedule.bookBuildingPeriod.startDate"
          order      = "ASCENDING"
        },
        {
          field_path = "schedule.bookBuildingPeriod.endDate"
          order      = "ASCENDING"
        },
      ]
    }
    lottery_applications_status_created_at = {
      collection = "lottery_applications"
      fields = [
        {
          field_path = "status"
          order      = "ASCENDING"
        },
        {
          field_path = "createdAt"
          order      = "DESCENDING"
        },
      ]
    }
    operation_logs_event_type_executed_at = {
      collection = "operation_logs"
      fields = [
        {
          field_path = "eventType"
          order      = "ASCENDING"
        },
        {
          field_path = "executedAt"
          order      = "DESCENDING"
        },
      ]
    }
  }
}
