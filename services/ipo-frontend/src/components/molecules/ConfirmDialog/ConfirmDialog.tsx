import { useEffect, useRef } from "react";

import { Button } from "@/components/atoms/Button/Button";
import { Typography } from "@/components/atoms/Typography/Typography";

import styles from "./ConfirmDialog.module.css";

type Props = {
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
};

export const ConfirmDialog = (props: Props) => {
  const dialogRef = useRef<HTMLDialogElement | null>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    if (props.open && !dialog.open) {
      dialog.showModal();
    } else if (!props.open && dialog.open) {
      dialog.close();
    }
  }, [props.open]);

  return (
    <dialog className={styles.container} ref={dialogRef} onClose={props.onCancel}>
      <Typography variant="h3">{props.title}</Typography>
      <Typography variant="body">{props.message}</Typography>
      <div className={styles.actions}>
        <Button
          label={props.cancelLabel ?? "キャンセル"}
          variant="secondary"
          onClick={props.onCancel}
        />
        <Button
          label={props.confirmLabel ?? "削除"}
          onClick={props.onConfirm}
        />
      </div>
    </dialog>
  );
};
