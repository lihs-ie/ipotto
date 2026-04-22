import { type DashboardRecentActivity } from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./RecentActivitiesList.module.css";

type Props = {
  activities: DashboardRecentActivity[];
};

const formatDateTime = (value: string): string =>
  new Date(value).toLocaleString("ja-JP", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });

export const RecentActivitiesList = (props: Props) => (
  <section className={styles.container}>
    <Typography variant="h3">最近の活動</Typography>
    {props.activities.length === 0 ? (
      <Typography variant="caption">活動履歴はありません。</Typography>
    ) : (
      <ul className={styles.list}>
        {props.activities.map((activity) => (
          <li
            key={`${activity.stock}-${activity.occurredAt}`}
            className={styles.row}
          >
            <span className={styles.timestamp}>
              {formatDateTime(activity.occurredAt)}
            </span>
            <span className={styles.company}>{activity.companyName}</span>
            <span className={styles.event}>{activity.eventType}</span>
            <span className={styles.securities}>
              {activity.securitiesCompany}
            </span>
          </li>
        ))}
      </ul>
    )}
  </section>
);
