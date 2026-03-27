locals {
  ingress_map = {
    all                               = "INGRESS_TRAFFIC_ALL"
    internal                          = "INGRESS_TRAFFIC_INTERNAL_ONLY"
    internal-and-cloud-load-balancing = "INGRESS_TRAFFIC_INTERNAL_LOAD_BALANCER"
  }
}

resource "google_cloud_run_v2_service" "this" {
  provider = google-beta

  project             = var.project_id
  location            = var.region
  name                = var.service_name
  ingress             = local.ingress_map[var.ingress]
  deletion_protection = var.deletion_protection
  labels              = var.labels

  template {
    service_account                  = var.service_account_email
    timeout                          = "${var.timeout_seconds}s"
    max_instance_request_concurrency = var.max_concurrent_requests

    scaling {
      min_instance_count = var.min_instance_count
      max_instance_count = var.max_instance_count
    }

    containers {
      image = var.image

      ports {
        container_port = var.container_port
      }

      resources {
        limits = {
          cpu    = var.cpu
          memory = var.memory
        }
        cpu_idle = var.cpu_idle
      }

      dynamic "env" {
        for_each = var.environment_variables

        content {
          name  = env.key
          value = env.value
        }
      }

      dynamic "env" {
        for_each = var.secret_environment_variables

        content {
          name = env.key

          value_source {
            secret_key_ref {
              secret  = env.value.secret_id
              version = env.value.version
            }
          }
        }
      }
    }
  }

  traffic {
    percent = 100
    type    = "TRAFFIC_TARGET_ALLOCATION_TYPE_LATEST"
  }
}

resource "google_cloud_run_v2_service_iam_member" "invoker" {
  provider = google-beta

  for_each = var.invoker_members

  project  = var.project_id
  location = var.region
  name     = google_cloud_run_v2_service.this.name
  role     = "roles/run.invoker"
  member   = each.value
}
