"use client";

import {
  type ChannelDestination,
  type ChannelType,
  type ChannelSubscriptions,
  type NotificationEventType,
  type UpdateNotificationChannelInput,
} from "@ipotto/shared";

import { Button } from "@/components/atoms/Button/Button";
import { Typography } from "@/components/atoms/Typography/Typography";
import { SelectField } from "@/components/molecules/SelectField/SelectField";
import { TextField } from "@/components/molecules/TextField/TextField";

import styles from "./NotificationChannelForm.module.css";

type Props = {
  value: UpdateNotificationChannelInput;
  onChange: (next: UpdateNotificationChannelInput) => void;
  onRemove: () => void;
};

const eventTypes: NotificationEventType[] = [
  "ApplicationCompleted",
  "LotteryResultWon",
  "LotteryResultLost",
  "OperationError",
  "StockUpdated",
];

const destinationFieldsFor = (
  channelType: ChannelType,
): { key: string; label: string; type?: "text" | "password" }[] => {
  if (channelType === "LINE") {
    return [
      { key: "accessToken", label: "アクセストークン", type: "password" },
    ];
  }
  if (channelType === "Email") {
    return [{ key: "address", label: "メールアドレス", type: "text" }];
  }
  return [{ key: "webhookUrl", label: "Webhook URL", type: "text" }];
};

export const NotificationChannelForm = (props: Props) => {
  const updateDestination = (key: string, next: string): void => {
    const destination: ChannelDestination = { ...props.value.destination };
    if (next === "") {
      delete destination[key];
    } else {
      destination[key] = next;
    }
    props.onChange({ ...props.value, destination });
  };

  const updateSubscription = (
    eventType: NotificationEventType,
    subscribed: boolean,
  ): void => {
    const subscriptions: ChannelSubscriptions = { ...props.value.subscriptions };
    if (subscribed) {
      subscriptions[eventType] = true;
    } else {
      delete subscriptions[eventType];
    }
    props.onChange({ ...props.value, subscriptions });
  };

  return (
    <fieldset className={styles.container}>
      <div className={styles.header}>
        <Typography variant="h3">チャネル</Typography>
        <Button label="削除" variant="secondary" onClick={props.onRemove} />
      </div>
      <SelectField
        label="種別"
        value={props.value.channelType}
        onChange={(next) =>
          props.onChange({
            ...props.value,
            channelType: next as ChannelType,
            destination: {},
          })
        }
        options={[
          { label: "LINE", value: "LINE" },
          { label: "Email", value: "Email" },
          { label: "Slack", value: "Slack" },
        ]}
      />
      <label className={styles.toggle}>
        <input
          type="checkbox"
          checked={props.value.enabled}
          onChange={(event) =>
            props.onChange({ ...props.value, enabled: event.target.checked })
          }
        />
        <span>有効</span>
      </label>
      {destinationFieldsFor(props.value.channelType).map((field) => (
        <TextField
          key={field.key}
          label={field.label}
          type={field.type}
          value={props.value.destination[field.key] ?? ""}
          onChange={(next) => updateDestination(field.key, next)}
        />
      ))}
      <Typography variant="caption">購読するイベント</Typography>
      <div className={styles.subscriptions}>
        {eventTypes.map((eventType) => (
          <label key={eventType} className={styles.subscriptionItem}>
            <input
              type="checkbox"
              checked={props.value.subscriptions[eventType] === true}
              onChange={(event) =>
                updateSubscription(eventType, event.target.checked)
              }
            />
            <span>{eventType}</span>
          </label>
        ))}
      </div>
    </fieldset>
  );
};
