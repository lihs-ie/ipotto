"use client";

import { useCallback } from "react";
import { useRouter } from "next/navigation";

import { type CreateSecuritiesAccountRequest } from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";
import { FormErrorBanner } from "@/components/molecules/FormErrorBanner/FormErrorBanner";
import { SecuritiesAccountForm } from "@/components/molecules/SecuritiesAccountForm/SecuritiesAccountForm";
import { useIpoApi } from "@/app/client-providers";
import { useMutation } from "@/lib/api/use-mutation";

export const NewAccountClient = () => {
  const api = useIpoApi();
  const router = useRouter();

  const mutation = useMutation(
    useCallback(
      (payload: CreateSecuritiesAccountRequest) =>
        api.createSecuritiesAccount(payload),
      [api],
    ),
  );

  const handleSubmit = useCallback(
    async (payload: CreateSecuritiesAccountRequest): Promise<void> => {
      const outcome = await mutation.mutate(payload);
      if (outcome.ok) {
        router.push("/accounts");
      }
    },
    [mutation, router],
  );

  const error =
    mutation.state.status === "error" ? mutation.state.error : null;

  return (
    <section>
      <Typography variant="h1">証券口座を登録</Typography>
      <FormErrorBanner error={error} />
      <SecuritiesAccountForm
        mode="create"
        pending={mutation.state.status === "loading"}
        onSubmit={handleSubmit}
      />
    </section>
  );
};
