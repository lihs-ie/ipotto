import Link from "next/link";

import styles from "./Sidebar.module.css";

type NavigationItem = {
  label: string;
  path: string;
  icon: React.ReactNode;
};

const DashboardIcon = () => (
  <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="1.6" fill="none" strokeLinecap="round" strokeLinejoin="round">
    <rect x="3" y="3" width="7" height="9" />
    <rect x="14" y="3" width="7" height="5" />
    <rect x="14" y="12" width="7" height="9" />
    <rect x="3" y="16" width="7" height="5" />
  </svg>
);

const StocksIcon = () => (
  <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="1.6" fill="none" strokeLinecap="round" strokeLinejoin="round">
    <path d="M3 17L9 11L13 15L21 7" />
    <path d="M15 7H21V13" />
  </svg>
);

const BanIcon = () => (
  <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="1.6" fill="none" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="12" cy="12" r="9" />
    <path d="M5.6 5.6L18.4 18.4" />
  </svg>
);

const BellIcon = () => (
  <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="1.6" fill="none" strokeLinecap="round" strokeLinejoin="round">
    <path d="M6 8A6 6 0 0 1 18 8C18 15 21 16 21 16H3C3 16 6 15 6 8Z" />
    <path d="M10 20A2 2 0 0 0 14 20" />
  </svg>
);

const WalletIcon = () => (
  <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="1.6" fill="none" strokeLinecap="round" strokeLinejoin="round">
    <rect x="3" y="6" width="18" height="14" rx="2" />
    <path d="M3 10H21" />
    <circle cx="16" cy="15" r="1.2" fill="currentColor" />
  </svg>
);

const LogIcon = () => (
  <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="1.6" fill="none" strokeLinecap="round" strokeLinejoin="round">
    <path d="M4 4H20V20H4Z" />
    <path d="M8 9H16M8 13H16M8 17H12" />
  </svg>
);

const mainItems: NavigationItem[] = [
  { label: "ダッシュボード", path: "/", icon: <DashboardIcon /> },
  { label: "IPO銘柄", path: "/stocks", icon: <StocksIcon /> },
  { label: "除外リスト", path: "/exclusions", icon: <BanIcon /> },
];

const settingsItems: NavigationItem[] = [
  { label: "通知設定", path: "/notifications/settings", icon: <BellIcon /> },
  { label: "証券口座", path: "/accounts", icon: <WalletIcon /> },
  { label: "操作ログ", path: "/logs", icon: <LogIcon /> },
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
    <div className={styles.brand}>
      <div className={styles.logo}>
        <svg viewBox="0 0 24 24" width="16" height="16">
          <path d="M4 18L9 9L13 13L20 4" stroke="#0A1120" strokeWidth="2.2" fill="none" strokeLinecap="round" strokeLinejoin="round" />
          <circle cx="20" cy="4" r="2" fill="#0A1120" />
        </svg>
      </div>
      <div>
        <div className={styles.name}>IPOtto</div>
        <div className={styles.subtitle}>IPO Lottery</div>
      </div>
    </div>

    <ul className={styles.list}>
      {mainItems.map((item) => (
        <li key={item.path}>
          <Link
            href={item.path}
            className={styles.link}
            data-active={isActive(props.currentPath, item.path)}
          >
            {item.icon}
            {item.label}
          </Link>
        </li>
      ))}
    </ul>

    <div className={styles.section}>設定</div>

    <ul className={styles.list}>
      {settingsItems.map((item) => (
        <li key={item.path}>
          <Link
            href={item.path}
            className={styles.link}
            data-active={isActive(props.currentPath, item.path)}
          >
            {item.icon}
            {item.label}
          </Link>
        </li>
      ))}
    </ul>

    <div className={styles.user}>
      <div className={styles.avatar} />
      <div className={styles.profile}>
        <div className={styles.username}>ユーザー</div>
        <div className={styles.email}>user@example.com</div>
      </div>
    </div>
  </nav>
);
