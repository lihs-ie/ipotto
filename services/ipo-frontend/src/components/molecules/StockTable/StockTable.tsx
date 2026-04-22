import {
  type IpoStockSummary,
  type StockIdentifier,
} from "@ipotto/shared";

import { StatusBadge } from "@/components/atoms/StatusBadge/StatusBadge";

import styles from "./StockTable.module.css";

type Props = {
  stocks: IpoStockSummary[];
  onRowClick: (identifier: StockIdentifier) => void;
};

const formatPriceRange = (min: number, max: number): string =>
  `${min.toLocaleString()} 〜 ${max.toLocaleString()}円`;

export const StockTable = (props: Props) => (
  <table className={styles.container}>
    <thead>
      <tr>
        <th>会社名</th>
        <th>市場</th>
        <th>BB期間</th>
        <th>抽選日</th>
        <th>仮条件</th>
        <th>公開価格</th>
        <th>ステータス</th>
      </tr>
    </thead>
    <tbody>
      {props.stocks.length === 0 ? (
        <tr>
          <td colSpan={7} className={styles.empty}>
            対象の銘柄はありません。
          </td>
        </tr>
      ) : (
        props.stocks.map((stock) => (
          <tr
            key={stock.identifier}
            className={styles.row}
            onClick={() => props.onRowClick(stock.identifier)}
            role="button"
            tabIndex={0}
          >
            <td>{stock.companyName}</td>
            <td>{stock.market}</td>
            <td>
              {stock.bookBuildingStartDate} 〜 {stock.bookBuildingEndDate}
            </td>
            <td>{stock.lotteryDate}</td>
            <td>
              {formatPriceRange(stock.priceRangeMin, stock.priceRangeMax)}
            </td>
            <td>
              {stock.offerPrice !== null
                ? `${stock.offerPrice.toLocaleString()}円`
                : "—"}
            </td>
            <td>
              <StatusBadge status={stock.status} />
            </td>
          </tr>
        ))
      )}
    </tbody>
  </table>
);
