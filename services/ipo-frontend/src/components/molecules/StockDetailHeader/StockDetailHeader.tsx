import {
  type StockIdentifier,
  type StockStatus,
} from "@ipotto/shared";

import { StatusBadge } from "@/components/atoms/StatusBadge/StatusBadge";
import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./StockDetailHeader.module.css";

type Props = {
  identifier: StockIdentifier;
  companyName: string;
  status: StockStatus;
};

export const StockDetailHeader = (props: Props) => (
  <header className={styles.container}>
    <div>
      <Typography variant="h1">{props.companyName}</Typography>
      <Typography variant="caption">{props.identifier}</Typography>
    </div>
    <StatusBadge status={props.status} />
  </header>
);
