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
}

pubsub_topics     = {}
scheduler_jobs    = {}
secret_containers = {}

enable_firestore = false

firestore = {
  location_id       = "asia-northeast1"
  composite_indexes = {}
}
