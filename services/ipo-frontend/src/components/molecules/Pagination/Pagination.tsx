import { Button } from "@/components/atoms/Button/Button";

import styles from "./Pagination.module.css";

type Props = {
  hasMore: boolean;
  onNext: () => void;
  loading?: boolean;
};

export const Pagination = (props: Props) => (
  <div className={styles.container}>
    <Button
      label="次へ"
      variant="secondary"
      disabled={!props.hasMore || props.loading === true}
      onClick={props.onNext}
    />
  </div>
);
