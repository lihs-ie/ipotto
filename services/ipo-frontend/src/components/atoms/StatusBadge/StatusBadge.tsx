import {
  type ApplicationStatus,
  type LotteryResult,
  type StockStatus,
} from "@ipotto/shared";

import styles from "./StatusBadge.module.css";

type StatusValue = StockStatus | ApplicationStatus | LotteryResult;

type Props = {
  status: StatusValue;
  label?: string;
};

type Tone = "neutral" | "success" | "warning" | "danger";

const toneFor = (status: StatusValue): Tone => {
  switch (status) {
    case "Won":
    case "Purchased":
    case "Sold":
      return "success";
    case "Eligible":
    case "Applied":
    case "ResultChecked":
    case "Alternate":
    case "Pending":
      return "warning";
    case "Lost":
    case "Failed":
    case "Declined":
    case "Excluded":
      return "danger";
    case "Fetched":
    default:
      return "neutral";
  }
};

export const StatusBadge = (props: Props) => (
  <span
    className={styles.container}
    data-tone={toneFor(props.status)}
    data-status={props.status}
  >
    {props.label ?? props.status}
  </span>
);
