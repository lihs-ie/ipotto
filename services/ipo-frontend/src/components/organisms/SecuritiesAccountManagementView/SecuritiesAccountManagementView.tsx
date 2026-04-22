"use client";

import { useCallback, useState } from "react";
import Link from "next/link";

import {
  type SecuritiesAccountIdentifier,
  type TestSecuritiesAccountConnectionResponse,
} from "@ipotto/shared";

import { Button } from "@/components/atoms/Button/Button";
import { Typography } from "@/components/atoms/Typography/Typography";
import { ConfirmDialog } from "@/components/molecules/ConfirmDialog/ConfirmDialog";
import { ConnectionTestResultBanner } from "@/components/molecules/ConnectionTestResultBanner/ConnectionTestResultBanner";
import { FormErrorBanner } from "@/components/molecules/FormErrorBanner/FormErrorBanner";
import { SecuritiesAccountTable } from "@/components/molecules/SecuritiesAccountTable/SecuritiesAccountTable";
import { useIpoApi } from "@/app/client-providers";
import { useAsyncResult } from "@/lib/api/use-async-result";
import { useMutation } from "@/lib/api/use-mutation";

import styles from "./SecuritiesAccountManagementView.module.css";

export const SecuritiesAccountManagementView = () => {
  const api = useIpoApi();

  const listFetcher = useCallback(
    () => api.listSecuritiesAccounts(),
    [api],
  );
  const list = useAsyncResult(listFetcher, [listFetcher]);

  const deleteMutation = useMutation(
    useCallback(
      (identifier: SecuritiesAccountIdentifier) =>
        api.deleteSecuritiesAccount(identifier),
      [api],
    ),
  );

  const testMutation = useMutation(
    useCallback(
      (identifier: SecuritiesAccountIdentifier) =>
        api.testSecuritiesAccountConnection(identifier),
      [api],
    ),
  );

  const [pendingDelete, setPendingDelete] = useState<
    SecuritiesAccountIdentifier | null
  >(null);
  const [testResult, setTestResult] =
    useState<TestSecuritiesAccountConnectionResponse | null>(null);

  const handleTest = useCallback(
    async (identifier: SecuritiesAccountIdentifier): Promise<void> => {
      setTestResult(null);
      const outcome = await testMutation.mutate(identifier);
      if (outcome.ok) {
        setTestResult(outcome.value);
      }
    },
    [testMutation],
  );

  const handleConfirmDelete = useCallback(async (): Promise<void> => {
    if (pendingDelete === null) return;
    const outcome = await deleteMutation.mutate(pendingDelete);
    setPendingDelete(null);
    if (outcome.ok) list.reload();
  }, [deleteMutation, list, pendingDelete]);

  const mutationError =
    deleteMutation.state.status === "error"
      ? deleteMutation.state.error
      : testMutation.state.status === "error"
      ? testMutation.state.error
      : null;

  return (
    <section className={styles.container}>
      <div className={styles.header}>
        <Typography variant="h1">証券口座管理</Typography>
        <Link href="/accounts/new" className={styles.registerLink}>
          <Button label="新規登録" />
        </Link>
      </div>
      <FormErrorBanner error={mutationError} />
      <ConnectionTestResultBanner result={testResult} />
      {list.state.status === "loading" && (
        <Typography variant="body">読み込んでいます…</Typography>
      )}
      {list.state.status === "error" && (
        <Typography variant="body">
          読み込みに失敗しました: {list.state.error.kind}
        </Typography>
      )}
      {list.state.status === "ok" && (
        <SecuritiesAccountTable
          items={list.state.value.items}
          onEdit={(identifier) => {
            window.location.assign(
              `/accounts/${encodeURIComponent(identifier)}/edit`,
            );
          }}
          onTest={(identifier) => {
            void handleTest(identifier);
          }}
          onDelete={setPendingDelete}
        />
      )}
      <ConfirmDialog
        open={pendingDelete !== null}
        title="口座を削除しますか?"
        message="選択した証券口座を削除し、保存されている認証情報も消去します。"
        confirmLabel="削除"
        onConfirm={handleConfirmDelete}
        onCancel={() => setPendingDelete(null)}
      />
    </section>
  );
};
