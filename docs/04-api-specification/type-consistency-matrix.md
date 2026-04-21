---
title: "API 型整合マトリクス"
version: "0.1.0"
status: "draft"
created: "2026-04-21"
last_updated: "2026-04-21"
author: "lihs"
---

# API 型整合マトリクス (Phase 0 Task 0.4)

`docs/04-api-specification/api-specification.md` で定義する 14 エンドポイント (API-001 ~ 014) のリクエスト/レスポンスと、`ipo-backend-shared` のドメイン型 + `ipo-api` の use case output の整合を確認する。

---

## 1. スコープ・前提

- **対象エンドポイント**: API-001 ~ API-014。
- **比較軸**: (a) レスポンス/リクエストに含まれるフィールド名、(b) 値の serialize 形式、(c) マスキング等の加工の有無。
- **変換モデル**: API は `domain → use case output (application/use_cases/*_output)` → `handler` の 2 段変換で生成される。
  - ドメイン enum (`Market`, `ChannelType`, `StockStatus` など) は **use case 層で `.as_str().to_string()` により文字列化** されて output 構造体に詰められる。serde の `derive` による JSON 形式がそのまま外に出るのは使用 **していない**（重要な前提）。
  - 機密情報 (`LoginId`, `LoginPassword`, `MailAddress` など) は use case 層で `mask_value` / `mask_mail_address` を介してマスクされる。
- **集約**: stock / exclusion / application / notification / account / operation_log / lottery_application。

---

## 2. エンドポイント × 主要フィールド対応表

### 2.1 参照系

| API ID | メソッド / パス | 代表フィールド | 元になる domain 型 | use case 層での変換 | 備考 |
|---|---|---|---|---|---|
| API-001 | `GET /stocks` | `items[].identifier`, `market`, `status`, `bookBuildingStartDate` | `IpoStock`, `Market`, `StockStatus` | `market().as_str()`, `status().as_str()`, `start().to_string()` | 整合 |
| API-002 | `GET /stocks/{id}` | `companyProfile.market`, `schedule.listingDate`, `metaSource.source` | `IpoStock`, `FetchOrigin` | `FetchOrigin::{ExternalSite, OfficialSite, Unknown}.as_str()` | 整合 |
| API-003 | `GET /dashboard` | `statusCounts{}`, `recentActivities[].eventType`, `systemStatus` | 複数集約を use case で集計 | `StockStatus::as_str()` を key、`NotificationEventType::as_str()` を eventType に | §3-b 参照 |
| API-004 | `GET /exclusions` | `items[].identifier`, `companyName`, `reason`, `registeredAt` | `Exclusion` (`ExclusionReason` VO) | VO を `.value().to_string()` で平坦化 | 整合 |
| API-007 | `GET /notifications/settings` | `enabled`, `channels[].channelType`, `channels[].destination{}`, `channels[].subscriptions{}` | `NotificationSetting`, `NotificationChannel`, `ChannelType` | `channel_type().as_str()` で `"LINE"` / `"Email"` / `"Slack"` | §3-a 参照 |
| API-009 | `GET /accounts` | `items[].securitiesCompany`, `loginId`, `mailAddress`, `activation.active` | `SecuritiesAccount` | `mask_value(login_id.value())`, `mask_mail_address(...)` | §3-b 正当なマスキング |
| API-014 | `GET /logs` | `items[].eventType`, `status`, `details{}`, `recordedAt` | `OperationLog`, `OperationEventType` | `event_type.as_str()` で `"ApplyLottery"` など | §3-c 参照 |

### 2.2 操作系

| API ID | メソッド / パス | 代表フィールド | 元になる domain 型 | 備考 |
|---|---|---|---|---|
| API-005 | `POST /exclusions` | `identifier`, `companyName` | `Exclusion::create` | 登録フィールドは `companyName` / `reason` のみ。整合 |
| API-006 | `DELETE /exclusions/{id}` | 204 No Content | — | 整合 |
| API-008 | `PUT /notifications/settings` | `channelCount` | `NotificationSetting::reconstruct` | リクエストは channels 配列全置換。整合 |
| API-010 | `POST /accounts` | `identifier`, `securitiesCompany` | `SecuritiesAccount::register` | 機密フィールド (`loginId` / `password` 等) はリクエストのみで受け取り、レスポンスにはマスク済みで出す方針 |
| API-011 | `PUT /accounts/{id}` | `identifier` | `SecuritiesAccount::update_credentials` | 整合 |
| API-012 | `DELETE /accounts/{id}` | 204 No Content | — | 整合 |
| API-013 | `POST /accounts/{id}/test` | `success`, `message`, `testedAt` | `ConnectionTestResult` | `result.success()`, `result.message()` を直接載せる。整合 |

---

## 3. 特記すべき差異

### (a) ドメイン enum / serde 設定の微妙な非対称性

1. **`ChannelType` の serde 形式 vs 文字列形式が非対称**
   - `ChannelType::Line` の serde derive 出力は `"Line"` （variant 名そのまま）。
   - `ChannelType::as_str()` は `"LINE"` （全大文字）。
   - 現状は **use case 層で `as_str()` 経由で文字列化** されるため API には `"LINE"` が出る。API spec も `"LINE"` を要求しており整合している。
   - **リスク**: 将来 use case output を介さず domain 型を直接 serialize するパス（例: Pub/Sub event envelope 内の payload、Firestore に domain 型を直接書き込むコード）が生まれると、`"Line"` が外に出て spec と不整合になる。
   - **対策案**: ドメイン enum に `#[serde(rename_all = "UPPERCASE")]` を入れるか、公開 API に出る enum は独立 DTO を置いて serde 挙動を固定する。**本 Phase 0 では対応しないが、Phase 1 Sprint 2 (API 実装の本格化) で決める**。契約テストで serde derive 出力を固定し、回帰を検知する。

2. **`Market` の serde 形式と API spec の表示言語の乖離**
   - 実装: `Market::Growth` serde 出力 = `"Growth"`、`as_str()` も `"Growth"` で統一済み。
   - API spec: レスポンス例で `"market": "グロース"` と日本語ラベルを例示 (`docs/04-api-specification/api-specification.md` §4.1)。
   - **実害**: フロントエンド実装者が JP 文字列を期待してコードを書くと不整合。
   - **対策**: spec 側を `"Growth"` に修正するのが最小コスト。`Market::new` は JP / EN 双方を受け付けるので、入力側は後方互換。**後続 PR で spec 修正**。

3. **`StockStatus` の variants 一覧が spec に揃っていない**
   - 実装 (`services/ipo-backend-shared/src/domain/stock/stock_status.rs`): Fetched / Eligible / Applied / Won / Lost / Alternate / Purchased / Declined / Sold / Excluded / Failed。
   - spec の `statusCounts{}` 例: Fetched / Eligible / Applied / Won / Lost / Alternate / Excluded / Failed （Purchased / Declined / Sold が抜けている）。
   - **対策**: spec 側に Purchased / Declined / Sold を追記 or 「購入後ステータスはダッシュボード集計から除外する」旨を明記。**後続 PR で対応**。

4. **`OperationEventType` の serde 出力と `as_str()` が大きく乖離** (重要)
   - 実装 (`services/ipo-backend-shared/src/domain/operation_log/operation_event_type.rs`):
     - serde derive 出力 (variant 名): `"FetchStocks"` / `"ApplyLottery"` / `"CheckLotteryResult"` / `"NotificationDispatch"` / `"ConnectionTest"` / `"Other"` (PascalCase)。
     - `as_str()` 出力: `"fetch_stocks"` / `"apply_lottery"` / `"check_lottery_result"` / `"notification_dispatch"` / `"connection_test"` / `"other"` (snake_case)。
   - 同一 enum で PascalCase と snake_case が両立しており、**どちらが API/Firestore の正** なのかがコードだけでは読み取れない。
   - ipo-api / ipo-result-checker / ipo-info-fetcher の実コードを確認し、Firestore に書くときと API に出すときで使っている関数を統一する必要がある。
   - **対策**: Phase 1 Sprint 2 の API-014 実装着手前に、serde 出力 (PascalCase) に寄せるか snake_case に寄せるかを決定し、片方を削除する。本 Phase 0 では契約テストで現状の挙動（両立）を固定化し、齟齬を可視化する。

### (b) プレゼンテーション層で吸収すべき正当な加工

1. **証券口座レスポンスのマスキング**
   - `loginId`: `user***` の形。`mail_address`: `u***@example.com` の形。
   - use case 側で `mask_value` / `mask_mail_address` 関数を介して生成する。
   - **評価**: ドメイン型と API レスポンスが意図的に非対称（セキュリティ要件）。正当。契約テスト対象外。

2. **ダッシュボードの `recentActivities[].eventType`**
   - 値 (例): `ApplicationCompleted`, `LotteryResultWon`, `LotteryResultLost`, `LotteryResultAlternate`。
   - これは **`NotificationEventType` enum** の `as_str()` を利用する。`OperationEventType` とは別体系。
   - **評価**: 意図的な分離。通知購読の対象イベントと、操作ログの記録対象イベントは独立。契約テストで両 enum の serialize が干渉しないことを確認する。

### (c) API spec を修正すべき誤記・不足

| 項目 | 現状 | 修正方針 | 優先度 |
|---|---|---|---|
| API-001 `market` 値例 | `"グロース"` | `"Growth"` に変更 | Medium |
| API-003 `statusCounts` keys | Purchased/Declined/Sold 欠落 | 全 StockStatus variant を列挙 or 非対象を明記 | Medium |
| API-014 クエリ `eventType` 取りうる値 | 列挙なし | `OperationEventType::as_str()` の値 (`ApplyLottery`, `FetchStocks`, `CheckLotteryResult`, `NotificationDispatch`, `ConnectionTest`, `Other`) を列挙 | Medium |
| `NotificationEventType` と `OperationEventType` の使い分け | 記述なし | 「通知系は NotificationEventType、ログ系は OperationEventType」を共通仕様 §2 に明記 | Low |

---

## 4. 後続 PR 候補

| 優先度 | タイトル案 | スコープ |
|---|---|---|
| Medium | `docs(api-spec): fix market example and list all StockStatus values` | (c) 表の API-001 / API-003 項目 |
| Medium | `docs(api-spec): document supported eventType values for GET /logs` | (c) 表の API-014 項目 |
| Low | `fix(backend-shared): pin serde rename for public-facing enums` | (a) の ChannelType / NotificationEventType に `#[serde(rename_all)]` を付与し、契約テストを厳格化 |

---

## 5. 契約テスト (serde roundtrip)

`services/ipo-backend-shared/tests/api_contract.rs` に以下を実装し、将来 enum や構造体の serde 属性が変わった場合に CI で回帰を検知する:

| 対象 | 検証項目 |
|---|---|
| `ChannelType` | serde derive 出力が `"Line"` / `"Email"` / `"Slack"`、`as_str()` が `"LINE"` / `"Email"` / `"Slack"` |
| `Market` | serde derive 出力と `as_str()` が `"Prime"` / `"Standard"` / `"Growth"` で一致。日本語からも `Market::new` で復元可能 |
| `StockStatus` | 全 11 variant が serde derive → 文字列 → `StockStatus::from_str` 相当で roundtrip（現状 `as_str()` のみ） |
| `NotificationEventType` | `as_str()` で `"ApplicationCompleted"` 等、serde derive も同一 |
| `OperationEventType` | `as_str()` で `"ApplyLottery"` 等、NotificationEventType と別体系であることを試験 |
| `FetchOrigin` | serde derive で `"ExternalSite"` / `"OfficialSite"` / `"Unknown"`、`as_str()` と一致 |
| `LotteryResult` | `"Won"` / `"Lost"` / `"Alternate"` の roundtrip |
| `ConnectionTestResult` | 成功レスポンスのフィールド構造 (success, message, tested_at) |

テスト実装後は `cargo test -p ipo-backend-shared --test api_contract` で個別に実行できる。

---

## 6. 結論

- **Phase 0 のマイルストーン M0 判定には重大齟齬なし**: use case 層での明示的な文字列化によって、domain enum serde derive の内部挙動と API の期待値が実質的に吻合している。
- **§3-c の 3 項目は spec 側修正で解消** (後続 PR)。Phase 1 着手の blocker にはならない。
- **契約テスト (§5) を追加することで、将来 domain enum の serde 挙動を変えた場合に API 仕様との乖離を検知できる状態** にする。これを本 PR (Task 0.4) のスコープに含める。
