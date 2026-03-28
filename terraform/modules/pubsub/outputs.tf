output "topic_ids" {
  description = "トピック ID 一覧。"
  value = {
    for key, topic in google_pubsub_topic.topics : key => topic.id
  }
}

output "topic_names" {
  description = "トピック名一覧。"
  value = {
    for key, topic in google_pubsub_topic.topics : key => topic.name
  }
}

output "subscription_names" {
  description = "サブスクリプション名一覧。"
  value = {
    for key, subscription in google_pubsub_subscription.subscriptions : key => subscription.name
  }
}
