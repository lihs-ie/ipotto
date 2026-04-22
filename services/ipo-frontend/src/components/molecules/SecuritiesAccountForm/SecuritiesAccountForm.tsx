"use client";

import { useState, type FormEvent } from "react";

import {
  createSecuritiesAccountRequestSchema,
  updateSecuritiesAccountRequestSchema,
  type CreateSecuritiesAccountRequest,
  type UpdateSecuritiesAccountRequest,
} from "@ipotto/shared";

import { Button } from "@/components/atoms/Button/Button";
import { Typography } from "@/components/atoms/Typography/Typography";
import { SelectField } from "@/components/molecules/SelectField/SelectField";
import { TextField } from "@/components/molecules/TextField/TextField";
import { validateWithZod } from "@/lib/forms/validate-with-zod";

import styles from "./SecuritiesAccountForm.module.css";

type CreateMode = {
  mode: "create";
  pending: boolean;
  onSubmit: (payload: CreateSecuritiesAccountRequest) => Promise<void>;
};

type UpdateMode = {
  mode: "update";
  pending: boolean;
  onSubmit: (payload: UpdateSecuritiesAccountRequest) => Promise<void>;
};

type Props = CreateMode | UpdateMode;

type FormState = {
  securitiesCompany: string;
  loginId: string;
  loginPassword: string;
  tradingPassword: string;
  mailAddress: string;
  mailPassword: string;
  imapHost: string;
  imapPort: string;
  active: boolean;
};

const emptyState = (): FormState => ({
  securitiesCompany: "Rakuten",
  loginId: "",
  loginPassword: "",
  tradingPassword: "",
  mailAddress: "",
  mailPassword: "",
  imapHost: "",
  imapPort: "993",
  active: true,
});

export const SecuritiesAccountForm = (props: Props) => {
  const [state, setState] = useState<FormState>(emptyState());
  const [errors, setErrors] = useState<Record<string, string>>({});

  const setField = <K extends keyof FormState>(
    key: K,
    value: FormState[K],
  ): void => {
    setState((previous) => ({ ...previous, [key]: value }));
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (props.mode === "create") {
      const parsed = validateWithZod(createSecuritiesAccountRequestSchema, {
        securitiesCompany: state.securitiesCompany,
        loginId: state.loginId,
        loginPassword: state.loginPassword,
        tradingPassword: state.tradingPassword,
        mailAddress: state.mailAddress,
        mailPassword: state.mailPassword,
        imapHost: state.imapHost,
        imapPort: Number(state.imapPort),
      });
      if (!parsed.ok) {
        setErrors(parsed.errors);
        return;
      }
      setErrors({});
      await props.onSubmit(parsed.value);
      return;
    }
    const partial: Record<string, unknown> = {};
    if (state.loginId !== "") partial.loginId = state.loginId;
    if (state.loginPassword !== "") partial.loginPassword = state.loginPassword;
    if (state.tradingPassword !== "")
      partial.tradingPassword = state.tradingPassword;
    if (state.mailAddress !== "") partial.mailAddress = state.mailAddress;
    if (state.mailPassword !== "") partial.mailPassword = state.mailPassword;
    if (state.imapHost !== "") partial.imapHost = state.imapHost;
    if (state.imapPort !== "") partial.imapPort = Number(state.imapPort);
    partial.active = state.active;
    const parsed = validateWithZod(
      updateSecuritiesAccountRequestSchema,
      partial,
    );
    if (!parsed.ok) {
      setErrors(parsed.errors);
      return;
    }
    setErrors({});
    await props.onSubmit(parsed.value);
  };

  return (
    <form className={styles.container} onSubmit={handleSubmit} noValidate>
      <Typography variant="h3">
        {props.mode === "create" ? "証券口座を登録" : "証券口座を編集"}
      </Typography>
      {props.mode === "create" && (
        <SelectField
          label="証券会社"
          value={state.securitiesCompany}
          onChange={(next) => setField("securitiesCompany", next)}
          options={[{ label: "楽天証券", value: "Rakuten" }]}
          required
          error={errors.securitiesCompany}
        />
      )}
      <TextField
        label="ログインID"
        value={state.loginId}
        onChange={(next) => setField("loginId", next)}
        required={props.mode === "create"}
        error={errors.loginId}
      />
      <TextField
        label="ログインパスワード"
        type="password"
        value={state.loginPassword}
        onChange={(next) => setField("loginPassword", next)}
        required={props.mode === "create"}
        error={errors.loginPassword}
      />
      <TextField
        label="取引パスワード"
        type="password"
        value={state.tradingPassword}
        onChange={(next) => setField("tradingPassword", next)}
        required={props.mode === "create"}
        error={errors.tradingPassword}
      />
      <TextField
        label="メールアドレス"
        type="email"
        value={state.mailAddress}
        onChange={(next) => setField("mailAddress", next)}
        required={props.mode === "create"}
        error={errors.mailAddress}
      />
      <TextField
        label="メールパスワード"
        type="password"
        value={state.mailPassword}
        onChange={(next) => setField("mailPassword", next)}
        required={props.mode === "create"}
        error={errors.mailPassword}
      />
      <TextField
        label="IMAPホスト"
        value={state.imapHost}
        onChange={(next) => setField("imapHost", next)}
        required={props.mode === "create"}
        error={errors.imapHost}
      />
      <TextField
        label="IMAPポート"
        type="number"
        value={state.imapPort}
        onChange={(next) => setField("imapPort", next)}
        required={props.mode === "create"}
        error={errors.imapPort}
      />
      {props.mode === "update" && (
        <label className={styles.toggle}>
          <input
            type="checkbox"
            checked={state.active}
            onChange={(event) => setField("active", event.target.checked)}
          />
          <span>有効</span>
        </label>
      )}
      <Button
        label={props.mode === "create" ? "登録" : "更新"}
        type="submit"
        disabled={props.pending}
      />
    </form>
  );
};
