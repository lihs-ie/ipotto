import { Timestamp, Firestore } from "@google-cloud/firestore";

import type { IpoStockRepository } from "../../application/use-cases/apply-for-lottery-use-case.js";
import type { TargetIpoStock } from "../../domain/ipo-stock.js";

interface IpoStockDocument {
  readonly identifier: string;
  readonly companyProfile: {
    readonly companyName: string;
  };
  readonly schedule: {
    readonly bookBuildingPeriod: {
      readonly startDate: Timestamp | string;
      readonly endDate: Timestamp | string;
    };
  };
  readonly pricing: {
    readonly offerPrice: number | null;
    readonly priceRange: {
      readonly minimumPrice: number;
    };
  };
  readonly offering: {
    readonly numberOfOfferedShares: number;
  };
}

/**
 * Firestore backed repository for IPO stocks.
 */
export class FirestoreIpoStockRepository implements IpoStockRepository {
  /**
   * Creates the repository.
   */
  public constructor(private readonly firestore: Firestore) {}

  /**
   * Returns stocks whose book-building period contains the given date.
   */
  public async findInBookBuildingPeriod(
    targetDate: string,
  ): Promise<readonly TargetIpoStock[]> {
    const snapshot = await this.firestore.collection("ipo_stocks").get();
    const target = new Date(`${targetDate}T00:00:00.000Z`);

    return snapshot.docs
      .map((document) => document.data() as IpoStockDocument)
      .filter((data) => {
        const startDate = toDateOnlyString(
          data.schedule.bookBuildingPeriod.startDate,
        );
        const endDate = toDateOnlyString(data.schedule.bookBuildingPeriod.endDate);
        return startDate <= targetDate && targetDate <= endDate;
      })
      .map((data) => ({
        identifier: data.identifier,
        companyName: data.companyProfile.companyName,
        price: data.pricing.offerPrice ?? data.pricing.priceRange.minimumPrice,
        shares: data.offering.numberOfOfferedShares,
        bookBuildingStartDate: toDateOnlyString(
          data.schedule.bookBuildingPeriod.startDate,
        ),
        bookBuildingEndDate: toDateOnlyString(
          data.schedule.bookBuildingPeriod.endDate,
        ),
      }) satisfies TargetIpoStock)
      .filter(() => !Number.isNaN(target.getTime()));
  }
}

/**
 * Converts a Firestore timestamp-like value to an ISO date string.
 */
function toDateOnlyString(value: Timestamp | string): string {
  if (value instanceof Timestamp) {
    return value.toDate().toISOString().slice(0, 10);
  }
  return new Date(value).toISOString().slice(0, 10);
}
