import { fireEvent, render, screen } from "@testing-library/react";
import {
  securitiesAccountSummarySchema,
  type SecuritiesAccountSummary,
} from "@ipotto/shared";
import { describe, expect, it, vi } from "vitest";

import { SecuritiesAccountTable } from "./SecuritiesAccountTable";

const buildAccount = (): SecuritiesAccountSummary =>
  securitiesAccountSummarySchema.parse({
    identifier: "acct_abc",
    securitiesCompany: "Rakuten",
    loginIdMasked: "r***n",
    mailAddressMasked: "u***@example.com",
    imapHost: "imap.example.com",
    imapPort: 993,
    active: true,
    registeredAt: "2026-03-25T10:00:00Z",
    lastTestedAt: null,
    lastTestedStatus: null,
  });

describe("SecuritiesAccountTable", () => {
  it("renders masked credentials", () => {
    render(
      <SecuritiesAccountTable
        items={[buildAccount()]}
        onEdit={() => undefined}
        onTest={() => undefined}
        onDelete={() => undefined}
      />,
    );
    expect(screen.getByText("r***n")).toBeDefined();
    expect(screen.getByText("u***@example.com")).toBeDefined();
  });

  it("invokes edit/test/delete callbacks", () => {
    const onEdit = vi.fn();
    const onTest = vi.fn();
    const onDelete = vi.fn();
    render(
      <SecuritiesAccountTable
        items={[buildAccount()]}
        onEdit={onEdit}
        onTest={onTest}
        onDelete={onDelete}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "編集" }));
    fireEvent.click(screen.getByRole("button", { name: "テスト" }));
    fireEvent.click(screen.getByRole("button", { name: "削除" }));
    expect(onEdit).toHaveBeenCalledWith("acct_abc");
    expect(onTest).toHaveBeenCalledWith("acct_abc");
    expect(onDelete).toHaveBeenCalledWith("acct_abc");
  });
});
