import { type DashboardUpcomingStock } from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./UpcomingStocksList.module.css";

type Props = {
  stocks: DashboardUpcomingStock[];
};

export const UpcomingStocksList = (props: Props) => (
  <section className={styles.container}>
    <Typography variant="h3">今後の新規上場</Typography>
    {props.stocks.length === 0 ? (
      <Typography variant="caption">対象の銘柄はありません。</Typography>
    ) : (
      <ul className={styles.list}>
        {props.stocks.map((stock) => (
          <li key={stock.stock} className={styles.row}>
            <span className={styles.company}>{stock.companyName}</span>
            <span className={styles.dates}>
              BB: {stock.bookBuildingStartDate} 〜 {stock.bookBuildingEndDate} /
              抽選: {stock.lotteryDate}
            </span>
          </li>
        ))}
      </ul>
    )}
  </section>
);
