locals {
  dead_letter_topic_names = toset(compact(distinct(flatten([
    for _, topic in var.topics : [
      for _, subscription in topic.subscriptions : try(subscription.dead_letter_topic, null)
    ]
  ]))))

  subscriptions = flatten([
    for topic_key, topic in var.topics : [
      for subscription_key, subscription in topic.subscriptions : {
        key        = "${topic_key}-${subscription_key}"
        topic_key  = topic_key
        definition = subscription
      }
    ]
  ])
}

resource "google_pubsub_topic" "topics" {
  for_each = var.topics

  project = var.project_id
  name    = each.value.name
  labels  = each.value.labels
}

resource "google_pubsub_topic" "dead_letter_topics" {
  for_each = local.dead_letter_topic_names

  project = var.project_id
  name    = each.value
}

resource "google_pubsub_subscription" "subscriptions" {
  for_each = {
    for subscription in local.subscriptions : subscription.key => subscription
  }

  project                    = var.project_id
  name                       = each.value.definition.name
  topic                      = google_pubsub_topic.topics[each.value.topic_key].id
  ack_deadline_seconds       = each.value.definition.ack_deadline_seconds
  message_retention_duration = each.value.definition.message_retention_duration
  retain_acked_messages      = each.value.definition.retain_acked_messages
  enable_message_ordering    = each.value.definition.enable_message_ordering
  filter                     = each.value.definition.filter

  dynamic "expiration_policy" {
    for_each = each.value.definition.expiration_policy_ttl == null ? [] : [each.value.definition.expiration_policy_ttl]

    content {
      ttl = expiration_policy.value
    }
  }

  dynamic "dead_letter_policy" {
    for_each = each.value.definition.dead_letter_topic == null ? [] : [each.value.definition]

    content {
      dead_letter_topic     = google_pubsub_topic.dead_letter_topics[dead_letter_policy.value.dead_letter_topic].id
      max_delivery_attempts = dead_letter_policy.value.max_delivery_attempts
    }
  }
}
