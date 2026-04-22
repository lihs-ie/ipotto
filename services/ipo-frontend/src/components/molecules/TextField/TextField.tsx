import { Input } from "@/components/atoms/Input/Input";
import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./TextField.module.css";

type Props = {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: "text" | "email" | "password" | "number";
  placeholder?: string;
  name?: string;
  required?: boolean;
  error?: string;
  disabled?: boolean;
};

export const TextField = (props: Props) => (
  <label className={styles.container}>
    <span className={styles.label}>
      {props.label}
      {props.required && <span className={styles.required}>*</span>}
    </span>
    <Input
      value={props.value}
      onChange={props.onChange}
      type={props.type ?? "text"}
      placeholder={props.placeholder}
      name={props.name}
      required={props.required}
      disabled={props.disabled}
    />
    {props.error !== undefined && props.error !== "" && (
      <Typography variant="caption">
        <span className={styles.error}>{props.error}</span>
      </Typography>
    )}
  </label>
);
