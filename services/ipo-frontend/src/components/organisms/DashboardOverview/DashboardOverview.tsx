import { type DashboardSummaryResponse } from "@ipotto/shared";

import { RecentActivitiesList } from "@/components/molecules/RecentActivitiesList/RecentActivitiesList";
import { StatusCountsCard } from "@/components/molecules/StatusCountsCard/StatusCountsCard";
import { SystemStatusCard } from "@/components/molecules/SystemStatusCard/SystemStatusCard";
import { UpcomingStocksList } from "@/components/molecules/UpcomingStocksList/UpcomingStocksList";

import styles from "./DashboardOverview.module.css";

type Props = {
  summary: DashboardSummaryResponse;
};

export const DashboardOverview = (props: Props) => (
  <div className={styles.container}>
    <StatusCountsCard counts={props.summary.statusCounts} />
    <RecentActivitiesList activities={props.summary.recentActivities} />
    <UpcomingStocksList stocks={props.summary.upcomingStocks} />
    <SystemStatusCard
      nextJobScheduledAt={props.summary.systemStatus.nextJobScheduledAt}
      accounts={props.summary.systemStatus.accounts}
    />
  </div>
);
