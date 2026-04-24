output "job_names" {
  description = "Cloud Scheduler ジョブ名一覧。"
  value = {
    for key, job in google_cloud_scheduler_job.this : key => job.name
  }
}
