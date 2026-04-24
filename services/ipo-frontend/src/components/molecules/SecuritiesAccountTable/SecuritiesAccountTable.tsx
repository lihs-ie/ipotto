import {
  type SecuritiesAccountIdentifier,
  type SecuritiesAccountSummary,
} from "@ipotto/shared";

import { Button } from "@/components/atoms/Button/Button";

import styles from "./SecuritiesAccountTable.module.css";

type Props = {
  items: SecuritiesAccountSummary[];
  onEdit: (identifier: SecuritiesAccountIdentifier) => void;
  onTest: (identifier: SecuritiesAccountIdentifier) => void;
  onDelete: (identifier: SecuritiesAccountIdentifier) => void;
};

const formatDateTime = (value: string | null): string => {
  if (value === null) return "—";
  return new Date(value).toLocaleString("ja-JP", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
};

export const SecuritiesAccountTable = (props: Props) => (
  <table className={styles.container}>
    <thead>
      <tr>
        <th>証券会社</th>
        <th>ログインID</th>
        <th>メール</th>
        <th>IMAP</th>
        <th>有効</th>
        <th>最終テスト</th>
        <th>アクション</th>
      </tr>
    </thead>
    <tbody>
      {props.items.length === 0 ? (
        <tr>
          <td colSpan={7} className={styles.empty}>
            口座がまだ登録されていません。
          </td>
        </tr>
      ) : (
        props.items.map((account) => (
          <tr key={account.identifier}>
            <td>{account.securitiesCompany}</td>
            <td>{account.loginIdMasked}</td>
            <td>{account.mailAddressMasked}</td>
            <td>
              {account.imapHost}:{account.imapPort}
            </td>
            <td>{account.active ? "Active" : "Inactive"}</td>
            <td>
              {formatDateTime(account.lastTestedAt)}
              {account.lastTestedStatus !== null && (
                <span className={styles.tone}> ({account.lastTestedStatus})</span>
              )}
            </td>
            <td className={styles.actions}>
              <Button
                label="編集"
                variant="secondary"
                onClick={() => props.onEdit(account.identifier)}
              />
              <Button
                label="テスト"
                variant="secondary"
                onClick={() => props.onTest(account.identifier)}
              />
              <Button
                label="削除"
                variant="secondary"
                onClick={() => props.onDelete(account.identifier)}
              />
            </td>
          </tr>
        ))
      )}
    </tbody>
  </table>
);
