import Link from "next/link";

import styles from "./Sidebar.module.css";

type NavigationItem = {
  label: string;
  path: string;
};

const items: NavigationItem[] = [
  { label: "ダッシュボード", path: "/" },
  { label: "銘柄一覧", path: "/stocks" },
  { label: "操作ログ", path: "/logs" },
  { label: "除外リスト", path: "/exclusions" },
  { label: "通知設定", path: "/notifications/settings" },
  { label: "証券口座", path: "/accounts" },
];

type Props = {
  currentPath: string;
};

const isActive = (currentPath: string, itemPath: string): boolean => {
  if (itemPath === "/") return currentPath === "/";
  return currentPath === itemPath || currentPath.startsWith(`${itemPath}/`);
};

export const Sidebar = (props: Props) => (
  <nav className={styles.container} aria-label="メインナビゲーション">
    <ul className={styles.list}>
      {items.map((item) => (
        <li key={item.path}>
          <Link
            href={item.path}
            className={styles.link}
            data-active={isActive(props.currentPath, item.path)}
          >
            {item.label}
          </Link>
        </li>
      ))}
    </ul>
  </nav>
);
