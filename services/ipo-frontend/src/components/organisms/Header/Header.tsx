import { Button } from "@/components/atoms/Button/Button";
import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./Header.module.css";

type Props = {
  userEmail: string | null;
  onSignOut: () => Promise<void>;
};

export const Header = (props: Props) => {
  const handleSignOut = (): void => {
    void props.onSignOut();
  };

  return (
    <header className={styles.container}>
      <div className={styles.brand}>
        <Typography variant="h2">IPOtto</Typography>
      </div>
      <div className={styles.actions}>
        {props.userEmail !== null && (
          <Typography variant="caption">{props.userEmail}</Typography>
        )}
        <Button label="ログアウト" variant="secondary" onClick={handleSignOut} />
      </div>
    </header>
  );
};
