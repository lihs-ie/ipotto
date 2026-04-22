import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./Footer.module.css";

type Props = {
  year?: number;
};

export const Footer = (props: Props) => {
  const year = props.year ?? new Date().getFullYear();
  return (
    <footer className={styles.container}>
      <Typography variant="caption">© {year} IPOtto</Typography>
    </footer>
  );
};
