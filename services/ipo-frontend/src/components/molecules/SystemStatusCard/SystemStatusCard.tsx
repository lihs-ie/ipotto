import { type DashboardAccountStatus } from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./SystemStatusCard.module.css";

type Props = {
  nextJobScheduledAt: string;
  accounts: DashboardAccountStatus[];
};

const formatDateTime = (value: string): string =>
  new Date(value).toLocaleString("ja-JP", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });

export const SystemStatusCard = (props: Props) => (
  <section className={styles.container}>
    <Typography variant="h3">システム状態</Typography>
    <dl className={styles.list}>
      <div className={styles.row}>
        <dt className={styles.label}>次回ジョブ</dt>
        <dd className={styles.value}>
          {formatDateTime(props.nextJobScheduledAt)}
        </dd>
      </div>
      {props.accounts.map((account) => (
        <div key={account.securitiesCompany} className={styles.row}>
          <dt className={styles.label}>{account.securitiesCompany}</dt>
          <dd className={styles.value} data-status={account.connectionStatus}>
            {account.connectionStatus}
          </dd>
        </div>
      ))}
    </dl>
  </section>
);
