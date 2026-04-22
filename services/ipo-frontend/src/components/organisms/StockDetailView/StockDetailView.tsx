import { type GetIpoStockResponse } from "@ipotto/shared";

import { StockApplicationsTable } from "@/components/molecules/StockApplicationsTable/StockApplicationsTable";
import { StockDetailHeader } from "@/components/molecules/StockDetailHeader/StockDetailHeader";
import { StockOfferingCard } from "@/components/molecules/StockOfferingCard/StockOfferingCard";
import { StockPricingCard } from "@/components/molecules/StockPricingCard/StockPricingCard";
import { StockScheduleCard } from "@/components/molecules/StockScheduleCard/StockScheduleCard";

import styles from "./StockDetailView.module.css";

type Props = {
  stock: GetIpoStockResponse;
};

export const StockDetailView = (props: Props) => (
  <article className={styles.container}>
    <StockDetailHeader
      identifier={props.stock.identifier}
      companyName={props.stock.companyProfile.companyName}
      status={props.stock.status}
    />
    <div className={styles.grid}>
      <StockScheduleCard schedule={props.stock.schedule} />
      <StockPricingCard pricing={props.stock.pricing} />
      <StockOfferingCard offering={props.stock.offering} />
    </div>
    <StockApplicationsTable applications={props.stock.applications} />
  </article>
);
