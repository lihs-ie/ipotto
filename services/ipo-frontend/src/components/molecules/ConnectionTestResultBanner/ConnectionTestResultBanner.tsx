import { type TestSecuritiesAccountConnectionResponse } from "@ipotto/shared";

import styles from "./ConnectionTestResultBanner.module.css";

type Props = {
  result: TestSecuritiesAccountConnectionResponse | null;
};

const formatDateTime = (value: string): string =>
  new Date(value).toLocaleString("ja-JP", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });

export const ConnectionTestResultBanner = (props: Props) => {
  if (props.result === null) return null;
  return (
    <div
      className={styles.container}
      data-success={props.result.success}
      role="status"
    >
      <strong>
        {props.result.success ? "接続成功" : "接続失敗"} (
        {formatDateTime(props.result.testedAt)})
      </strong>
      <p>{props.result.message}</p>
    </div>
  );
};
