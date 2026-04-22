import { type Schedule } from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./StockScheduleCard.module.css";

type Props = {
  schedule: Schedule;
};

export const StockScheduleCard = (props: Props) => (
  <section className={styles.container}>
    <Typography variant="h3">スケジュール</Typography>
    <dl className={styles.grid}>
      <div className={styles.row}>
        <dt>BB開始</dt>
        <dd>{props.schedule.bookBuildingStartDate}</dd>
      </div>
      <div className={styles.row}>
        <dt>BB終了</dt>
        <dd>{props.schedule.bookBuildingEndDate}</dd>
      </div>
      <div className={styles.row}>
        <dt>抽選日</dt>
        <dd>{props.schedule.lotteryDate}</dd>
      </div>
      <div className={styles.row}>
        <dt>上場日</dt>
        <dd>{props.schedule.listingDate}</dd>
      </div>
    </dl>
  </section>
);
