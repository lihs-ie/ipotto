import { type ApiError } from "@/lib/api/error";

import styles from "./FormErrorBanner.module.css";

type Props = {
  error: ApiError | null;
};

const describe = (error: ApiError): string => {
  switch (error.kind) {
    case "network":
      return `ネットワークエラー: ${error.message}`;
    case "http":
      return `${error.body.error.code}: ${error.body.error.message}`;
    case "validation":
      return "サーバーからの応答が不正です。";
    case "unexpected":
      return `予期しないエラー (${error.status ?? "unknown"}): ${error.message}`;
  }
};

export const FormErrorBanner = (props: Props) => {
  if (props.error === null) return null;
  const detailLines: string[] =
    props.error.kind === "http" && props.error.body.error.details
      ? props.error.body.error.details.map(
          (detail) => `${detail.field}: ${detail.message}`,
        )
      : [];

  return (
    <div className={styles.container} role="alert">
      <p className={styles.summary}>{describe(props.error)}</p>
      {detailLines.length > 0 && (
        <ul className={styles.details}>
          {detailLines.map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
      )}
    </div>
  );
};
