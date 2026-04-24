import { type ChangeEvent } from "react";

import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./SelectField.module.css";

type Option = {
  label: string;
  value: string;
};

type Props = {
  label: string;
  value: string;
  options: Option[];
  onChange: (value: string) => void;
  required?: boolean;
  error?: string;
  disabled?: boolean;
};

export const SelectField = (props: Props) => {
  const handleChange = (event: ChangeEvent<HTMLSelectElement>): void => {
    props.onChange(event.target.value);
  };

  return (
    <label className={styles.container}>
      <span className={styles.label}>
        {props.label}
        {props.required && <span className={styles.required}>*</span>}
      </span>
      <select
        className={styles.select}
        value={props.value}
        onChange={handleChange}
        disabled={props.disabled}
        required={props.required}
      >
        {props.options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      {props.error !== undefined && props.error !== "" && (
        <Typography variant="caption">
          <span className={styles.error}>{props.error}</span>
        </Typography>
      )}
    </label>
  );
};
