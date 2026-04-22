"use client";

import { useState, type FormEvent } from "react";

import {
  createExclusionRequestSchema,
  type CreateExclusionRequest,
} from "@ipotto/shared";

import { Button } from "@/components/atoms/Button/Button";
import { Typography } from "@/components/atoms/Typography/Typography";
import { TextField } from "@/components/molecules/TextField/TextField";
import { validateWithZod } from "@/lib/forms/validate-with-zod";

import styles from "./ExclusionForm.module.css";

type Props = {
  pending: boolean;
  onSubmit: (payload: CreateExclusionRequest) => Promise<void>;
};

export const ExclusionForm = (props: Props) => {
  const [companyName, setCompanyName] = useState("");
  const [reason, setReason] = useState("");
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const result = validateWithZod(createExclusionRequestSchema, {
      companyName,
      reason,
    });
    if (!result.ok) {
      setErrors(result.errors);
      return;
    }
    setErrors({});
    await props.onSubmit(result.value);
    setCompanyName("");
    setReason("");
  };

  return (
    <form className={styles.container} onSubmit={handleSubmit} noValidate>
      <Typography variant="h3">除外銘柄を登録</Typography>
      <TextField
        label="会社名"
        value={companyName}
        onChange={setCompanyName}
        required
        error={errors.companyName}
      />
      <TextField
        label="理由"
        value={reason}
        onChange={setReason}
        required
        error={errors.reason}
      />
      <Button label="登録" type="submit" disabled={props.pending} />
    </form>
  );
};
