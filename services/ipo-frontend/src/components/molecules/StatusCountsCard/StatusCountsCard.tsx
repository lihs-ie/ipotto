import { type StockStatus } from "@ipotto/shared";

import { StatusBadge } from "@/components/atoms/StatusBadge/StatusBadge";
import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./StatusCountsCard.module.css";

const displayOrder: StockStatus[] = [
  "Fetched",
  "Eligible",
  "Applied",
  "Won",
  "Lost",
  "Alternate",
  "Purchased",
  "Declined",
  "Sold",
  "Excluded",
  "Failed",
];

type Props = {
  counts: Partial<Record<StockStatus, number>>;
};

export const StatusCountsCard = (props: Props) => (
  <section className={styles.container}>
    <Typography variant="h3">ステータス別件数</Typography>
    <dl className={styles.grid}>
      {displayOrder.map((status) => {
        const count = props.counts[status] ?? 0;
        return (
          <div key={status} className={styles.cell}>
            <dt>
              <StatusBadge status={status} />
            </dt>
            <dd className={styles.count}>{count}</dd>
          </div>
        );
      })}
    </dl>
  </section>
);
