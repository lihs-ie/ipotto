import { type StockStatus } from "@ipotto/shared";

import styles from "./StockStatusFilter.module.css";

const options: StockStatus[] = [
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
  value: StockStatus | null;
  onChange: (next: StockStatus | null) => void;
};

export const StockStatusFilter = (props: Props) => (
  <div className={styles.container} role="group" aria-label="ステータスフィルタ">
    <button
      type="button"
      className={styles.chip}
      data-active={props.value === null}
      onClick={() => props.onChange(null)}
    >
      すべて
    </button>
    {options.map((status) => (
      <button
        key={status}
        type="button"
        className={styles.chip}
        data-active={props.value === status}
        onClick={() => props.onChange(status)}
      >
        {status}
      </button>
    ))}
  </div>
);
