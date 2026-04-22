import { type StockApplication } from "@ipotto/shared";

import { StatusBadge } from "@/components/atoms/StatusBadge/StatusBadge";
import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./StockApplicationsTable.module.css";

type Props = {
  applications: StockApplication[];
};

const formatDateTime = (value: string): string =>
  new Date(value).toLocaleString("ja-JP", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });

export const StockApplicationsTable = (props: Props) => (
  <section className={styles.container}>
    <Typography variant="h3">申込履歴</Typography>
    {props.applications.length === 0 ? (
      <Typography variant="caption">申込はまだありません。</Typography>
    ) : (
      <table className={styles.table}>
        <thead>
          <tr>
            <th>証券会社</th>
            <th>申込株数</th>
            <th>申込価格</th>
            <th>申込日時</th>
            <th>結果</th>
            <th>ステータス</th>
          </tr>
        </thead>
        <tbody>
          {props.applications.map((application) => (
            <tr key={application.identifier}>
              <td>{application.securitiesCompany}</td>
              <td>{application.appliedShares.toLocaleString()}株</td>
              <td>{application.appliedPrice.toLocaleString()}円</td>
              <td>{formatDateTime(application.appliedAt)}</td>
              <td>
                {application.lotteryOutcome !== null ? (
                  <StatusBadge status={application.lotteryOutcome} />
                ) : (
                  "—"
                )}
              </td>
              <td>
                <StatusBadge status={application.status} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    )}
  </section>
);
