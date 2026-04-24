import { type Pricing } from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./StockPricingCard.module.css";

type Props = {
  pricing: Pricing;
};

const formatYen = (amount: number): string => `${amount.toLocaleString()}円`;

export const StockPricingCard = (props: Props) => (
  <section className={styles.container}>
    <Typography variant="h3">価格</Typography>
    <dl className={styles.grid}>
      <div className={styles.row}>
        <dt>仮条件(下限)</dt>
        <dd>{formatYen(props.pricing.priceRangeMin)}</dd>
      </div>
      <div className={styles.row}>
        <dt>仮条件(上限)</dt>
        <dd>{formatYen(props.pricing.priceRangeMax)}</dd>
      </div>
      <div className={styles.row}>
        <dt>公開価格</dt>
        <dd>
          {props.pricing.offerPrice !== null
            ? formatYen(props.pricing.offerPrice)
            : "未定"}
        </dd>
      </div>
    </dl>
  </section>
);
