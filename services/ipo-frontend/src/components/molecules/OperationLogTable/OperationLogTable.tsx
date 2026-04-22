import { type OperationLogSummary } from "@ipotto/shared";

import styles from "./OperationLogTable.module.css";

type Props = {
  items: OperationLogSummary[];
};

const formatDateTime = (value: string): string =>
  new Date(value).toLocaleString("ja-JP", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });

export const OperationLogTable = (props: Props) => (
  <table className={styles.container}>
    <thead>
      <tr>
        <th>実行日時</th>
        <th>イベント</th>
        <th>サービス</th>
        <th>ステータス</th>
        <th>メッセージ</th>
      </tr>
    </thead>
    <tbody>
      {props.items.length === 0 ? (
        <tr>
          <td colSpan={5} className={styles.empty}>
            ログはありません。
          </td>
        </tr>
      ) : (
        props.items.map((item) => (
          <tr key={item.identifier}>
            <td>{formatDateTime(item.executedAt)}</td>
            <td>{item.eventType}</td>
            <td>{item.serviceName}</td>
            <td data-status={item.status}>{item.status}</td>
            <td>{item.message}</td>
          </tr>
        ))
      )}
    </tbody>
  </table>
);
