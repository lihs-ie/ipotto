locals {
  common_labels = {
    environment = var.environment
    managed-by  = "terraform"
    project     = "ipotto"
  }

  scheduler_jobs = {
    for key, job in var.scheduler_jobs : key => merge(job, {
      topic_id = module.pubsub.topic_ids[job.topic_key]
    })
  }
}

resource "google_project_service" "enabled" {
  for_each = var.enabled_apis

  project = var.project_id
  service = each.value

  disable_dependent_services = false
  disable_on_destroy         = false
}

module "artifact_registry" {
  source = "../../modules/artifact-registry"

  project_id    = var.project_id
  region        = var.region
  repository_id = var.artifact_registry.repository_id
  format        = var.artifact_registry.format
  description   = var.artifact_registry.description
  labels        = local.common_labels

  depends_on = [google_project_service.enabled]
}

module "iam" {
  source = "../../modules/iam"

  project_id       = var.project_id
  service_accounts = var.service_accounts

  depends_on = [google_project_service.enabled]
}

module "secret_manager" {
  source = "../../modules/secret-manager"

  project_id = var.project_id
  secrets    = var.secret_containers

  depends_on = [google_project_service.enabled]
}

module "pubsub" {
  source = "../../modules/pubsub"

  project_id = var.project_id
  topics     = var.pubsub_topics

  depends_on = [google_project_service.enabled]
}

module "scheduler" {
  source = "../../modules/scheduler"

  project_id = var.project_id
  region     = var.region
  jobs       = local.scheduler_jobs

  depends_on = [module.pubsub]
}

module "firestore" {
  count  = var.enable_firestore ? 1 : 0
  source = "../../modules/firestore"

  project_id            = var.project_id
  location_id           = var.firestore.location_id
  database_name         = var.firestore.database_name
  database_type         = var.firestore.database_type
  concurrency_mode      = var.firestore.concurrency_mode
  composite_indexes     = var.firestore.composite_indexes
  firestore_rules_path   = "${path.module}/../../../firestore.rules"
  backup_daily_retention = "259200s"
  backup_weekly_retention = "604800s"

  depends_on = [google_project_service.enabled]
}

module "observability" {
  source = "../../modules/observability"

  project_id              = var.project_id
  alert_email_recipients  = var.observability.alert_email_recipients
  cloud_run_service_names = [for key, service in var.cloud_run_services : service.service_name]
  error_rate_threshold    = var.observability.error_rate_threshold
  restart_threshold       = var.observability.restart_threshold
  labels                  = local.common_labels

  depends_on = [google_project_service.enabled]
}

module "cloud_run" {
  for_each = var.cloud_run_services
  source   = "../../modules/cloud-run"

  project_id                   = var.project_id
  region                       = var.region
  service_name                 = each.value.service_name
  image                        = coalesce(each.value.image, "${module.artifact_registry.repository_url}/${each.value.service_name}:latest")
  cpu                          = each.value.cpu
  memory                       = each.value.memory
  min_instance_count           = each.value.min_instance_count
  max_instance_count           = each.value.max_instance_count
  timeout_seconds              = each.value.timeout_seconds
  max_concurrent_requests      = each.value.max_concurrent_requests
  ingress                      = each.value.ingress
  service_account_email        = module.iam.service_account_emails[each.value.service_account_key]
  container_port               = each.value.container_port
  environment_variables        = each.value.environment_variables
  secret_environment_variables = each.value.secret_environment_variables
  invoker_members              = each.value.invoker_members
  labels                       = local.common_labels

  depends_on = [
    google_project_service.enabled,
    module.artifact_registry,
    module.iam,
    module.secret_manager,
  ]
}
