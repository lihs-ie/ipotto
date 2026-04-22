"use client";

import { useCallback } from "react";
import { useParams, useRouter } from "next/navigation";

import {
  securitiesAccountIdentifierSchema,
  type UpdateSecuritiesAccountRequest,
} from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";
import { FormErrorBanner } from "@/components/molecules/FormErrorBanner/FormErrorBanner";
import { SecuritiesAccountForm } from "@/components/molecules/SecuritiesAccountForm/SecuritiesAccountForm";
import { useIpoApi } from "@/app/client-providers";
import { useMutation } from "@/lib/api/use-mutation";

export const EditAccountClient = () => {
  const params = useParams<{ accountIdentifier: string }>();
  const router = useRouter();
  const api = useIpoApi();

  const parsed = securitiesAccountIdentifierSchema.safeParse(
    params.accountIdentifier,
  );

  const mutation = useMutation(
    useCallback(
      async (payload: UpdateSecuritiesAccountRequest) => {
        if (!parsed.success) {
          throw new Error("invalid account identifier");
        }
        return api.updateSecuritiesAccount(parsed.data, payload);
      },
      [api, parsed],
    ),
  );

  if (!parsed.success) {
    return (
      <section>
        <Typography variant="h1">口座が見つかりません</Typography>
        <Typography variant="body">識別子の形式が不正です。</Typography>
      </section>
    );
  }

  const handleSubmit = async (
    payload: UpdateSecuritiesAccountRequest,
  ): Promise<void> => {
    const outcome = await mutation.mutate(payload);
    if (outcome.ok) {
      router.push("/accounts");
    }
  };

  const error =
    mutation.state.status === "error" ? mutation.state.error : null;

  return (
    <section>
      <Typography variant="h1">証券口座を編集</Typography>
      <Typography variant="caption">{parsed.data}</Typography>
      <FormErrorBanner error={error} />
      <SecuritiesAccountForm
        mode="update"
        pending={mutation.state.status === "loading"}
        onSubmit={handleSubmit}
      />
    </section>
  );
};
