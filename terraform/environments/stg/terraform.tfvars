project_id  = "ipotto-stg"
region      = "asia-northeast1"
environment = "stg"

artifact_registry = {
  repository_id = "ipotto"
}

service_accounts = {
  frontend = {
    account_id   = "ipo-frontend"
    display_name = "IPOtto Frontend STG"
    project_roles = [
      "roles/logging.logWriter",
      "roles/monitoring.metricWriter",
    ]
  }
  api = {
    account_id   = "ipo-api"
    display_name = "IPOtto API STG"
    project_roles = [
      "roles/logging.logWriter",
      "roles/monitoring.metricWriter",
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
    environment_variables = {}
    invoker_members = ["allUsers"]
  }
  api = {
    service_name            = "ipo-api"
    service_account_key     = "api"
    container_port          = 8080
    cpu                     = "1"
    memory                  = "256Mi"
    min_instance_count      = 0
    max_instance_count      = 1
    timeout_seconds         = 300
    max_concurrent_requests = 80
    ingress                 = "all"
    environment_variables = {
      RUST_LOG = "info"
    }
  }
}

pubsub_topics     = {}
scheduler_jobs    = {}
secret_containers = {}

enable_firestore = false

firestore = {
  location_id       = "asia-northeast1"
  composite_indexes = {}
}
