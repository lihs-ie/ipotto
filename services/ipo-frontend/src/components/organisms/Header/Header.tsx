import { Button } from "@/components/atoms/Button/Button";

import styles from "./Header.module.css";

type Breadcrumb = {
  label: string;
  path: string;
};

type Props = {
  userEmail: string | null;
  onSignOut: () => Promise<void>;
  breadcrumbs: Breadcrumb[];
};

const PAGE_LABELS: Record<string, string> = {
  "/": "ダッシュボード",
  "/stocks": "IPO銘柄",
  "/exclusions": "除外リスト",
  "/notifications/settings": "通知設定",
  "/accounts": "証券口座",
  "/logs": "操作ログ",
};

export const resolveBreadcrumbs = (pathname: string): Breadcrumb[] => {
  const directMatch = PAGE_LABELS[pathname];
  if (directMatch !== undefined) {
    return [{ label: directMatch, path: pathname }];
  }

  const segments = pathname.split("/").filter(Boolean);
  const breadcrumbs: Breadcrumb[] = [];

  let accumulated = "";
  for (const segment of segments) {
    accumulated = `${accumulated}/${segment}`;
    const label = PAGE_LABELS[accumulated];
    if (label !== undefined) {
      breadcrumbs.push({ label, path: accumulated });
    }
  }

  if (breadcrumbs.length === 0) {
    return [{ label: "ダッシュボード", path: "/" }];
  }

  return breadcrumbs;
};

export const Header = (props: Props) => {
  const handleSignOut = (): void => {
    void props.onSignOut();
  };

  return (
    <header className={styles.container}>
      <nav className={styles.breadcrumb} aria-label="パンくずリスト">
        {props.breadcrumbs.map((crumb, index) => {
          const isLast = index === props.breadcrumbs.length - 1;
          return (
            <span key={crumb.path}>
              {index > 0 && (
                <span className={styles.separator} aria-hidden="true">
                  /
                </span>
              )}
              <span className={isLast ? styles.active : undefined}>
                {crumb.label}
              </span>
            </span>
          );
        })}
      </nav>
      <div className={styles.spacer} />
      <div className={styles.actions}>
        {props.userEmail !== null && (
          <span className={styles.email}>{props.userEmail}</span>
        )}
        <Button label="ログアウト" variant="ghost" onClick={handleSignOut} />
      </div>
    </header>
  );
};
