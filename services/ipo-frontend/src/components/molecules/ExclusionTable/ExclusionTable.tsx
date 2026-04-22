import {
  type ExclusionIdentifier,
  type ExclusionSummary,
} from "@ipotto/shared";

import { Button } from "@/components/atoms/Button/Button";

import styles from "./ExclusionTable.module.css";

type Props = {
  items: ExclusionSummary[];
  onDelete: (identifier: ExclusionIdentifier) => void;
};

const formatDateTime = (value: string): string =>
  new Date(value).toLocaleString("ja-JP", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });

export const ExclusionTable = (props: Props) => (
  <table className={styles.container}>
    <thead>
      <tr>
        <th>会社名</th>
        <th>理由</th>
        <th>登録日時</th>
        <th>アクション</th>
      </tr>
    </thead>
    <tbody>
      {props.items.length === 0 ? (
        <tr>
          <td colSpan={4} className={styles.empty}>
            除外対象はまだ登録されていません。
          </td>
        </tr>
      ) : (
        props.items.map((exclusion) => (
          <tr key={exclusion.identifier}>
            <td>{exclusion.companyName}</td>
            <td>{exclusion.reason}</td>
            <td>{formatDateTime(exclusion.registeredAt)}</td>
            <td>
              <Button
                label="削除"
                variant="secondary"
                onClick={() => props.onDelete(exclusion.identifier)}
              />
            </td>
          </tr>
        ))
      )}
    </tbody>
  </table>
);
