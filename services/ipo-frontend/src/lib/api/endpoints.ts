import {
  createExclusionRequestSchema,
  createExclusionResponseSchema,
  createSecuritiesAccountRequestSchema,
  createSecuritiesAccountResponseSchema,
  dashboardSummaryResponseSchema,
  getIpoStockResponseSchema,
  getNotificationSettingResponseSchema,
  listExclusionsResponseSchema,
  listIpoStocksQuerySchema,
  listIpoStocksResponseSchema,
  listOperationLogsQuerySchema,
  listOperationLogsResponseSchema,
  listSecuritiesAccountsResponseSchema,
  testSecuritiesAccountConnectionResponseSchema,
  updateNotificationSettingRequestSchema,
  updateNotificationSettingResponseSchema,
  updateSecuritiesAccountRequestSchema,
  updateSecuritiesAccountResponseSchema,
  type CreateExclusionRequest,
  type CreateExclusionResponse,
  type CreateSecuritiesAccountRequest,
  type CreateSecuritiesAccountResponse,
  type DashboardSummaryResponse,
  type ExclusionIdentifier,
  type GetIpoStockResponse,
  type GetNotificationSettingResponse,
  type ListExclusionsResponse,
  type ListIpoStocksQuery,
  type ListIpoStocksResponse,
  type ListOperationLogsQuery,
  type ListOperationLogsResponse,
  type ListSecuritiesAccountsResponse,
  type SecuritiesAccountIdentifier,
  type StockIdentifier,
  type TestSecuritiesAccountConnectionResponse,
  type UpdateNotificationSettingRequest,
  type UpdateNotificationSettingResponse,
  type UpdateSecuritiesAccountRequest,
  type UpdateSecuritiesAccountResponse,
} from "@ipotto/shared";

import { type ApiClient } from "./client";
import { type ApiError } from "./error";
import { type AsyncResult } from "./result";

export type IpoApi = {
  /// API-001
  listIpoStocks: (
    query?: ListIpoStocksQuery,
  ) => AsyncResult<ListIpoStocksResponse, ApiError>;
  /// API-002
  getIpoStock: (
    identifier: StockIdentifier,
  ) => AsyncResult<GetIpoStockResponse, ApiError>;
  /// API-003
  getDashboardSummary: () => AsyncResult<DashboardSummaryResponse, ApiError>;
  /// API-004
  listExclusions: () => AsyncResult<ListExclusionsResponse, ApiError>;
  /// API-005
  createExclusion: (
    payload: CreateExclusionRequest,
  ) => AsyncResult<CreateExclusionResponse, ApiError>;
  /// API-006
  deleteExclusion: (
    identifier: ExclusionIdentifier,
  ) => AsyncResult<null, ApiError>;
  /// API-007
  getNotificationSetting: () => AsyncResult<
    GetNotificationSettingResponse,
    ApiError
  >;
  /// API-008
  updateNotificationSetting: (
    payload: UpdateNotificationSettingRequest,
  ) => AsyncResult<UpdateNotificationSettingResponse, ApiError>;
  /// API-009
  listSecuritiesAccounts: () => AsyncResult<
    ListSecuritiesAccountsResponse,
    ApiError
  >;
  /// API-010
  createSecuritiesAccount: (
    payload: CreateSecuritiesAccountRequest,
  ) => AsyncResult<CreateSecuritiesAccountResponse, ApiError>;
  /// API-011
  updateSecuritiesAccount: (
    identifier: SecuritiesAccountIdentifier,
    payload: UpdateSecuritiesAccountRequest,
  ) => AsyncResult<UpdateSecuritiesAccountResponse, ApiError>;
  /// API-012
  deleteSecuritiesAccount: (
    identifier: SecuritiesAccountIdentifier,
  ) => AsyncResult<null, ApiError>;
  /// API-013
  testSecuritiesAccountConnection: (
    identifier: SecuritiesAccountIdentifier,
  ) => AsyncResult<TestSecuritiesAccountConnectionResponse, ApiError>;
  /// API-014
  listOperationLogs: (
    query?: ListOperationLogsQuery,
  ) => AsyncResult<ListOperationLogsResponse, ApiError>;
};

const toQueryRecord = (
  query: Record<string, unknown> | undefined,
): Record<string, string | number | boolean | undefined> | undefined => {
  if (!query) return undefined;
  const record: Record<string, string | number | boolean | undefined> = {};
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined) continue;
    if (
      typeof value === "string" ||
      typeof value === "number" ||
      typeof value === "boolean"
    ) {
      record[key] = value;
    } else {
      record[key] = String(value);
    }
  }
  return record;
};

export const createIpoApi = (client: ApiClient): IpoApi => ({
  listIpoStocks: (query) => {
    const parsed = query ? listIpoStocksQuerySchema.parse(query) : {};
    return client.get("/api/v1/stocks", listIpoStocksResponseSchema, {
      query: toQueryRecord(parsed),
    });
  },
  getIpoStock: (identifier) =>
    client.get(
      `/api/v1/stocks/${encodeURIComponent(identifier)}`,
      getIpoStockResponseSchema,
    ),
  getDashboardSummary: () =>
    client.get("/api/v1/dashboard", dashboardSummaryResponseSchema),
  listExclusions: () =>
    client.get("/api/v1/exclusions", listExclusionsResponseSchema),
  createExclusion: (payload) =>
    client.post("/api/v1/exclusions", createExclusionResponseSchema, {
      body: createExclusionRequestSchema.parse(payload),
    }),
  deleteExclusion: (identifier) =>
    client.delete(`/api/v1/exclusions/${encodeURIComponent(identifier)}`),
  getNotificationSetting: () =>
    client.get(
      "/api/v1/notifications/settings",
      getNotificationSettingResponseSchema,
    ),
  updateNotificationSetting: (payload) =>
    client.put(
      "/api/v1/notifications/settings",
      updateNotificationSettingResponseSchema,
      { body: updateNotificationSettingRequestSchema.parse(payload) },
    ),
  listSecuritiesAccounts: () =>
    client.get("/api/v1/accounts", listSecuritiesAccountsResponseSchema),
  createSecuritiesAccount: (payload) =>
    client.post("/api/v1/accounts", createSecuritiesAccountResponseSchema, {
      body: createSecuritiesAccountRequestSchema.parse(payload),
    }),
  updateSecuritiesAccount: (identifier, payload) =>
    client.put(
      `/api/v1/accounts/${encodeURIComponent(identifier)}`,
      updateSecuritiesAccountResponseSchema,
      { body: updateSecuritiesAccountRequestSchema.parse(payload) },
    ),
  deleteSecuritiesAccount: (identifier) =>
    client.delete(`/api/v1/accounts/${encodeURIComponent(identifier)}`),
  testSecuritiesAccountConnection: (identifier) =>
    client.post(
      `/api/v1/accounts/${encodeURIComponent(identifier)}/test`,
      testSecuritiesAccountConnectionResponseSchema,
    ),
  listOperationLogs: (query) => {
    const parsed = query ? listOperationLogsQuerySchema.parse(query) : {};
    return client.get("/api/v1/logs", listOperationLogsResponseSchema, {
      query: toQueryRecord(parsed),
    });
  },
});
