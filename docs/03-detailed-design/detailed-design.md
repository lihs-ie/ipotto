---
title: "詳細設計書"
version: "1.0.0"
status: "draft"
created: "2026-03-25"
last_updated: "2026-03-25"
author: "lihs"
---

# 詳細設計書

## 1. はじめに

### 1.1 目的

本文書は、IPOttoの横断的な技術設計を定義する。5マイクロサービスのモジュール構成、サービス間のクラス設計、主要処理のシーケンス設計、エラー処理設計、および状態管理を記述する。

### 1.2 対応する基本設計項目

> [基本設計書](../02-system-design/system-design.md) — システム構成、技術スタック、機能一覧

> **ID体系**: 本文書は DD-400〜DD-499 の範囲を使用する。

## 2. クラス設計

### 2.1 サービス間の依存関係

```mermaid
graph TD
    subgraph Client["クライアント"]
        Browser["Webブラウザ"]
    end

    subgraph CloudRun["Cloud Run"]
        Frontend["⑤ ipo-frontend<br>(Next.js)"]
        API["① ipo-api<br>(Rust/Axum)"]
        BrowserSvc["② ipo-browser<br>(Node.js/Playwright)"]
        Fetcher["③ ipo-info-fetcher<br>(Rust)"]
        Checker["④ ipo-result-checker<br>(Rust)"]
    end

    subgraph GCP["GCPサービス"]
        PubSub["Pub/Sub"]
        Scheduler["Cloud Scheduler"]
        Firestore[(Firestore)]
        SecretMgr["Secret Manager"]
        FireAuth["Firebase Auth"]
    end

    Browser --> Frontend
    Frontend -->|REST API| API
    Browser -->|認証| FireAuth

    Scheduler -->|cron| PubSub
    PubSub -->|ipo-info-fetch| Fetcher
    PubSub -->|ipo-apply| BrowserSvc
    PubSub -->|ipo-result-check| Checker
    PubSub -->|ipo-notification| API

    API --> Firestore
    API --> SecretMgr
    Fetcher --> Firestore
    Fetcher -->|publish| PubSub
    BrowserSvc --> Firestore
    BrowserSvc --> SecretMgr
    BrowserSvc -->|publish| PubSub
    Checker --> Firestore
    Checker -->|HTTP| BrowserSvc
    Checker -->|publish| PubSub
```

### 2.2 各サービスのモジュール構成

#### ① ipo-api（Rust/Axum）

```mermaid
classDiagram
    namespace Handler {
        class StockHandler {
            +list_stocks(Query) Response
            +get_stock(Path) Response
        }
        class ExclusionHandler {
            +list_exclusions() Response
            +register_exclusion(Json) Response
            +remove_exclusion(Path) Response
        }
        class AccountHandler {
            +list_accounts() Response
            +register_account(Json) Response
            +update_account(Path, Json) Response
            +delete_account(Path) Response
            +test_connection(Path) Response
        }
        class NotificationHandler {
            +get_setting() Response
            +update_setting(Json) Response
        }
        class LogHandler {
            +list_logs(Query) Response
        }
        class DashboardHandler {
            +get_summary() Response
        }
    }

    namespace UseCase {
        class RegisterExclusionUseCase {
            +execute(input) Result~Output~
        }
        class ListIpoStocksUseCase {
            +execute(input) Result~Output~
        }
        class RegisterSecuritiesAccountUseCase {
            +execute(input) Result~Output~
        }
    }

    namespace DomainPort {
        class IpoStockRepository {
            <<trait>>
        }
        class ExclusionRepository {
            <<trait>>
        }
        class NotificationPort {
            <<trait>>
        }
        class EventPublisherPort {
            <<trait>>
        }
    }

    namespace Infrastructure {
        class FirestoreIpoStockRepository
        class FirestoreExclusionRepository
        class PubSubEventPublisher
        class LineNotificationAdapter
        class SlackNotificationAdapter
    }

    StockHandler --> ListIpoStocksUseCase
    ExclusionHandler --> RegisterExclusionUseCase
    AccountHandler --> RegisterSecuritiesAccountUseCase
    RegisterExclusionUseCase --> ExclusionRepository
    ListIpoStocksUseCase --> IpoStockRepository
    FirestoreIpoStockRepository ..|> IpoStockRepository
    FirestoreExclusionRepository ..|> ExclusionRepository
    PubSubEventPublisher ..|> EventPublisherPort
    LineNotificationAdapter ..|> NotificationPort
    SlackNotificationAdapter ..|> NotificationPort
```

#### ② ipo-browser（Node.js/Playwright）

```mermaid
classDiagram
    namespace Handler {
        class PubSubMessageHandler {
            +handleApplyMessage(message) void
            +handleResultCheckMessage(message) void
            +handleConnectionTestMessage(message) void
        }
    }

    namespace Adapter {
        class RakutenBrokerAdapter {
            +login(credential, mailReader) BrokerSession
            +applyForIpo(session, stock) ApplicationResult
            +checkLotteryResult(session, stock) LotteryOutcome
            +testConnection(credential, mailReader) ConnectionTestResult
            +logout(session) void
        }
    }

    namespace PageObject {
        class LoginPage {
            +enterLoginId(loginId) void
            +enterPassword(password) void
            +clickSubmit() void
            +isLoginSuccessful() boolean
            +isImageAuthenticationRequired() boolean
        }
        class ImageAuthenticationPage {
            +getImageButtons() ImageButton[]
            +clickImageByAltText(altText) void
            +isAuthenticationSuccessful() boolean
            +getErrorMessage() string?
        }
        class IpoApplicationPage {
            +enterShares(shares) void
            +enterPrice(price) void
            +enterTradingPassword(password) void
            +clickConfirm() void
            +clickSubmit() void
        }
        class IpoResultPage {
            +navigate() void
            +getResults() RawLotteryResultEntry[]
        }
    }

    namespace MailReader {
        class MailReaderPort {
            <<interface>>
            +fetchImageAuthenticationKeywords(mailCredential, receivedAfter, timeout) Promise~ImageAuthenticationKeyword~
        }
    }

    namespace SessionManagement {
        class BrowserSessionStorage {
            +getUserDataDirectory(account) string
            +sessionExists(account) boolean
            +cleanupExpiredSessions(maxAgeHours) number
        }
    }

    namespace Selector {
        class SelectorResolver {
            +resolve(page, elementName) ElementHandle
            +resolveAll(page, elementName) ElementHandle[]
        }
        class SelectorConfig {
            +loadFromYaml(path) SelectorConfig
        }
    }

    PubSubMessageHandler --> RakutenBrokerAdapter
    RakutenBrokerAdapter --> LoginPage
    RakutenBrokerAdapter --> ImageAuthenticationPage
    RakutenBrokerAdapter --> IpoApplicationPage
    RakutenBrokerAdapter --> IpoResultPage
    RakutenBrokerAdapter --> MailReaderPort
    RakutenBrokerAdapter --> BrowserSessionStorage
    LoginPage --> SelectorResolver
    ImageAuthenticationPage --> SelectorResolver
    IpoApplicationPage --> SelectorResolver
    IpoResultPage --> SelectorResolver
    SelectorResolver --> SelectorConfig
```

#### ③ ipo-info-fetcher（Rust）

```mermaid
classDiagram
    namespace Handler {
        class PubSubHandler {
            +handle_fetch_message(message) Result~()~
        }
    }

    namespace UseCase {
        class FetchIpoStocksUseCase {
            +execute() Result~FetchIpoStocksOutput~
        }
    }

    namespace Scraper {
        class FallbackScraperAdapter {
            +scrape() Result~Vec~ScrapedStock~~
        }
        class ExternalSiteScraperAdapter {
            +scrape() Result~Vec~ScrapedStock~~
        }
        class SecuritiesSiteScraperAdapter {
            +scrape() Result~Vec~ScrapedStock~~
        }
    }

    namespace Parser {
        class HtmlParser {
            +parse(html) Result~Vec~RawScrapedEntry~~
        }
        class StockTranslator {
            +translate(raw) Result~ScrapedStock~
        }
    }

    PubSubHandler --> FetchIpoStocksUseCase
    FetchIpoStocksUseCase --> FallbackScraperAdapter
    FallbackScraperAdapter --> ExternalSiteScraperAdapter
    FallbackScraperAdapter --> SecuritiesSiteScraperAdapter
    ExternalSiteScraperAdapter --> HtmlParser
    HtmlParser --> StockTranslator
```

#### ④ ipo-result-checker（Rust）

```mermaid
classDiagram
    namespace Handler {
        class PubSubHandler {
            +handle_check_message(message) Result~()~
        }
    }

    namespace UseCase {
        class CheckLotteryResultUseCase {
            +execute(input) Result~CheckLotteryResultOutput~
        }
    }

    namespace Infrastructure {
        class BrowserServiceClient {
            +check_lottery_result(credential, stock) Result~LotteryOutcome~
        }
        class FirestoreApplicationRepository
    }

    PubSubHandler --> CheckLotteryResultUseCase
    CheckLotteryResultUseCase --> BrowserServiceClient
    CheckLotteryResultUseCase --> FirestoreApplicationRepository
```

#### ⑤ ipo-frontend（Next.js）

```mermaid
classDiagram
    namespace AppRouter {
        class DashboardPage {
            +page() JSX
        }
        class StockListPage {
            +page() JSX
        }
        class StockDetailPage {
            +page(params) JSX
        }
        class SettingsLayout {
            +layout(children) JSX
        }
    }

    namespace Components {
        class StatusSummaryCard
        class IpoStockTable
        class ExclusionListPanel
        class NotificationSettingForm
        class AccountRegistrationForm
        class OperationLogTable
    }

    namespace Hooks {
        class useIpoStocks {
            +stocks: IpoStock[]
            +isLoading: boolean
            +refetch() void
        }
        class useDashboard {
            +summary: DashboardSummary
            +isLoading: boolean
        }
    }

    namespace Lib {
        class ApiClient {
            +get(path) Promise~T~
            +post(path, body) Promise~T~
            +put(path, body) Promise~T~
            +delete(path) Promise~void~
        }
        class FirebaseAuthProvider {
            +getIdToken() Promise~string~
            +onAuthStateChanged(callback) void
        }
    }

    DashboardPage --> useDashboard
    StockListPage --> useIpoStocks
    useDashboard --> ApiClient
    useIpoStocks --> ApiClient
    ApiClient --> FirebaseAuthProvider
```

### 2.3 各クラスの責務

| クラス/モジュール | サービス | レイヤー | 責務 |
|---|---|---|---|
| StockHandler | ipo-api | ハンドラー | HTTPリクエストの受付、入力バリデーション、レスポンス返却 |
| RegisterExclusionUseCase | ipo-api | ユースケース | 除外銘柄登録のビジネスフロー実行 |
| ApplicationEligibilityService | ipo-api | ドメインサービス | 銘柄の申し込み適格性判定 |
| IpoStockRepository (trait) | ipo-api | ドメインポート | IPO銘柄の永続化インターフェース |
| FirestoreIpoStockRepository | ipo-api | インフラ | Firestoreへの具体的な読み書き |
| PubSubEventPublisher | ipo-api | インフラ | Pub/Subへのイベント発行 |
| RakutenBrokerAdapter | ipo-browser | アダプター | 楽天証券固有のPlaywright操作を実装 |
| LoginPage | ipo-browser | Page Object | ログインページの要素操作をカプセル化 |
| SelectorResolver | ipo-browser | セレクタ | YAML設定からセレクタを解決、フォールバック実行 |
| FallbackScraperAdapter | ipo-info-fetcher | スクレイパー | メイン→フォールバックの切り替え制御 |
| HtmlParser | ipo-info-fetcher | パーサー | HTML文字列から構造化データを抽出 |
| StockTranslator | ipo-info-fetcher | パーサー | 生データ→ドメイン値オブジェクトへの変換 |
| ApiClient | ipo-frontend | Lib | ipo-apiへのHTTPリクエスト、トークン付与 |
| FirebaseAuthProvider | ipo-frontend | Lib | Firebase Auth SDK のラッパー |

## 3. シーケンス設計

### 3.1 DD-400: 自動申し込みフルフロー

```mermaid
sequenceDiagram
    participant Scheduler as Cloud Scheduler
    participant PubSub as Pub/Sub
    participant API as ipo-api
    participant Fetcher as ipo-info-fetcher
    participant Browser as ipo-browser
    participant Firestore as Firestore
    participant SecretMgr as Secret Manager
    participant Rakuten as 楽天証券
    participant MailSvc as メールサービス
    participant Notif as 通知サービス

    Note over Scheduler: 定時トリガー（例: 毎日9:00）

    Scheduler->>PubSub: publish(ipo-job-trigger)

    par IPO情報取得
        PubSub->>Fetcher: message(type: fetch)
        Fetcher->>Fetcher: 外部サイトスクレイピング
        alt スクレイピング成功
            Fetcher->>Firestore: IPO銘柄情報を保存/更新
            Fetcher->>PubSub: publish(ipo-info-updated)
        else スクレイピング失敗
            Fetcher->>Fetcher: フォールバック先へ切り替え
            Fetcher->>Firestore: 操作ログ記録
        end
    and 自動申し込み
        PubSub->>Browser: message(type: apply)
        Browser->>Firestore: BB期間中の銘柄取得
        Browser->>Firestore: 除外リスト取得
        Browser->>Firestore: 有効な証券口座取得
        Browser->>Browser: 適格性判定（除外チェック、重複チェック）

        loop 各証券口座
            Browser->>SecretMgr: 認証情報取得
            SecretMgr-->>Browser: AccountCredential

            Note over Browser, Rakuten: 第1防御線: セッション永続化
            Browser->>Rakuten: persistentContext起動（セッション確認）
            alt セッション有効（ダッシュボード表示）
                Rakuten-->>Browser: ログイン済み
            else セッション切れ（ログインページに遷移）
                Browser->>Rakuten: Playwright: ID/PW入力 → ログイン
                alt 画像認証が要求された
                    Note over Browser, MailSvc: 第2防御線: メールOTP自動突破
                    Rakuten-->>Browser: 画像認証画面（10個のimg[alt]ボタン）
                    Browser->>Browser: 画像ボタンのalt属性を収集
                    Browser->>MailSvc: OTPキーワード取得（IMAP/Gmail API、ポーリング最大120秒）
                    MailSvc-->>Browser: ImageAuthenticationKeyword{first, second}
                    Browser->>Rakuten: alt一致ボタンを順番にクリック
                    alt 認証成功
                        Rakuten-->>Browser: ログイン成功（セッション自動永続化）
                    else 認証失敗
                        Note over Browser: 第3防御線: 手動フォールバック
                        Browser->>PubSub: publish(ipo-notification, ImageAuthenticationFailed)
                        Browser->>Firestore: 操作ログ記録（認証失敗）
                        Note over Browser: ⚠ 口座ロック防止のため再試行しない。該当口座をスキップ
                    end
                else 画像認証なし
                    Rakuten-->>Browser: ログイン成功
                end
            end

            loop 対象銘柄ごと
                Browser->>Rakuten: Playwright: IPO申し込み操作
                alt 申し込み成功
                    Rakuten-->>Browser: 「受け付けました」
                    Browser->>Browser: ACL変換: ApplicationResult::Success
                    Browser->>Firestore: LotteryApplication保存
                    Browser->>Firestore: 操作ログ記録（成功）
                    Browser->>PubSub: publish(ipo-notification, ApplicationCompleted)
                else 申し込み失敗
                    Rakuten-->>Browser: エラーメッセージ
                    Browser->>Browser: ACL変換: ApplicationResult::Failure
                    Browser->>Firestore: 操作ログ記録（失敗）
                    Browser->>PubSub: publish(ipo-notification, ApplicationFailed)
                end
            end

            Browser->>Rakuten: Playwright: ログアウト
        end
    end

    PubSub->>API: message(ipo-notification)
    API->>Firestore: 通知設定取得
    API->>API: 有効チャネル・購読イベント判定
    API->>Notif: 通知送信（LINE/Email/Slack）
```

### 3.2 DD-401: 証券口座登録 + 接続テスト

```mermaid
sequenceDiagram
    actor User as 投資家
    participant Frontend as ipo-frontend
    participant FireAuth as Firebase Auth
    participant API as ipo-api
    participant Firestore as Firestore
    participant SecretMgr as Secret Manager
    participant Browser as ipo-browser
    participant Rakuten as 楽天証券

    User->>Frontend: 口座登録フォーム入力
    Frontend->>FireAuth: getIdToken()
    FireAuth-->>Frontend: IDトークン

    Frontend->>API: POST /accounts (Bearer Token)
    API->>API: IDトークン検証（middleware）
    API->>API: 入力バリデーション

    API->>SecretMgr: 機密情報保存（loginId, loginPassword, tradingPassword）
    SecretMgr-->>API: secretKey

    API->>Firestore: SecuritiesAccountドキュメント保存（credentialSecretKey含む）
    Firestore-->>API: 保存成功

    API-->>Frontend: 201 Created { identifier }
    Frontend-->>User: 「口座を登録しました」

    User->>Frontend: 「接続テスト」ボタン押下
    Frontend->>API: POST /accounts/{identifier}/test (Bearer Token)

    API->>Firestore: SecuritiesAccount取得
    API->>SecretMgr: 機密情報取得
    SecretMgr-->>API: AccountCredential

    API->>Browser: POST /test-connection { credential }
    Browser->>Rakuten: Playwright: ログイン試行
    alt ログイン成功
        Rakuten-->>Browser: ログイン成功
        Browser->>Rakuten: Playwright: ログアウト
        Browser-->>API: { success: true, message: "接続成功" }
    else ログイン失敗
        Rakuten-->>Browser: エラーページ
        Browser-->>API: { success: false, message: "ログインIDまたはパスワードが不正です" }
    end

    API->>Firestore: connectionTest結果を更新
    API-->>Frontend: { success, message, testedAt }
    Frontend-->>User: テスト結果表示
```

### 3.3 DD-402: ダッシュボード表示

```mermaid
sequenceDiagram
    actor User as 投資家
    participant Frontend as ipo-frontend
    participant FireAuth as Firebase Auth
    participant API as ipo-api
    participant Firestore as Firestore

    User->>Frontend: ダッシュボードにアクセス
    Frontend->>FireAuth: onAuthStateChanged()
    FireAuth-->>Frontend: User（認証済み）
    Frontend->>FireAuth: getIdToken()
    FireAuth-->>Frontend: IDトークン

    par サマリー取得
        Frontend->>API: GET /dashboard (Bearer Token)
        API->>API: IDトークン検証
        API->>Firestore: 全IPO銘柄取得
        API->>Firestore: 直近の申し込み取得（limit: 5）
        API->>API: ステータス別集計
        API-->>Frontend: DashboardSummaryOutput
    and 銘柄一覧取得
        Frontend->>API: GET /stocks (Bearer Token)
        API->>Firestore: IPO銘柄一覧取得
        API-->>Frontend: ListIpoStocksOutput
    end

    Frontend->>Frontend: React: SSR → ダッシュボード描画
    Frontend-->>User: ダッシュボード表示
```

### 3.4 DD-403: エラー時の通知フロー

```mermaid
sequenceDiagram
    participant Browser as ipo-browser
    participant Rakuten as 楽天証券
    participant Firestore as Firestore
    participant PubSub as Pub/Sub
    participant API as ipo-api
    participant Setting as NotificationSetting
    participant LINE as LINE Notify
    participant Slack as Slack Webhook

    Browser->>Rakuten: Playwright: ログイン試行
    Rakuten-->>Browser: タイムアウト

    Browser->>Browser: リトライ（1回目、1秒後）
    Browser->>Rakuten: Playwright: ログイン試行
    Rakuten-->>Browser: タイムアウト

    Browser->>Browser: リトライ（2回目、4秒後）
    Browser->>Rakuten: Playwright: ログイン試行
    Rakuten-->>Browser: タイムアウト

    Browser->>Browser: リトライ（3回目、16秒後）
    Browser->>Rakuten: Playwright: ログイン試行
    Rakuten-->>Browser: タイムアウト

    Note over Browser: 最大リトライ回数超過

    Browser->>Browser: スクリーンショット取得
    Browser->>Firestore: 操作ログ記録（エラー詳細 + スクリーンショットパス）
    Browser->>PubSub: publish(ipo-notification, OperationErrorOccurred)

    PubSub->>API: message(OperationErrorOccurred)
    API->>Firestore: 通知設定取得
    Firestore-->>API: NotificationSetting

    API->>Setting: findActiveChannelsForEvent(OperationError)
    Setting-->>API: [LINE, Slack]

    par LINE通知
        API->>LINE: POST /api/notify「[エラー] 楽天証券へのログインに失敗しました（3回リトライ後）」
        LINE-->>API: 200 OK
    and Slack通知
        API->>Slack: POST webhook「🚨 操作エラー」+ Blockテンプレート
        Slack-->>API: 200 OK
    end

    API->>Firestore: 通知送信ログ記録
```

## 4. 処理フロー

### 4.1 申し込み適格性判定フロー

```mermaid
flowchart TD
    Start([銘柄の適格性判定開始]) --> CheckBB{BB期間中か?}
    CheckBB -->|No| Skip1[スキップ: BB期間外]
    CheckBB -->|Yes| CheckExclusion{除外リストに該当?}
    CheckExclusion -->|Yes| Skip2[スキップ: 除外銘柄]
    CheckExclusion -->|No| CheckDuplicate{同一口座で申し込み済み?}
    CheckDuplicate -->|Yes| Skip3[スキップ: 申し込み済み]
    CheckDuplicate -->|No| CheckAccount{有効な証券口座あり?}
    CheckAccount -->|No| Skip4[スキップ: 有効口座なし]
    CheckAccount -->|Yes| Eligible([対象: 申し込み実行])

    Skip1 --> End([判定終了])
    Skip2 --> End
    Skip3 --> End
    Skip4 --> End
    Eligible --> End
```

## 5. データ構造

### 5.1 主要な型定義（Rust）

```rust
// ── エラー型 ──

// ドメイン層エラー（thiserror）
#[derive(Debug, thiserror::Error)]
enum DomainError {
    #[error("無効なステータス遷移: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },
    #[error("重複申し込み: stock={stock}, account={account}")]
    DuplicateApplication { stock: String, account: String },
    #[error("無効な企業名: {reason}")]
    InvalidCompanyName { reason: String },
    #[error("機密情報が不完全です")]
    IncompleteCredential,
    #[error("有効な通知チャネルがありません")]
    NoActiveChannel,
    #[error("通知チャネルタイプが重複しています: {channel_type}")]
    DuplicateChannelType { channel_type: String },
    #[error("無効なスケジュール: {reason}")]
    InvalidSchedule { reason: String },
    #[error("無効な仮条件: {reason}")]
    InvalidPriceRange { reason: String },
}

// インフラ層エラー（thiserror）
#[derive(Debug, thiserror::Error)]
enum InfrastructureError {
    #[error("Firestoreエラー: {message}")]
    Firestore { message: String },
    #[error("Secret Managerエラー: {message}")]
    SecretManager { message: String },
    #[error("Pub/Subエラー: {message}")]
    PubSub { message: String },
    #[error("ブラウザ操作エラー: {message}")]
    BrowserOperation { message: String },
    #[error("スクレイピングエラー: {message}")]
    Scraping { message: String },
    #[error("通知送信エラー: channel={channel_type}, message={message}")]
    NotificationSend { channel_type: String, message: String },
}

// APIレスポンスエラー
struct ApiErrorResponse {
    error: ApiErrorBody,
}

struct ApiErrorBody {
    code: String,       // "INVALID_STATUS_TRANSITION"
    message: String,    // ユーザー向けメッセージ
}
```

### 5.2 主要な型定義（TypeScript / ipo-browser）

```typescript
// ── ブラウザ操作の型 ──

interface BrokerOperationPort {
  login(credential: AccountCredential, mailReader: MailReaderPort): Promise<BrokerSession>;
  applyForIpo(session: BrokerSession, stock: IpoStockReference): Promise<ApplicationResult>;
  checkLotteryResult(session: BrokerSession, stock: IpoStockReference): Promise<LotteryOutcomeResult>;
  testConnection(credential: AccountCredential, mailReader: MailReaderPort): Promise<ConnectionTestResult>;
  logout(session: BrokerSession): Promise<void>;
}

interface AccountCredential {
  readonly loginId: string;
  readonly loginPassword: string;
  readonly tradingPassword: string;
  readonly mailCredential: MailCredential;
}

interface MailCredential {
  readonly mailAddress: string;
  readonly mailPassword: string;
  readonly imapHost: string;
  readonly imapPort: number;
}

interface BrokerSession {
  readonly identifier: string;
  readonly loggedInAt: string;
  readonly userDataDirectory: string;
}

// ── 画像認証関連の型 ──

interface ImageAuthenticationKeyword {
  readonly firstKeyword: string;
  readonly secondKeyword: string;
}

interface ImageButton {
  readonly altText: string;
  readonly elementHandle: ElementHandle;
}

interface MailReaderPort {
  fetchImageAuthenticationKeywords(
    mailCredential: MailCredential,
    receivedAfter: Date,
    timeoutSeconds: number
  ): Promise<ImageAuthenticationKeyword>;
}

interface BrowserSessionStorage {
  getUserDataDirectory(account: string): string;
  sessionExists(account: string): boolean;
  cleanupExpiredSessions(maxAgeHours: number): Promise<number>;
}

type ApplicationResult =
  | { status: "success" }
  | { status: "failure"; reason: string }
  | { status: "already_applied" }
  | { status: "insufficient_balance" };

type LotteryOutcomeResult =
  | { status: "won" }
  | { status: "lost" }
  | { status: "alternate" }
  | { status: "not_yet_available" };

// ── セレクタ設定の型 ──

interface SelectorConfig {
  version: string;
  lastVerified: string;
  [pageName: string]: PageSelectors | string;
}

interface PageSelectors {
  url?: string;
  [elementName: string]: SelectorEntry[] | string | undefined;
}

interface SelectorEntry {
  selector: string;
  strategy: "css" | "text" | "role" | "xpath";
  description: string;
}
```

## 6. エラー処理設計

### 6.1 エラー分類

| 分類 | 説明 | HTTPステータス | ログレベル | 発生源 |
|---|---|---|---|---|
| バリデーションエラー | 入力値の不正 | 400 | WARN | ハンドラー層 |
| ドメインルール違反 | ビジネスルールへの違反 | 409 / 422 | WARN | ドメイン層 |
| リソース未検出 | 対象が存在しない | 404 | INFO | ユースケース層 |
| 認証エラー | IDトークンの不正・期限切れ | 401 | WARN | ミドルウェア |
| 認可エラー | 許可されていないユーザー | 403 | WARN | ミドルウェア |
| 外部サービスエラー | ブラウザ操作・スクレイピング・通知の失敗 | 503 | ERROR | インフラ層 |
| システムエラー | 予期しないエラー | 500 | ERROR | 全層 |

### 6.2 エラーハンドリング方針

#### レイヤー別の責務

```mermaid
graph TD
    subgraph Handler["ハンドラー層"]
        H1["入力バリデーション"]
        H2["anyhow::Error → HTTPレスポンス変換"]
    end

    subgraph UseCase["ユースケース層"]
        U1["ドメインエラーの伝播（?演算子）"]
        U2["コンテキスト付加（.context()）"]
    end

    subgraph Domain["ドメイン層"]
        D1["不変条件違反 → DomainError"]
    end

    subgraph Infra["インフラ層"]
        I1["外部サービスエラー → InfrastructureError"]
        I2["リトライ・フォールバック"]
    end

    D1 -->|thiserror| U1
    I1 -->|thiserror| U1
    U1 -->|anyhow| H2
    H2 -->|axum::Response| Client

    style Handler fill:#fff3e0
    style UseCase fill:#e1f5fe
    style Domain fill:#e8f5e9
    style Infra fill:#fce4ec
```

#### エラー変換フロー（Rust擬似コード）

```rust
// ハンドラー層: エラーをHTTPレスポンスに変換
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error = self.0;

        // ドメインエラーの判定
        if let Some(domain_error) = error.downcast_ref::<DomainError>() {
            return match domain_error {
                DomainError::InvalidStatusTransition { .. } =>
                    (StatusCode::CONFLICT, json_error("INVALID_STATUS_TRANSITION", &domain_error.to_string())),
                DomainError::DuplicateApplication { .. } =>
                    (StatusCode::CONFLICT, json_error("DUPLICATE_APPLICATION", &domain_error.to_string())),
                DomainError::InvalidCompanyName { .. } =>
                    (StatusCode::BAD_REQUEST, json_error("INVALID_COMPANY_NAME", &domain_error.to_string())),
                DomainError::IncompleteCredential =>
                    (StatusCode::BAD_REQUEST, json_error("INCOMPLETE_CREDENTIAL", &domain_error.to_string())),
                DomainError::NoActiveChannel =>
                    (StatusCode::UNPROCESSABLE_ENTITY, json_error("NO_ACTIVE_CHANNEL", &domain_error.to_string())),
                _ =>
                    (StatusCode::BAD_REQUEST, json_error("DOMAIN_ERROR", &domain_error.to_string())),
            }.into_response();
        }

        // インフラエラーの判定
        if let Some(infra_error) = error.downcast_ref::<InfrastructureError>() {
            tracing::error!(error = %infra_error, "インフラストラクチャエラー");
            return (StatusCode::SERVICE_UNAVAILABLE, json_error("SERVICE_UNAVAILABLE", "外部サービスとの通信に失敗しました"))
                .into_response();
        }

        // 予期しないエラー
        tracing::error!(error = %error, "予期しないエラー");
        (StatusCode::INTERNAL_SERVER_ERROR, json_error("INTERNAL_ERROR", "内部エラーが発生しました"))
            .into_response()
    }
}
```

### 6.3 エラーコード一覧

| エラーコード | HTTPステータス | 説明 | 対処方法 |
|---|---|---|---|
| `INVALID_STATUS_TRANSITION` | 409 | 許可されていないステータス遷移 | 現在のステータスを確認してください |
| `DUPLICATE_APPLICATION` | 409 | 同一銘柄・口座で申し込み済み | 既に申し込まれています |
| `INVALID_COMPANY_NAME` | 400 | 企業名が無効 | 1〜200文字で入力してください |
| `INCOMPLETE_CREDENTIAL` | 400 | 機密情報が不完全 | 全項目を入力してください |
| `NO_ACTIVE_CHANNEL` | 422 | 有効な通知チャネルがない | 少なくとも1つのチャネルを有効にしてください |
| `DUPLICATE_CHANNEL_TYPE` | 409 | 同一チャネルタイプが重複 | 各チャネルタイプは1つまでです |
| `STOCK_NOT_FOUND` | 404 | IPO銘柄が存在しない | 銘柄IDを確認してください |
| `ACCOUNT_NOT_FOUND` | 404 | 証券口座が存在しない | 口座IDを確認してください |
| `EXCLUSION_NOT_FOUND` | 404 | 除外銘柄が存在しない | 除外IDを確認してください |
| `BROWSER_OPERATION_FAILED` | 503 | ブラウザ操作に失敗 | 証券会社サイトの状態を確認してください |
| `SCRAPING_FAILED` | 503 | スクレイピングに失敗 | IPO情報サイトの状態を確認してください |
| `IMAGE_AUTHENTICATION_FAILED` | 503 | 画像認証の自動突破に失敗 | 手動でログインし画像認証を完了してください |
| `MAIL_RETRIEVAL_FAILED` | 503 | 認証メールの取得に失敗 | メール設定を確認してください |
| `MAIL_RETRIEVAL_TIMEOUT` | 503 | 認証メールの取得がタイムアウト | メール受信の遅延が考えられます。手動で対応してください |
| `ACCOUNT_LOCKED` | 403 | 証券口座がロックされた | 証券会社に連絡して口座ロックを解除してください |
| `INVALID_MAIL_ADDRESS` | 400 | メールアドレスが無効 | 有効なメールアドレスを入力してください |
| `SESSION_EXPIRED` | — | セッションが切れた（内部エラー） | 自動的にメールOTP認証でリトライされます |
| `UNAUTHORIZED` | 401 | 認証に失敗 | 再ログインしてください |
| `FORBIDDEN` | 403 | アクセス権限がない | 許可されたユーザーではありません |
| `INTERNAL_ERROR` | 500 | 予期しないエラー | 管理者に連絡してください |

## 7. 状態管理

### 7.1 IPO銘柄ステータス遷移

```mermaid
stateDiagram-v2
    [*] --> Fetched : IPO情報サイトから取得

    Fetched --> Eligible : BB期間開始 & 除外リスト外
    Fetched --> Excluded : 除外リストに該当

    Eligible --> Applied : 自動申し込み成功
    Eligible --> Failed : 自動申し込みエラー

    Failed --> Eligible : リトライ対象として再判定

    Applied --> Won : 抽選結果: 当選
    Applied --> Lost : 抽選結果: 落選
    Applied --> Alternate : 抽選結果: 補欠当選

    Alternate --> Won : 繰上当選
    Alternate --> Lost : 繰上なし

    Won --> Purchased : 購入意思表示（Phase 2）
    Won --> Declined : 購入辞退（Phase 2）

    Purchased --> Sold : 初値売却（Phase 2）

    Lost --> [*]
    Declined --> [*]
    Sold --> [*]
    Excluded --> [*]
```

### 7.2 IPO銘柄ステータス遷移表

| 現在の状態 | イベント | 次の状態 | 条件 | 実行者 |
|---|---|---|---|---|
| — | IPO情報取得 | Fetched | 外部サイトからの情報取得成功 | ipo-info-fetcher |
| Fetched | BB期間開始 & 適格性判定 | Eligible | BB期間中かつ除外リスト外 | ipo-browser |
| Fetched | 除外リスト一致 | Excluded | 企業名が除外リストと完全一致 | ipo-browser |
| Eligible | 自動申し込み成功 | Applied | 証券会社サイトで申し込み完了 | ipo-browser |
| Eligible | 自動申し込みエラー | Failed | 申し込み操作が失敗 | ipo-browser |
| Failed | リトライ | Eligible | 次回スケジュール実行時に再判定 | ipo-browser |
| Applied | 当選 | Won | 抽選結果が「当選」 | ipo-result-checker |
| Applied | 落選 | Lost | 抽選結果が「落選」 | ipo-result-checker |
| Applied | 補欠当選 | Alternate | 抽選結果が「補欠当選」 | ipo-result-checker |
| Alternate | 繰上当選 | Won | 補欠からの繰上 | ipo-result-checker |
| Alternate | 繰上なし | Lost | 繰上期間終了 | ipo-result-checker |
| Won | 購入意思表示 | Purchased | Phase 2 | — |
| Won | 購入辞退 | Declined | Phase 2 | — |
| Purchased | 初値売却 | Sold | Phase 2 | — |

### 7.3 申し込みステータス遷移

```mermaid
stateDiagram-v2
    [*] --> Pending : 申し込み対象として生成
    Pending --> Applied : ブラウザ操作で申し込み完了
    Applied --> ResultChecked : 抽選結果を記録
    ResultChecked --> [*]
```

| 現在の状態 | イベント | 次の状態 | 条件 |
|---|---|---|---|
| — | 適格性判定通過 | Pending | 対象銘柄として選出 |
| Pending | 申し込み完了 | Applied | 証券会社サイトで申し込み操作成功 |
| Applied | 結果確認 | ResultChecked | 抽選結果（Won/Lost/Alternate）を記録 |

## 変更履歴

| バージョン | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 1.0.0 | 2026-03-25 | lihs | 初版作成 |
