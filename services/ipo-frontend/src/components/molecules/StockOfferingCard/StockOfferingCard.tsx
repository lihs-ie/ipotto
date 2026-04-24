import { type Offering } from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./StockOfferingCard.module.css";

type Props = {
  offering: Offering;
};

export const StockOfferingCard = (props: Props) => (
  <section className={styles.container}>
    <Typography variant="h3">募集</Typography>
    <dl className={styles.grid}>
      <div className={styles.row}>
        <dt>主幹事</dt>
        <dd>{props.offering.leadUnderwriter}</dd>
      </div>
      <div className={styles.row}>
        <dt>公募株数</dt>
        <dd>{props.offering.numberOfOfferedShares.toLocaleString()}株</dd>
      </div>
    </dl>
  </section>
);
