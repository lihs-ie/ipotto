"use client";

import { useCallback, useEffect, useState, type FormEvent } from "react";

import {
  type GetNotificationSettingResponse,
  type UpdateNotificationChannelInput,
  type UpdateNotificationSettingRequest,
} from "@ipotto/shared";

import { Button } from "@/components/atoms/Button/Button";
import { Typography } from "@/components/atoms/Typography/Typography";
import { FormErrorBanner } from "@/components/molecules/FormErrorBanner/FormErrorBanner";
import { NotificationChannelForm } from "@/components/molecules/NotificationChannelForm/NotificationChannelForm";
import { useIpoApi } from "@/app/client-providers";
import { useAsyncResult } from "@/lib/api/use-async-result";
import { useMutation } from "@/lib/api/use-mutation";

import styles from "./NotificationSettingForm.module.css";

const emptyChannel = (): UpdateNotificationChannelInput => ({
  channelType: "Email",
  destination: {},
  enabled: true,
  subscriptions: {},
});

const toFormValue = (
  setting: GetNotificationSettingResponse,
): UpdateNotificationSettingRequest => ({
  enabled: setting.enabled,
  channels: setting.channels.map((channel) => ({
    channelType: channel.channelType,
    destination: channel.destination,
    enabled: channel.enabled,
    subscriptions: channel.subscriptions,
  })),
});

export const NotificationSettingForm = () => {
  const api = useIpoApi();
  const fetcher = useCallback(() => api.getNotificationSetting(), [api]);
  const { state: fetchState } = useAsyncResult(fetcher, [fetcher]);

  const updateMutation = useMutation(
    useCallback(
      (payload: UpdateNotificationSettingRequest) =>
        api.updateNotificationSetting(payload),
      [api],
    ),
  );

  const [formValue, setFormValue] =
    useState<UpdateNotificationSettingRequest | null>(null);

  useEffect(() => {
    if (fetchState.status === "ok") {
      setFormValue(toFormValue(fetchState.value));
    }
  }, [fetchState]);

  if (fetchState.status === "loading" || formValue === null) {
    return <Typography variant="body">読み込んでいます…</Typography>;
  }
  if (fetchState.status === "error") {
    return (
      <Typography variant="body">
        読み込みに失敗しました: {fetchState.error.kind}
      </Typography>
    );
  }

  const updateChannel = (
    index: number,
    next: UpdateNotificationChannelInput,
  ): void => {
    setFormValue((current) =>
      current === null
        ? current
        : {
            ...current,
            channels: current.channels.map((channel, position) =>
              position === index ? next : channel,
            ),
          },
    );
  };

  const removeChannel = (index: number): void => {
    setFormValue((current) =>
      current === null
        ? current
        : {
            ...current,
            channels: current.channels.filter(
              (_, position) => position !== index,
            ),
          },
    );
  };

  const addChannel = (): void => {
    setFormValue((current) =>
      current === null
        ? current
        : { ...current, channels: current.channels.concat(emptyChannel()) },
    );
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (formValue === null) return;
    await updateMutation.mutate(formValue);
  };

  const error =
    updateMutation.state.status === "error" ? updateMutation.state.error : null;

  return (
    <form className={styles.container} onSubmit={handleSubmit} noValidate>
      <Typography variant="h1">通知設定</Typography>
      <FormErrorBanner error={error} />
      <label className={styles.toggle}>
        <input
          type="checkbox"
          checked={formValue.enabled}
          onChange={(changeEvent) =>
            setFormValue({ ...formValue, enabled: changeEvent.target.checked })
          }
        />
        <span>通知を有効化</span>
      </label>
      <Typography variant="h3">チャネル</Typography>
      {formValue.channels.map((channel, index) => (
        <NotificationChannelForm
          key={index}
          value={channel}
          onChange={(next) => updateChannel(index, next)}
          onRemove={() => removeChannel(index)}
        />
      ))}
      <div className={styles.actions}>
        <Button label="チャネルを追加" variant="secondary" onClick={addChannel} />
        <Button
          label="保存"
          type="submit"
          disabled={updateMutation.state.status === "loading"}
        />
      </div>
      {updateMutation.state.status === "ok" && (
        <Typography variant="caption">
          <span className={styles.savedNotice}>保存しました。</span>
        </Typography>
      )}
    </form>
  );
};
