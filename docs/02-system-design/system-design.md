---
title: 基本設計書
version: "1.0.0"
created: 2026-03-25
last_updated: 2026-03-25
status: draft
author: lihs
---

# 基本設計書

## 1. はじめに

### 1.1 目的

本文書は、IPO抽選申し込み自動化・進捗管理Webアプリケーション「IPOtto」の基本設計を定義する。システム全体のアーキテクチャ、サービス構成、技術スタック、画面設計、外部インターフェース、概念データモデルの概要を示す。

### 1.2 対象読者

- 開発者（自身）
- 将来的なコントリビューター

### 1.3 関連文書

- [要件定義書](../01-requirements/requirements-specification.md)
- [ペルソナ設計書](../13-persona/persona.md)

## 2. システム概要

IPOttoは、個人投資家向けのIPO抽選申し込み自動化Webアプリケーションである。以下の5つのマイクロサービスで構成され、Pub/Subによるイベント駆動アーキテクチャで疎結合に連携する。

- **ipo-api**: Web UI向けREST APIサーバー（Rust/Axum）
- **ipo-browser**: 証券会社サイトのブラウザ自動操作サービス（Node.js/Playwright）
- **ipo-info-fetcher**: 外部IPO情報サイトからの情報取得ジョブ（Rust）
- **ipo-result-checker**: 抽選結果確認ジョブ（Rust）
- **ipo-frontend**: ダッシュボードUI（Next.js）

認証にはFirebase Authentication、データストアにはFirestoreを採用し、GCPの無料枠を最大限に活用する。

## 3. システム構成図

```mermaid
graph TB
    subgraph クライアント
        Browser[Webブラウザ<br>PC / スマートフォン]
    end

    subgraph GCP
        subgraph Cloud Run
            Frontend["ipo-frontend<br>(Next.js)<br>always-on"]
            API["ipo-api<br>(Rust/Axum)<br>always-on"]
            BrowserService["ipo-browser<br>(Node.js/Playwright)<br>on-demand"]
            InfoFetcher["ipo-info-fetcher<br>(Rust)<br>on-demand"]
            ResultChecker["ipo-result-checker<br>(Rust)<br>on-demand"]
        end

        Scheduler[Cloud Scheduler]
        PubSub[Cloud Pub/Sub]
        SecretManager[Secret Manager]

        subgraph Firebase
            FireAuth[Firebase Authentication]
            Firestore[(Firestore)]
        end
    end

    subgraph 外部サービス
        SecuritiesSite[楽天証券<br>Webサイト]
        IPOInfoSite[IPO情報サイト]
        LINE[LINE Notify]
        Email[SendGrid]
        Slack[Slack Webhook]
    end

    Browser --> Frontend
    Frontend --> API
    Browser --> FireAuth

    Scheduler -->|cron| PubSub
    PubSub -->|ipo-info-fetch| InfoFetcher
    PubSub -->|ipo-apply| BrowserService
    PubSub -->|ipo-result-check| ResultChecker

    API --> Firestore
    API --> SecretManager
    API --> LINE
    API --> Email
    API --> Slack

    InfoFetcher --> IPOInfoSite
    InfoFetcher --> Firestore
    InfoFetcher -->|publish: ipo-info-updated| PubSub

    BrowserService --> SecuritiesSite
    BrowserService --> Firestore
    BrowserService --> SecretManager

    ResultChecker --> BrowserService
    ResultChecker --> Firestore
    ResultChecker -->|publish: ipo-result-updated| PubSub
```

### 3.1 イベントフロー

```mermaid
graph LR
    subgraph トリガー
        CS[Cloud Scheduler<br>cron]
    end

    subgraph Pub/Sub Topics
        T1[ipo-job-trigger]
        T2[ipo-info-updated]
        T3[ipo-result-updated]
        T4[ipo-notification]
    end

    subgraph サービス
        S1[ipo-info-fetcher]
        S2[ipo-browser]
        S3[ipo-result-checker]
        S4[ipo-api<br>通知処理]
    end

    CS -->|publish| T1
    T1 -->|subscription: fetch| S1
    T1 -->|subscription: apply| S2
    T1 -->|subscription: check| S3

    S1 -->|publish| T2
    T2 -->|subscription| S4

    S3 -->|ブラウザ操作依頼| S2
    S3 -->|publish| T3
    T3 -->|subscription| S4

    S2 -->|publish| T4
    S4 -->|LINE/Email/Slack| 通知送信
```

## 4. 技術スタック

| レイヤー | 技術 | バージョン | 選定理由 |
|---|---|---|---|
| フロントエンド | Next.js (React) | ^16.2.1 | SSR対応、React Server Components、Cloud Runとの親和性 |
| スタイリング | CSS Modules | - | Next.js標準サポート、スコープ付きCSS、追加依存なし |
| バックエンドAPI | Rust / Axum | latest stable | 高性能、型安全、Towerエコシステムとの親和性 |
| ブラウザ自動操作 | Node.js / Playwright | 22.x / latest | Playwrightの公式サポート言語。証券会社アダプターのマイクロサービスとして分離 |
| データストア | Firebase Firestore | - | 無料枠活用（1GiB保存、50K読取/日）、スキーマレスの柔軟性、GCPネイティブ |
| 認証 | Firebase Authentication | - | 無料枠活用、Google連携、トークン検証の容易さ |
| メッセージング | Cloud Pub/Sub | - | イベント駆動アーキテクチャの基盤。サービス間の疎結合を実現 |
| スケジューラ | Cloud Scheduler | - | マネージドcron。Pub/Subへのメッセージ発行でジョブをトリガー |
| 秘密情報管理 | GCP Secret Manager | - | 証券口座認証情報の暗号化管理 |
| インフラ | GCP Cloud Run | - | コンテナベースのサーバーレス。on-demandでコスト最適化 |
| IaC | Terraform | latest stable | GCPリソースのコード管理、再現性の確保 |
| CI/CD | GitHub Actions | - | リポジトリとの統合性 |

### 4.1 Rust依存クレート（主要）

| クレート | 用途 |
|---|---|
| axum | Webフレームワーク |
| tokio | 非同期ランタイム |
| serde / serde_json | シリアライズ/デシリアライズ |
| reqwest | HTTPクライアント（IPO情報スクレイピング、サービス間通信） |
| scraper | HTMLパーサー（IPO情報スクレイピング） |
| google-cloud-firestore または firestore-rs | Firestoreクライアント（設計時に評価して選定） |
| google-cloud-pubsub | Pub/Subクライアント |
| google-cloud-secretmanager | Secret Managerクライアント |
| tracing | 構造化ログ |
| anyhow / thiserror | エラーハンドリング |

## 5. 機能一覧

| ID | 機能名 | 概要 | 担当サービス | 関連要件 |
|---|---|---|---|---|
| SD-001 | IPO銘柄情報取得 | 外部IPO情報サイトから銘柄情報をスクレイピングし、Firestoreに保存する | ipo-info-fetcher | [REQ-001](../01-requirements/requirements-specification.md#req-001) | {#sd-001}
| SD-002 | IPO抽選自動申し込み | 楽天証券のWebサイトをPlaywrightで自動操作し、抽選申し込みを実行する | ipo-browser | [REQ-002](../01-requirements/requirements-specification.md#req-002) | {#sd-002}
| SD-003 | 抽選結果自動確認 | 楽天証券のWebサイトから当選/落選結果を自動取得する | ipo-result-checker, ipo-browser | [REQ-003](../01-requirements/requirements-specification.md#req-003) | {#sd-003}
| SD-004 | 進捗管理ダッシュボード | 全銘柄のステータス一覧、サマリー、フィルタリング | ipo-frontend, ipo-api | [REQ-004](../01-requirements/requirements-specification.md#req-004) | {#sd-004}
| SD-005 | 除外リスト管理 | 自動申し込み対象外銘柄のCRUD | ipo-frontend, ipo-api | [REQ-005](../01-requirements/requirements-specification.md#req-005) | {#sd-005}
| SD-006 | 通知送信 | アダプターパターンによるLINE/メール/Slack通知 | ipo-api | [REQ-006](../01-requirements/requirements-specification.md#req-006) | {#sd-006}
| SD-007 | 証券口座認証情報管理 | Secret Managerを利用した認証情報の暗号化保管・CRUD | ipo-api | [REQ-007](../01-requirements/requirements-specification.md#req-007) | {#sd-007}
| SD-008 | 操作ログ | 自動操作の実行履歴の記録・閲覧 | 全サービス, ipo-frontend | [REQ-008](../01-requirements/requirements-specification.md#req-008) | {#sd-008}
| SD-009 | 証券会社アダプター | 証券会社ごとのブラウザ操作を抽象化するアダプターパターン | ipo-browser | [REQ-009](../01-requirements/requirements-specification.md#req-009) | {#sd-009}
| SD-010 | 認証 | Firebase Authenticationによるログイン・トークン検証 | ipo-frontend, ipo-api | [REQ-NF-022](../01-requirements/requirements-specification.md) | {#sd-010}

## 6. 画面設計

### 6.1 画面一覧

| ID | 画面名 | 概要 | URLパターン |
|---|---|---|---|
| UI-001 | ログイン | Firebase Authenticationによるログイン | `/login` |
| UI-002 | ダッシュボード | 全IPO銘柄のステータス一覧、サマリー、フィルタリング | `/` |
| UI-003 | IPO銘柄詳細 | 銘柄の詳細情報、スケジュール、申し込み状況 | `/stocks/{stockId}` |
| UI-004 | 除外リスト管理 | 除外銘柄の一覧・追加・削除 | `/exclusions` |
| UI-005 | 通知設定 | 通知チャネル・イベントごとの有効/無効設定 | `/settings/notifications` |
| UI-006 | 証券口座管理 | 証券会社の認証情報登録・更新・削除・接続テスト | `/settings/accounts` |
| UI-007 | 操作ログ | 自動操作の実行履歴一覧、フィルタリング | `/logs` |

### 6.2 画面遷移図

```mermaid
stateDiagram-v2
    [*] --> ログイン
    ログイン --> ダッシュボード : Firebase Auth成功

    ダッシュボード --> IPO銘柄詳細 : 銘柄選択
    IPO銘柄詳細 --> ダッシュボード : 戻る

    ダッシュボード --> 除外リスト管理 : サイドメニュー
    ダッシュボード --> 通知設定 : サイドメニュー
    ダッシュボード --> 証券口座管理 : サイドメニュー
    ダッシュボード --> 操作ログ : サイドメニュー

    除外リスト管理 --> ダッシュボード : サイドメニュー
    通知設定 --> ダッシュボード : サイドメニュー
    証券口座管理 --> ダッシュボード : サイドメニュー
    操作ログ --> ダッシュボード : サイドメニュー
```

### 6.3 ダッシュボード画面レイアウト概要

```
┌──────────────────────────────────────────────────┐
│ IPOtto                    [ユーザー名 ▼] │
├──────────┬───────────────────────────────────────┤
│          │  ステータスサマリー                      │
│ ダッシュ  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐     │
│ ボード   │  │申込前│ │申込済│ │ 当選│ │ 落選│      │
│          │  │  5  │ │  3  │ │  1  │ │  8  │      │
│ 除外     │  └─────┘ └─────┘ └─────┘ └─────┘     │
│ リスト   │                                        │
│          │  IPO銘柄一覧              [ステータス ▼] │
│ 通知設定  │  ┌─────────────────────────────────┐  │
│          │  │ ○○(株)  楽天  申込済  03/28抽選  │  │
│ 証券口座  │  │ △△(株)  楽天  申込前  04/02締切  │  │
│          │  │ □□(株)  楽天  当選    04/05上場  │  │
│ 操作ログ  │  │ ◇◇(株)  楽天  落選    03/20抽選  │  │
│          │  └─────────────────────────────────┘  │
└──────────┴───────────────────────────────────────┘
```

## 7. 外部インターフェース

### 7.1 外部システム連携一覧

| ID | 連携先 | プロトコル | 方向 | 担当サービス | 概要 |
|---|---|---|---|---|---|
| IF-001 | 楽天証券Webサイト | HTTPS (ブラウザ自動操作) | 送受信 | ipo-browser | IPO抽選申し込み、結果確認 |
| IF-002 | IPO情報サイト | HTTPS (スクレイピング) | 受信 | ipo-info-fetcher | IPO銘柄情報の取得 |
| IF-003 | Firebase Authentication | HTTPS (SDK) | 送受信 | ipo-frontend, ipo-api | ユーザー認証・トークン検証 |
| IF-004 | Firestore | HTTPS (SDK) | 送受信 | ipo-api, ipo-info-fetcher, ipo-browser, ipo-result-checker | データの読み書き |
| IF-005 | Cloud Pub/Sub | HTTPS (SDK) | 送受信 | 全サービス | イベントメッセージの発行・購読 |
| IF-006 | GCP Secret Manager | HTTPS (SDK) | 受信 | ipo-api, ipo-browser | 証券口座認証情報の取得 |
| IF-007 | LINE Notify | HTTPS (REST API) | 送信 | ipo-api | LINE通知送信 |
| IF-008 | SendGrid | HTTPS (REST API) | 送信 | ipo-api | メール通知送信 |
| IF-009 | Slack Webhook | HTTPS (Webhook) | 送信 | ipo-api | Slack通知送信 |

### 7.2 連携シーケンス（IPO自動申し込み）

```mermaid
sequenceDiagram
    participant Scheduler as Cloud Scheduler
    participant PubSub as Pub/Sub
    participant Fetcher as ipo-info-fetcher
    participant Browser as ipo-browser
    participant API as ipo-api
    participant Firestore as Firestore
    participant Securities as 楽天証券
    participant Secret as Secret Manager
    participant Notification as 通知サービス

    Note over Scheduler: 毎日定時に実行

    Scheduler->>PubSub: publish(ipo-job-trigger)

    par IPO情報取得
        PubSub->>Fetcher: message(type: fetch)
        Fetcher->>Fetcher: 外部IPO情報サイトをスクレイピング
        Fetcher->>Firestore: IPO銘柄情報を保存
        Fetcher->>PubSub: publish(ipo-info-updated)
    and 自動申し込み
        PubSub->>Browser: message(type: apply)
        Browser->>Firestore: 申し込み対象銘柄を取得
        Browser->>Firestore: 除外リストを取得
        Browser->>Browser: 除外銘柄をフィルタリング
        Browser->>Secret: 認証情報を取得
        Browser->>Securities: ログイン（Playwright）
        loop 対象銘柄ごと
            Browser->>Securities: 抽選申し込み操作
            Securities-->>Browser: 申し込み結果
            Browser->>Firestore: 操作ログ・ステータス更新
        end
        Browser->>PubSub: publish(ipo-notification)
    end

    PubSub->>API: message(ipo-notification)
    API->>Notification: 通知送信（LINE/Email/Slack）
```

### 7.3 連携シーケンス（抽選結果確認）

```mermaid
sequenceDiagram
    participant Scheduler as Cloud Scheduler
    participant PubSub as Pub/Sub
    participant Checker as ipo-result-checker
    participant Browser as ipo-browser
    participant Firestore as Firestore
    participant Securities as 楽天証券
    participant API as ipo-api
    participant Notification as 通知サービス

    Scheduler->>PubSub: publish(ipo-job-trigger)
    PubSub->>Checker: message(type: check)
    Checker->>Firestore: 結果確認対象銘柄を取得
    Checker->>Browser: 結果取得リクエスト
    Browser->>Securities: ログイン → 結果ページ確認
    Securities-->>Browser: 当選/落選/補欠
    Browser-->>Checker: 結果返却
    Checker->>Firestore: 結果・ステータス更新
    Checker->>PubSub: publish(ipo-result-updated)
    PubSub->>API: message(ipo-result-updated)
    API->>Notification: 結果通知送信
```

## 8. 概念データモデル

```mermaid
erDiagram
    IPO_STOCKS ||--o{ APPLICATIONS : "has"
    IPO_STOCKS ||--o{ EXCLUSIONS : "excluded by"
    SECURITIES_ACCOUNTS ||--o{ APPLICATIONS : "applied from"
    APPLICATIONS ||--o{ OPERATION_LOGS : "generates"
    NOTIFICATION_SETTINGS ||--o{ NOTIFICATION_CHANNELS : "has"

    IPO_STOCKS {
        string identifier PK "ドキュメントID"
        string companyName "企業名"
        string tickerSymbol "証券コード"
        string market "上場市場"
        date bookBuildingStartDate "BB開始日"
        date bookBuildingEndDate "BB終了日"
        date lotteryDate "抽選日"
        date listingDate "上場日"
        int offerPrice "公募価格"
        int priceRangeMin "仮条件下限"
        int priceRangeMax "仮条件上限"
        string leadUnderwriter "主幹事証券"
        string status "ステータス"
        string fetchOrigin "情報取得元"
        timestamp fetchedAt "情報取得日時"
    }
    APPLICATIONS {
        string identifier PK "ドキュメントID"
        string stock FK "IPO銘柄ID"
        string securitiesAccount FK "証券口座ID"
        string status "申し込みステータス"
        int appliedShares "申し込み株数"
        int appliedPrice "申し込み価格"
        timestamp appliedAt "申し込み日時"
        string lotteryResult "抽選結果"
        timestamp resultConfirmedAt "結果確認日時"
    }
    EXCLUSIONS {
        string identifier PK "ドキュメントID"
        string companyName "企業名"
        string reason "除外理由"
        timestamp createdAt "作成日時"
    }
    SECURITIES_ACCOUNTS {
        string identifier PK "ドキュメントID"
        string securitiesCompany "証券会社名"
        string loginId "ログインID（暗号化）"
        string loginPassword "ログインパスワード（暗号化）"
        string tradingPassword "取引暗証番号（暗号化）"
        boolean isActive "有効フラグ"
        timestamp lastTestedAt "最終接続テスト日時"
        string lastTestResult "最終テスト結果"
    }
    OPERATION_LOGS {
        string identifier PK "ドキュメントID"
        string application FK "申し込みID（任意）"
        string eventType "イベント種別"
        string serviceName "実行サービス名"
        string status "成功/失敗"
        string details "詳細メッセージ"
        string errorMessage "エラーメッセージ（任意）"
        timestamp executedAt "実行日時"
    }
    NOTIFICATION_SETTINGS {
        string identifier PK "ドキュメントID"
        boolean enabled "通知全体の有効/無効"
    }
    NOTIFICATION_CHANNELS {
        string identifier PK "ドキュメントID"
        string channelType "LINE/Email/Slack"
        string configuration "チャネル固有設定（JSON）"
        boolean enabled "有効/無効"
        map eventSubscriptions "イベントごとの購読設定"
    }
```

### 8.1 IPO銘柄ステータス遷移

```mermaid
stateDiagram-v2
    [*] --> 情報取得済み : IPO情報サイトから取得
    情報取得済み --> 申し込み対象 : BB期間開始 & 除外リスト外
    情報取得済み --> 除外 : 除外リストに該当
    申し込み対象 --> 申し込み済み : 自動申し込み成功
    申し込み対象 --> 申し込み失敗 : 自動申し込みエラー
    申し込み失敗 --> 申し込み対象 : リトライ
    申し込み済み --> 当選 : 抽選結果
    申し込み済み --> 落選 : 抽選結果
    申し込み済み --> 補欠当選 : 抽選結果
    補欠当選 --> 当選 : 繰上当選
    補欠当選 --> 落選 : 繰上なし
    当選 --> 購入済み : 購入意思表示（Phase 2）
    当選 --> 辞退 : 購入辞退（Phase 2）
    購入済み --> 売却済み : 初値売却（Phase 2）
    落選 --> [*]
    辞退 --> [*]
    売却済み --> [*]
    除外 --> [*]
```

## 9. 共通処理方針

### 9.1 認証・認可

- Firebase Authenticationを使用し、Googleアカウントでのログインを提供する
- ipo-frontendでFirebase SDK経由でIDトークンを取得し、ipo-apiへのリクエストにBearerトークンとして付与する
- ipo-apiはFirebase Admin SDKでIDトークンを検証する
- 単一ユーザーのため、許可するユーザーのUID/メールアドレスを環境変数で設定し、それ以外のアクセスを拒否する

### 9.2 エラーハンドリング

- ipo-apiのエラーレスポンスは共通JSON形式で返却する:
  ```json
  {
    "error": {
      "code": "STOCK_NOT_FOUND",
      "message": "指定されたIPO銘柄が見つかりません"
    }
  }
  ```
- ブラウザ自動操作の失敗は、リトライ（最大3回、指数バックオフ）後にエラー通知を送信する
- 証券会社サイトのUI変更検知時は即座にアラート通知を送信し、以降のジョブ実行を停止する

### 9.3 ログ

- 全サービスで構造化ログ（JSON形式）を採用する
- Cloud Loggingに集約し、ログレベルはERROR / WARN / INFO / DEBUGを使い分ける
- 操作ログ（ユーザー向け）はFirestoreの`operation_logs`コレクションに永続化し、Web UIから閲覧可能とする
- リクエストIDを全ログに含め、サービス間のトレーサビリティを確保する

### 9.4 バリデーション

- ipo-apiのリクエストバリデーションはAxumのExtractorで実施する
- ipo-frontendでもUX向上のためバリデーションを実施するが、ipo-api側を正とする
- 証券口座の認証情報登録時は、接続テスト（実際のログイン試行）による検証を必須とする

### 9.5 通知アダプター

```mermaid
classDiagram
    class NotificationPort {
        <<trait>>
        +send(event: NotificationEvent) Result
    }
    class LineNotificationAdapter {
        +send(event: NotificationEvent) Result
    }
    class EmailNotificationAdapter {
        +send(event: NotificationEvent) Result
    }
    class SlackNotificationAdapter {
        +send(event: NotificationEvent) Result
    }

    NotificationPort <|.. LineNotificationAdapter
    NotificationPort <|.. EmailNotificationAdapter
    NotificationPort <|.. SlackNotificationAdapter
```

- 通知チャネルはPort/Adapterパターンで実装し、`NotificationPort` traitを各チャネルが実装する
- 新しい通知チャネルの追加は、trait実装の追加のみで対応可能とする

### 9.6 証券会社アダプター

```mermaid
classDiagram
    class SecuritiesBrokerPort {
        <<interface>>
        +login(credentials) Promise~Session~
        +applyForIPO(session, stock) Promise~ApplicationResult~
        +checkLotteryResult(session, stock) Promise~LotteryResult~
        +logout(session) Promise~void~
    }
    class RakutenSecuritiesAdapter {
        +login(credentials) Promise~Session~
        +applyForIPO(session, stock) Promise~ApplicationResult~
        +checkLotteryResult(session, stock) Promise~LotteryResult~
        +logout(session) Promise~void~
    }
    class SBISecuritiesAdapter {
        -未実装（Phase 2）
    }

    SecuritiesBrokerPort <|.. RakutenSecuritiesAdapter
    SecuritiesBrokerPort <|.. SBISecuritiesAdapter
```

- 証券会社の操作はインターフェースで抽象化し、証券会社ごとのPlaywright実装を差し替え可能にする
- ブラウザ操作のセレクタ（CSS/XPath）は外部設定ファイル（JSON）で管理し、UI変更時にコード修正なしで対応可能とする

## 10. 非機能設計方針

### 10.1 性能

- ipo-frontendはNext.jsのSSRでダッシュボードを事前レンダリングし、初期表示を高速化する
- FirestoreのクエリはインデックスをIPO銘柄のステータス・日付フィールドに設定し、一覧取得を最適化する
- ブラウザ自動操作は1銘柄あたり3分以内を目標とし、タイムアウトを設定する

### 10.2 スケーラビリティ

- 個人利用のため大規模なスケーラビリティは不要
- Cloud Runのon-demandインスタンスでコールドスタートを許容する（ジョブ実行は数分単位のため影響軽微）
- Firestoreの無料枠（50K読取/日）は個人利用では十分だが、使用量をモニタリングする

### 10.3 可用性

- スケジュールジョブの実行成功率99%以上を目標とする
- Cloud Runのヘルスチェックを設定し、異常時は自動再起動する
- ジョブ失敗時はPub/Subのリトライポリシー（指数バックオフ、最大3回）で自動リトライする
- 全ジョブ失敗後は通知を送信し、手動介入を促す

### 10.4 コスト最適化

- Firestoreの無料枠: 1GiB保存、50K読取/日、20K書込/日
- Firebase Authenticationの無料枠: 10K認証/月
- Cloud Runのon-demandモードで、ジョブ非実行時のコストをゼロに抑える
- Cloud Pub/Subの無料枠: 10GiBメッセージング/月
- Cloud Schedulerの無料枠: 3ジョブ/月

## 変更履歴

| バージョン | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 1.0.0 | 2026-03-25 | lihs | 初版作成 |
