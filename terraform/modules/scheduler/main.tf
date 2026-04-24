resource "google_cloud_scheduler_job" "this" {
  for_each = var.jobs

  project          = var.project_id
  region           = var.region
  name             = each.value.name
  description      = each.value.description
  schedule         = each.value.schedule
  time_zone        = each.value.time_zone
  attempt_deadline = each.value.attempt_deadline

  pubsub_target {
    topic_name = each.value.topic_id
    data       = base64encode(each.value.payload)
    attributes = each.value.attributes
  }
}
