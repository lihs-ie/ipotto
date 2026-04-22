import styles from "./Button.module.css";

type Props = {
  label: string;
  onClick?: () => void;
  variant?: "primary" | "secondary";
  type?: "button" | "submit" | "reset";
  disabled?: boolean;
};

export const Button = (props: Props) => (
  <button
    className={styles.container}
    data-variant={props.variant ?? "primary"}
    type={props.type ?? "button"}
    onClick={props.onClick}
    disabled={props.disabled}
  >
    {props.label}
  </button>
);
