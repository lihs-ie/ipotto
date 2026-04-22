project_id  = "ipotto-stg"
region      = "asia-northeast1"
environment = "stg"

artifact_registry = {
  repository_id = "ipotto"
}

service_accounts = {
  api = {
    account_id   = "ipo-api"
    display_name = "IPOtto API STG"
    project_roles = [
      "roles/logging.logWriter",
      "roles/monitoring.metricWriter",
      "roles/secretmanager.secretAccessor",
    ]
  }
  applier = {
    account_id   = "ipo-applier"
    display_name = "IPOtto Applier STG"
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
  api = {
    service_name            = "ipo-api"
    service_account_key     = "api"
    image                   = "asia-northeast1-docker.pkg.dev/ipotto-stg/ipotto/ipo-api:d88e799a13137f801fd5d956aadb89dab4499743-amd64"
    container_port          = 8080
    cpu                     = "1"
    memory                  = "256Mi"
    min_instance_count      = 0
    max_instance_count      = 1
    timeout_seconds         = 300
    max_concurrent_requests = 80
    ingress                 = "internal"
    environment_variables = {
      RUST_LOG = "info"
    }
  }
  applier = {
    service_name            = "ipo-applier"
    service_account_key     = "applier"
    container_port          = 8084
    cpu                     = "1"
    memory                  = "256Mi"
    min_instance_count      = 0
    max_instance_count      = 1
    timeout_seconds         = 600
    max_concurrent_requests = 1
    ingress                 = "internal"
    environment_variables = {
      RUST_LOG = "info"
    }
  }
}

pubsub_topics = {
  job_trigger = {
    name = "ipo-job-trigger"
    subscriptions = {
      info_fetch = {
        name                  = "ipo-info-fetch-sub"
        ack_deadline_seconds  = 60
        dead_letter_topic     = "ipo-dead-letter"
        max_delivery_attempts = 5
      }
      apply = {
        name                  = "ipo-apply-sub"
        ack_deadline_seconds  = 60
        dead_letter_topic     = "ipo-dead-letter"
        max_delivery_attempts = 5
      }
      result_check = {
        name                  = "ipo-result-check-sub"
        ack_deadline_seconds  = 60
        dead_letter_topic     = "ipo-dead-letter"
        max_delivery_attempts = 5
      }
    }
  }
  info_updated = {
    name = "ipo-info-updated"
    subscriptions = {
      notify = {
        name                  = "ipo-info-updated-notify-sub"
        ack_deadline_seconds  = 60
        dead_letter_topic     = "ipo-dead-letter"
        max_delivery_attempts = 5
      }
    }
  }
  result_updated = {
    name = "ipo-result-updated"
    subscriptions = {
      notify = {
        name                  = "ipo-result-updated-notify-sub"
        ack_deadline_seconds  = 60
        dead_letter_topic     = "ipo-dead-letter"
        max_delivery_attempts = 5
      }
    }
  }
  notification = {
    name = "ipo-notification"
    subscriptions = {
      fanout = {
        name                  = "ipo-notification-sub"
        ack_deadline_seconds  = 60
        dead_letter_topic     = "ipo-dead-letter"
        max_delivery_attempts = 5
      }
    }
  }
}
scheduler_jobs    = {}
secret_containers = {}

observability = {
  alert_email_recipients = ["ops@example.com"]
  error_rate_threshold   = 10
  restart_threshold      = 3
}

enable_firestore = false

firestore = {
  location_id       = "asia-northeast1"
  composite_indexes = {}
}
