---
title: "API仕様書"
version: "1.0.0"
status: "draft"
created: "2026-03-26"
last_updated: "2026-03-26"
author: "lihs"
---

# API仕様書

## 1. はじめに

### 1.1 目的

本文書は、IPOttoのipo-api（Rust/Axum）が提供するREST APIのエンドポイント仕様を定義する。ipo-frontend（Next.js）からの呼び出しを想定する。

### 1.2 ベースURL

| 環境 | ベースURL |
|---|---|
| 開発 | `http://localhost:8080/api/v1` |
| 本番 | `https://ipo-api-xxxxxxxxxx.a.run.app/api/v1` |

### 1.3 関連文書

- [基本設計書 - 外部インターフェース](../02-system-design/system-design.md#7-外部インターフェース)
- [ユースケース層設計書](../03-detailed-design/use-case.md) — 入出力DTO定義
- [詳細設計書 - エラーコード一覧](../03-detailed-design/detailed-design.md#63-エラーコード一覧)

## 2. 共通仕様

### 2.1 リクエスト形式

| 項目 | 仕様 |
|---|---|
| Content-Type | `application/json` |
| 文字コード | UTF-8 |
| 日時フォーマット | ISO 8601（例: `2026-03-26T09:00:00Z`） |
| 金額 | 整数（円単位） |

### 2.2 認証方式

Firebase Authentication の IDトークンをBearerトークンとして付与する。

```
Authorization: Bearer <firebase_id_token>
```

- ipo-apiはFirebase Admin SDKでIDトークンを検証する
- 許可されたUID/メールアドレス以外のアクセスは `403 Forbidden` を返す
- トークンの有効期限は1時間。期限切れ時はフロントエンドがFirebase SDKで自動リフレッシュする

### 2.3 レートリミット

個人利用のため、レートリミットは設定しない。Cloud Runのリクエスト制限（最大同時接続数）がデフォルトの保護として機能する。

### 2.4 ページネーション

操作ログ取得（`GET /api/v1/logs`）のみカーソルベースのページネーションを使用する。

| パラメータ | 型 | デフォルト | 説明 |
|---|---|---|---|
| `cursor` | string | — | 前回レスポンスの `nextCursor` 値 |
| `limit` | integer | 20 | 1ページあたりの件数（最大100） |

## 3. 共通レスポンス形式

### 3.1 成功レスポンス

**単一リソース:**

```json
{
  "identifier": "abc123",
  "companyName": "○○株式会社",
  ...
}
```

**一覧（ページネーションなし）:**

```json
{
  "items": [
    { "identifier": "abc123", ... },
    { "identifier": "def456", ... }
  ],
  "totalCount": 2
}
```

**一覧（カーソルベースページネーション）:**

```json
{
  "items": [ ... ],
  "nextCursor": "eyJpZCI6ImFiYzEyMyJ9",
  "hasMore": true
}
```

### 3.2 エラーレスポンス

```json
{
  "error": {
    "code": "STOCK_NOT_FOUND",
    "message": "指定されたIPO銘柄が見つかりません"
  }
}
```

バリデーションエラー時はフィールド単位の詳細を含む:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "入力値が不正です",
    "details": [
      { "field": "mailAddress", "message": "有効なメールアドレスを入力してください" },
      { "field": "imapPort", "message": "1〜65535の範囲で入力してください" }
    ]
  }
}
```

### 3.3 共通エラーコード

| HTTPステータス | エラーコード | 説明 |
|---|---|---|
| 400 | `VALIDATION_ERROR` | リクエストパラメータが不正 |
| 401 | `AUTHORIZATION_HEADER_INVALID` / `TOKEN_INVALID` / `TOKEN_EXPIRED` / `TOKEN_REJECTED` | IDトークンの欠落・署名不正・期限切れ・iss/aud 不一致 |
| 403 | `EMAIL_MISSING` / `EMAIL_NOT_ALLOWED` / `EMAIL_REQUIRED` | 許可メールリスト (`ALLOWED_EMAIL`) に含まれない、またはトークンに email がない |
| 404 | `NOT_FOUND` | リソースが見つからない |
| 409 | `CONFLICT` | リソースの競合（重複登録等） |
| 422 | `UNPROCESSABLE_ENTITY` | ビジネスルール違反 |
| 500 | `INTERNAL_ERROR` | サーバー内部エラー |
| 503 | `SERVICE_UNAVAILABLE` | 外部サービスとの通信失敗 |

## 4. エンドポイント一覧

| ID | メソッド | パス | 概要 | 認証 | 関連UC |
|---|---|---|---|---|---|
| API-001 | GET | `/api/v1/stocks` | IPO銘柄一覧取得 | 必要 | DD-110 |
| API-002 | GET | `/api/v1/stocks/{stockId}` | IPO銘柄詳細取得 | 必要 | DD-111 |
| API-003 | GET | `/api/v1/dashboard` | ダッシュボードサマリー取得 | 必要 | DD-112 |
| API-004 | GET | `/api/v1/exclusions` | 除外リスト取得 | 必要 | DD-113 |
| API-005 | POST | `/api/v1/exclusions` | 除外銘柄登録 | 必要 | DD-103 |
| API-006 | DELETE | `/api/v1/exclusions/{exclusionId}` | 除外銘柄削除 | 必要 | DD-104 |
| API-007 | GET | `/api/v1/notifications/settings` | 通知設定取得 | 必要 | DD-114 |
| API-008 | PUT | `/api/v1/notifications/settings` | 通知設定更新 | 必要 | DD-105 |
| API-009 | GET | `/api/v1/accounts` | 証券口座一覧取得 | 必要 | DD-115 |
| API-010 | POST | `/api/v1/accounts` | 証券口座登録 | 必要 | DD-106 |
| API-011 | PUT | `/api/v1/accounts/{accountId}` | 証券口座更新 | 必要 | DD-107 |
| API-012 | DELETE | `/api/v1/accounts/{accountId}` | 証券口座削除 | 必要 | DD-108 |
| API-013 | POST | `/api/v1/accounts/{accountId}/test` | 証券口座接続テスト | 必要 | DD-109 |
| API-014 | GET | `/api/v1/logs` | 操作ログ取得 | 必要 | DD-116 |

## 5. エンドポイント詳細

---

### API-001: IPO銘柄一覧取得

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| パス | `/api/v1/stocks` |
| 認証 | 必要 |
| 概要 | IPO銘柄の一覧を取得する。ステータスでフィルタリング可能 |
| 関連要件 | [REQ-001](../01-requirements/requirements-specification.md#req-001) |

**クエリパラメータ:**

| パラメータ | 型 | 必須 | 説明 |
|---|---|---|---|
| `status` | string | No | ステータスでフィルタ（Fetched / Eligible / Applied / Won / Lost / Alternate / Excluded / Failed） |

**成功レスポンス（200 OK）:**

```json
{
  "items": [
    {
      "identifier": "stock_abc123",
      "companyName": "○○株式会社",
      "tickerSymbol": "1234",
      "market": "グロース",
      "industry": "情報・通信業",
      "bookBuildingStartDate": "2026-04-01",
      "bookBuildingEndDate": "2026-04-10",
      "lotteryDate": "2026-04-15",
      "listingDate": "2026-04-25",
      "priceRangeMin": 1200,
      "priceRangeMax": 1500,
      "offerPrice": null,
      "leadUnderwriter": "楽天証券",
      "numberOfOfferedShares": 100000,
      "status": "Eligible"
    }
  ],
  "totalCount": 17
}
```

---

### API-002: IPO銘柄詳細取得

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| パス | `/api/v1/stocks/{stockId}` |
| 認証 | 必要 |
| 概要 | IPO銘柄の詳細情報と、紐づく申し込み状況を取得する |
| 関連要件 | [REQ-001](../01-requirements/requirements-specification.md#req-001) |

**パスパラメータ:**

| パラメータ | 型 | 説明 |
|---|---|---|
| `stockId` | string | IPO銘柄ID |

**成功レスポンス（200 OK）:**

```json
{
  "identifier": "stock_abc123",
  "companyProfile": {
    "companyName": "○○株式会社",
    "tickerSymbol": "1234",
    "market": "グロース",
    "industry": "情報・通信業"
  },
  "schedule": {
    "bookBuildingStartDate": "2026-04-01",
    "bookBuildingEndDate": "2026-04-10",
    "lotteryDate": "2026-04-15",
    "listingDate": "2026-04-25"
  },
  "pricing": {
    "priceRangeMin": 1200,
    "priceRangeMax": 1500,
    "offerPrice": 1400
  },
  "offering": {
    "leadUnderwriter": "楽天証券",
    "numberOfOfferedShares": 100000
  },
  "status": "Applied",
  "metaSource": {
    "source": "ExternalSite",
    "fetchedAt": "2026-03-25T09:30:00Z"
  },
  "applications": [
    {
      "identifier": "app_xyz789",
      "securitiesCompany": "楽天証券",
      "appliedShares": 100,
      "appliedPrice": 1400,
      "appliedAt": "2026-04-05T10:00:00Z",
      "lotteryOutcome": null,
      "status": "Applied"
    }
  ]
}
```

**エラーレスポンス:**

| HTTPステータス | エラーコード | 条件 |
|---|---|---|
| 404 | `STOCK_NOT_FOUND` | 指定されたstockIdが存在しない |

---

### API-003: ダッシュボードサマリー取得

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| パス | `/api/v1/dashboard` |
| 認証 | 必要 |
| 概要 | ダッシュボード表示用のサマリー情報を取得する |
| 関連要件 | [REQ-004](../01-requirements/requirements-specification.md#req-004) |

**成功レスポンス（200 OK）:**

```json
{
  "statusCounts": {
    "Fetched": 2,
    "Eligible": 5,
    "Applied": 3,
    "Won": 1,
    "Lost": 8,
    "Alternate": 0,
    "Excluded": 2,
    "Failed": 0
  },
  "recentActivities": [
    {
      "stock": "stock_abc123",
      "companyName": "○○株式会社",
      "securitiesCompany": "楽天証券",
      "eventType": "ApplicationCompleted",
      "occurredAt": "2026-03-25T10:00:00Z"
    },
    {
      "stock": "stock_def456",
      "companyName": "△△株式会社",
      "securitiesCompany": "楽天証券",
      "eventType": "LotteryResultLost",
      "occurredAt": "2026-03-24T18:00:00Z"
    }
  ],
  "upcomingStocks": [
    {
      "stock": "stock_ghi789",
      "companyName": "▲▲株式会社",
      "bookBuildingStartDate": "2026-04-01",
      "bookBuildingEndDate": "2026-04-10",
      "lotteryDate": "2026-04-15"
    }
  ],
  "systemStatus": {
    "nextJobScheduledAt": "2026-04-01T09:00:00Z",
    "accounts": [
      {
        "securitiesCompany": "楽天証券",
        "connectionStatus": "healthy",
        "lastTestedAt": "2026-03-25T10:00:00Z"
      }
    ]
  }
}
```

---

### API-004: 除外リスト取得

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| パス | `/api/v1/exclusions` |
| 認証 | 必要 |
| 概要 | 除外銘柄の一覧を取得する |
| 関連要件 | [REQ-005](../01-requirements/requirements-specification.md#req-005) |

**成功レスポンス（200 OK）:**

```json
{
  "items": [
    {
      "identifier": "excl_abc123",
      "companyName": "○○株式会社",
      "reason": "自社（インサイダー規制対応）",
      "registeredAt": "2026-03-20T12:00:00Z"
    }
  ],
  "totalCount": 1
}
```

---

### API-005: 除外銘柄登録

| 項目 | 内容 |
|---|---|
| メソッド | `POST` |
| パス | `/api/v1/exclusions` |
| 認証 | 必要 |
| 概要 | 除外リストに銘柄を登録する |
| 関連要件 | [REQ-005](../01-requirements/requirements-specification.md#req-005) |

**リクエストボディ:**

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `companyName` | string | Yes | 除外対象の企業名（1〜200文字） |
| `reason` | string | Yes | 除外理由（1〜500文字） |

**リクエスト例:**

```json
{
  "companyName": "○○株式会社",
  "reason": "自社（インサイダー規制対応）"
}
```

**成功レスポンス（201 Created）:**

```json
{
  "identifier": "excl_abc123",
  "companyName": "○○株式会社"
}
```

**エラーレスポンス:**

| HTTPステータス | エラーコード | 条件 |
|---|---|---|
| 400 | `VALIDATION_ERROR` | 企業名が空または200文字超過 |
| 409 | `CONFLICT` | 同一企業名が既に除外リストに登録済み |

---

### API-006: 除外銘柄削除

| 項目 | 内容 |
|---|---|
| メソッド | `DELETE` |
| パス | `/api/v1/exclusions/{exclusionId}` |
| 認証 | 必要 |
| 概要 | 除外リストから銘柄を削除する |
| 関連要件 | [REQ-005](../01-requirements/requirements-specification.md#req-005) |

**パスパラメータ:**

| パラメータ | 型 | 説明 |
|---|---|---|
| `exclusionId` | string | 除外銘柄ID |

**成功レスポンス（204 No Content）:**

レスポンスボディなし。

**エラーレスポンス:**

| HTTPステータス | エラーコード | 条件 |
|---|---|---|
| 404 | `EXCLUSION_NOT_FOUND` | 指定されたexclusionIdが存在しない |

---

### API-007: 通知設定取得

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| パス | `/api/v1/notifications/settings` |
| 認証 | 必要 |
| 概要 | 通知設定を取得する |
| 関連要件 | [REQ-006](../01-requirements/requirements-specification.md#req-006) |

**成功レスポンス（200 OK）:**

```json
{
  "identifier": "default",
  "enabled": true,
  "channels": [
    {
      "identifier": "ch_line_001",
      "channelType": "LINE",
      "destination": {
        "token": "xxxxxxxxxxxxxxxx"
      },
      "enabled": true,
      "subscriptions": {
        "ApplicationCompleted": true,
        "LotteryResultWon": true,
        "LotteryResultLost": false,
        "OperationError": true,
        "StockUpdated": false
      }
    },
    {
      "identifier": "ch_slack_001",
      "channelType": "Slack",
      "destination": {
        "webhookUrl": "https://hooks.slack.com/services/xxx/yyy/zzz"
      },
      "enabled": true,
      "subscriptions": {
        "ApplicationCompleted": true,
        "LotteryResultWon": true,
        "LotteryResultLost": true,
        "OperationError": true,
        "StockUpdated": true
      }
    }
  ]
}
```

---

### API-008: 通知設定更新

| 項目 | 内容 |
|---|---|
| メソッド | `PUT` |
| パス | `/api/v1/notifications/settings` |
| 認証 | 必要 |
| 概要 | 通知設定を更新する。全体の有効/無効、チャネル設定、イベント購読をまとめて更新 |
| 関連要件 | [REQ-006](../01-requirements/requirements-specification.md#req-006) |

**リクエストボディ:**

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `enabled` | boolean | Yes | 通知全体の有効/無効 |
| `channels` | array | Yes | チャネル設定の配列 |
| `channels[].channelType` | string | Yes | `LINE` / `Email` / `Slack` |
| `channels[].destination` | object | Yes | チャネル固有の送信先設定 |
| `channels[].enabled` | boolean | Yes | チャネルの有効/無効 |
| `channels[].subscriptions` | object | Yes | イベントごとの購読設定 |

**リクエスト例:**

```json
{
  "enabled": true,
  "channels": [
    {
      "channelType": "Slack",
      "destination": {
        "webhookUrl": "https://hooks.slack.com/services/xxx/yyy/zzz"
      },
      "enabled": true,
      "subscriptions": {
        "ApplicationCompleted": true,
        "LotteryResultWon": true,
        "LotteryResultLost": true,
        "OperationError": true,
        "StockUpdated": true
      }
    }
  ]
}
```

**成功レスポンス（200 OK）:**

```json
{
  "identifier": "default",
  "enabled": true,
  "channelCount": 1
}
```

**エラーレスポンス:**

| HTTPステータス | エラーコード | 条件 |
|---|---|---|
| 400 | `VALIDATION_ERROR` | チャネル設定が不正 |
| 409 | `DUPLICATE_CHANNEL_TYPE` | 同一チャネルタイプが重複 |
| 422 | `NO_ACTIVE_CHANNEL` | 通知有効時に有効なチャネルが0件 |

---

### API-009: 証券口座一覧取得

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| パス | `/api/v1/accounts` |
| 認証 | 必要 |
| 概要 | 登録済み証券口座の一覧を取得する。機密情報（パスワード等）は含まない |
| 関連要件 | [REQ-007](../01-requirements/requirements-specification.md#req-007) |

**成功レスポンス（200 OK）:**

```json
{
  "items": [
    {
      "identifier": "acct_abc123",
      "securitiesCompany": "Rakuten",
      "loginId": "user***",
      "mailAddress": "user***@example.com",
      "isActive": true,
      "connectionTest": {
        "success": true,
        "message": "接続成功",
        "testedAt": "2026-03-25T10:00:00Z"
      }
    }
  ],
  "totalCount": 1
}
```

> **注記:** `loginId` と `mailAddress` はマスク表示（先頭3文字 + `***`）。パスワード・取引暗証番号はレスポンスに含めない。

---

### API-010: 証券口座登録

| 項目 | 内容 |
|---|---|
| メソッド | `POST` |
| パス | `/api/v1/accounts` |
| 認証 | 必要 |
| 概要 | 証券口座を登録する。機密情報はSecret Managerに暗号化保存される |
| 関連要件 | [REQ-007](../01-requirements/requirements-specification.md#req-007) |

**リクエストボディ:**

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `securitiesCompany` | string | Yes | 証券会社コード（Phase 1 MVP は `Rakuten`、Phase 8 で `Nomura` を追加。詳細は [野村證券対応 要件定義書](../01-requirements/nomura-broker-support.md)） |
| `loginId` | string | Yes | ログインID |
| `loginPassword` | string | Yes | ログインパスワード |
| `tradingPassword` | string | Yes | 取引暗証番号 |
| `mailAddress` | string | Yes | 2FA認証メールの受信アドレス |
| `mailPassword` | string | Yes | メールアカウントのパスワード |
| `imapHost` | string | Yes | IMAPサーバーホスト名 |
| `imapPort` | integer | Yes | IMAPサーバーポート番号（1〜65535） |

**リクエスト例:**

```json
{
  "securitiesCompany": "Rakuten",
  "loginId": "myloginid",
  "loginPassword": "mypassword",
  "tradingPassword": "1234",
  "mailAddress": "myemail@gmail.com",
  "mailPassword": "app-specific-password",
  "imapHost": "imap.gmail.com",
  "imapPort": 993
}
```

**成功レスポンス（201 Created）:**

```json
{
  "identifier": "acct_abc123",
  "securitiesCompany": "Rakuten"
}
```

**エラーレスポンス:**

| HTTPステータス | エラーコード | 条件 |
|---|---|---|
| 400 | `VALIDATION_ERROR` | 必須フィールドが欠損またはフォーマット不正 |
| 400 | `INCOMPLETE_CREDENTIAL` | 機密情報が不完全 |
| 400 | `INVALID_MAIL_ADDRESS` | メールアドレスの形式が不正 |

---

### API-011: 証券口座更新

| 項目 | 内容 |
|---|---|
| メソッド | `PUT` |
| パス | `/api/v1/accounts/{accountId}` |
| 認証 | 必要 |
| 概要 | 証券口座の情報を更新する。指定されたフィールドのみ更新 |
| 関連要件 | [REQ-007](../01-requirements/requirements-specification.md#req-007) |

**パスパラメータ:**

| パラメータ | 型 | 説明 |
|---|---|---|
| `accountId` | string | 証券口座ID |

**リクエストボディ:**

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `loginId` | string | No | ログインID |
| `loginPassword` | string | No | ログインパスワード |
| `tradingPassword` | string | No | 取引暗証番号 |
| `mailAddress` | string | No | 2FA認証メールの受信アドレス |
| `mailPassword` | string | No | メールアカウントのパスワード |
| `imapHost` | string | No | IMAPサーバーホスト名 |
| `imapPort` | integer | No | IMAPサーバーポート番号 |

> 省略されたフィールドは既存値を保持する。

**成功レスポンス（200 OK）:**

```json
{
  "identifier": "acct_abc123"
}
```

**エラーレスポンス:**

| HTTPステータス | エラーコード | 条件 |
|---|---|---|
| 404 | `ACCOUNT_NOT_FOUND` | 指定されたaccountIdが存在しない |
| 400 | `INVALID_MAIL_ADDRESS` | メールアドレスの形式が不正 |

---

### API-012: 証券口座削除

| 項目 | 内容 |
|---|---|
| メソッド | `DELETE` |
| パス | `/api/v1/accounts/{accountId}` |
| 認証 | 必要 |
| 概要 | 証券口座を削除する。Secret Manager上の機密情報も削除される |
| 関連要件 | [REQ-007](../01-requirements/requirements-specification.md#req-007) |

**パスパラメータ:**

| パラメータ | 型 | 説明 |
|---|---|---|
| `accountId` | string | 証券口座ID |

**成功レスポンス（204 No Content）:**

レスポンスボディなし。

**エラーレスポンス:**

| HTTPステータス | エラーコード | 条件 |
|---|---|---|
| 404 | `ACCOUNT_NOT_FOUND` | 指定されたaccountIdが存在しない |

---

### API-013: 証券口座接続テスト

| 項目 | 内容 |
|---|---|
| メソッド | `POST` |
| パス | `/api/v1/accounts/{accountId}/test` |
| 認証 | 必要 |
| 概要 | 証券口座の認証情報で実際にログインを試行し、接続状態を確認する |
| 関連要件 | [REQ-007](../01-requirements/requirements-specification.md#req-007) |

**パスパラメータ:**

| パラメータ | 型 | 説明 |
|---|---|---|
| `accountId` | string | 証券口座ID |

**リクエストボディ:** なし

**成功レスポンス（200 OK）:**

```json
{
  "success": true,
  "message": "接続成功",
  "testedAt": "2026-03-26T10:00:00Z"
}
```

**テスト失敗時の成功レスポンス（200 OK）:**

```json
{
  "success": false,
  "message": "ログインIDまたはパスワードが不正です",
  "testedAt": "2026-03-26T10:00:00Z"
}
```

> **注記:** 接続テスト自体は正常に完了したが、ログインに失敗した場合は200 OKで `success: false` を返す。テスト処理自体がエラーになった場合のみ5xxを返す。

**エラーレスポンス:**

| HTTPステータス | エラーコード | 条件 |
|---|---|---|
| 404 | `ACCOUNT_NOT_FOUND` | 指定されたaccountIdが存在しない |
| 503 | `BROWSER_OPERATION_FAILED` | ブラウザ操作自体に失敗（タイムアウト等） |

---

### API-014: 操作ログ取得

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| パス | `/api/v1/logs` |
| 認証 | 必要 |
| 概要 | 自動操作の操作ログを取得する。日付範囲・イベント種別でフィルタリング可能 |
| 関連要件 | [REQ-008](../01-requirements/requirements-specification.md#req-008) |

**クエリパラメータ:**

| パラメータ | 型 | 必須 | 説明 |
|---|---|---|---|
| `startDate` | string (date) | No | 開始日（ISO 8601 日付） |
| `endDate` | string (date) | No | 終了日（ISO 8601 日付） |
| `eventType` | string | No | イベント種別でフィルタ。取りうる値: `fetch_stocks` / `apply_lottery` / `check_lottery_result` / `notification_dispatch` / `connection_test` / `other` (snake_case)。`OperationEventType` enum と一対一対応 |
| `cursor` | string | No | ページネーションカーソル |
| `limit` | integer | No | 取得件数（デフォルト20、最大100） |

**成功レスポンス（200 OK）:**

レスポンスの `eventType` は上記クエリパラメータと同じ snake_case 値を返す。通知購読で使う `NotificationEventType` (PascalCase、例: `"ApplicationCompleted"`) とは別体系である点に注意。

```json
{
  "items": [
    {
      "identifier": "log_abc123",
      "eventType": "apply_lottery",
      "serviceName": "ipo-browser",
      "status": "success",
      "message": "○○株式会社のIPO抽選に申し込みました（楽天証券、100株、1,400円）",
      "errorMessage": null,
      "executedAt": "2026-03-25T10:02:00Z"
    },
    {
      "identifier": "log_def456",
      "eventType": "connection_test",
      "serviceName": "ipo-browser",
      "status": "success",
      "message": "楽天証券にログインしました",
      "errorMessage": null,
      "executedAt": "2026-03-25T10:00:00Z"
    }
  ],
  "nextCursor": "eyJpZCI6ImxvZ19kZWY0NTYifQ",
  "hasMore": true
}
```

## 変更履歴

| バージョン | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 1.0.0 | 2026-03-26 | lihs | 初版作成 |
