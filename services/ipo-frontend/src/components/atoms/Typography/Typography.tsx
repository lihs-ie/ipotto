import { type ReactNode } from "react";

import styles from "./Typography.module.css";

type Props = {
  variant: "h1" | "h2" | "h3" | "body" | "caption";
  children: ReactNode;
};

export const Typography = (props: Props) => {
  if (props.variant === "h1") {
    return <h1 className={styles.container} data-variant="h1">{props.children}</h1>;
  }
  if (props.variant === "h2") {
    return <h2 className={styles.container} data-variant="h2">{props.children}</h2>;
  }
  if (props.variant === "h3") {
    return <h3 className={styles.container} data-variant="h3">{props.children}</h3>;
  }
  if (props.variant === "caption") {
    return (
      <span className={styles.container} data-variant="caption">
        {props.children}
      </span>
    );
  }
  return <p className={styles.container} data-variant="body">{props.children}</p>;
};
