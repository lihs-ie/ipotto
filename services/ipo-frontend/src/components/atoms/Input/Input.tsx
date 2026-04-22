import { type ChangeEvent } from "react";

import styles from "./Input.module.css";

type Props = {
  value: string;
  onChange: (value: string) => void;
  type?: "text" | "email" | "password" | "number";
  placeholder?: string;
  name?: string;
  disabled?: boolean;
  required?: boolean;
};

export const Input = (props: Props) => {
  const handleChange = (event: ChangeEvent<HTMLInputElement>): void => {
    props.onChange(event.target.value);
  };

  return (
    <input
      className={styles.container}
      type={props.type ?? "text"}
      name={props.name}
      value={props.value}
      placeholder={props.placeholder}
      disabled={props.disabled}
      required={props.required}
      onChange={handleChange}
    />
  );
};
