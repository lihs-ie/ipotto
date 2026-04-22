"use client";

import { useCallback, useState } from "react";

import {
  type CreateExclusionRequest,
  type ExclusionIdentifier,
} from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";
import { ConfirmDialog } from "@/components/molecules/ConfirmDialog/ConfirmDialog";
import { ExclusionForm } from "@/components/molecules/ExclusionForm/ExclusionForm";
import { ExclusionTable } from "@/components/molecules/ExclusionTable/ExclusionTable";
import { FormErrorBanner } from "@/components/molecules/FormErrorBanner/FormErrorBanner";
import { useIpoApi } from "@/app/client-providers";
import { useAsyncResult } from "@/lib/api/use-async-result";
import { useMutation } from "@/lib/api/use-mutation";

import styles from "./ExclusionManagementView.module.css";

export const ExclusionManagementView = () => {
  const api = useIpoApi();

  const listFetcher = useCallback(() => api.listExclusions(), [api]);
  const list = useAsyncResult(listFetcher, [listFetcher]);

  const createMutation = useMutation(
    useCallback(
      (payload: CreateExclusionRequest) => api.createExclusion(payload),
      [api],
    ),
  );
  const deleteMutation = useMutation(
    useCallback(
      (identifier: ExclusionIdentifier) => api.deleteExclusion(identifier),
      [api],
    ),
  );

  const [pendingDelete, setPendingDelete] = useState<ExclusionIdentifier | null>(
    null,
  );

  const handleCreate = useCallback(
    async (payload: CreateExclusionRequest): Promise<void> => {
      const outcome = await createMutation.mutate(payload);
      if (outcome.ok) list.reload();
    },
    [createMutation, list],
  );

  const handleConfirmDelete = useCallback(async (): Promise<void> => {
    if (pendingDelete === null) return;
    const outcome = await deleteMutation.mutate(pendingDelete);
    setPendingDelete(null);
    if (outcome.ok) list.reload();
  }, [deleteMutation, list, pendingDelete]);

  const mutationError =
    createMutation.state.status === "error"
      ? createMutation.state.error
      : deleteMutation.state.status === "error"
      ? deleteMutation.state.error
      : null;

  return (
    <section className={styles.container}>
      <Typography variant="h1">除外リスト管理</Typography>
      <FormErrorBanner error={mutationError} />
      <ExclusionForm
        pending={createMutation.state.status === "loading"}
        onSubmit={handleCreate}
      />
      {list.state.status === "loading" && (
        <Typography variant="body">読み込んでいます…</Typography>
      )}
      {list.state.status === "error" && (
        <Typography variant="body">
          読み込みに失敗しました: {list.state.error.kind}
        </Typography>
      )}
      {list.state.status === "ok" && (
        <ExclusionTable
          items={list.state.value.items}
          onDelete={setPendingDelete}
        />
      )}
      <ConfirmDialog
        open={pendingDelete !== null}
        title="除外を解除しますか?"
        message="選択した除外対象をリストから削除します。"
        confirmLabel="削除"
        onConfirm={handleConfirmDelete}
        onCancel={() => setPendingDelete(null)}
      />
    </section>
  );
};
