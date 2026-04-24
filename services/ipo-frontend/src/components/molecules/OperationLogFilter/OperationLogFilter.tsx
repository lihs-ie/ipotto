import { type OperationEventType } from "@ipotto/shared";

import styles from "./OperationLogFilter.module.css";

export type OperationLogFilterValue = {
  eventType: OperationEventType | null;
};

type Props = {
  value: OperationLogFilterValue;
  onChange: (next: OperationLogFilterValue) => void;
};

const eventTypeOptions: OperationEventType[] = [
  "fetch_stocks",
  "apply_lottery",
  "check_lottery_result",
  "notification_dispatch",
  "connection_test",
  "other",
];

export const OperationLogFilter = (props: Props) => {
  const handleEventChange = (next: string): void => {
    props.onChange({
      eventType:
        next === "" ? null : (next as OperationEventType),
    });
  };

  return (
    <div className={styles.container}>
      <label className={styles.field}>
        <span className={styles.label}>イベント種別</span>
        <select
          className={styles.select}
          value={props.value.eventType ?? ""}
          onChange={(event) => handleEventChange(event.target.value)}
        >
          <option value="">すべて</option>
          {eventTypeOptions.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      </label>
    </div>
  );
};
