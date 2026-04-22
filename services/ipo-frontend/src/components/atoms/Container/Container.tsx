import { type ReactNode } from "react";

import styles from "./Container.module.css";

type Props = {
  children: ReactNode;
};

export const Container = (props: Props) => (
  <div className={styles.container}>{props.children}</div>
);
