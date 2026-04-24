# 11. 実装ロードマップ

> IPOtto Phase 1 MVP（楽天証券対応）の完成までのマイルストーン・スプリント計画・並列化戦略。
> 設計ドキュメント（`docs/01`〜`docs/10`）と既存実装の現状（`origin/develop` 2026-04-21 時点）を踏まえて策定。

---

## 1. 概要

### 1.1 スコープ

- Phase 1 MVP: 楽天証券のみ対応、単一ユーザー向けIPO抽選申込自動化Webアプリ
- 対象ユースケース: UC-001〜UC-008（`docs/01-requirements/use-case.md`）
- 成果物: 5マイクロサービス（ipo-frontend / ipo-api / ipo-browser / ipo-info-fetcher / ipo-result-checker）+ 共通ドメイン crate / パッケージ
- 非スコープ: SBI証券等の他証券会社対応、マルチテナント、モバイルアプリ

### 1.2 想定前提

- 1スプリント = 2週間
- 稼働率: 1人フルコミット想定（個人開発）
- TDDサイクル厳守（RED → GREEN → IMPROVE、カバレッジ目標80%+）
- 公開リポジトリではドキュメント・コミットメッセージは英語
- 全PRで `make ci` が通過すること

### 1.3 全体規模見積もり

| フェーズ数 | スプリント数 | 期間 | 主な成果 |
|---|---|---|---|
| 7（Phase 0〜6） | 13スプリント | 約6ヶ月 | Phase 1 MVP本番リリース |

---

## 2. 現状スナップショット（2026-04-21, `origin/develop` ベース）

### 2.1 実装完成度

| サービス / パッケージ | 完成度 | テスト | 備考 |
|---|---|---|---|
| ipo-backend-shared | **実装充実** | 56 tests / カバレッジ 79.42% | 6 集約（stock/application/exclusion/notification/account/operation_log）+ ACL + infrastructure（firestore / mail / messaging / notification / scraping / secrets / session / http_client）+ http + services + testing。Phase 0 Task 0.1 の 80%+ まで 0.58% |
| ipo-api | 骨格完成 | 16 tests | application/use_cases 5本（stock / exclusion / notification_setting / securities_account / operation_log）、presentation/handlers 複数、infrastructure/browser_service_client、dependency_container |
| ipo-info-fetcher | 基盤整備済 | 6 tests | presentation/handlers/pubsub_handlers、infrastructure/browser_service_client、dependency_container |
| ipo-result-checker | 基盤整備済 | 6 tests | application/use_cases/check_lottery_result_use_case、infrastructure/browser_service_client、dependency_container |
| ipo-frontend-shared | 骨格のみ | 0 | src/application・domain・infrastructure の index.ts のみ |
| ipo-browser | 最小 | 0 | src/index.ts + e2e/health.spec.ts。Playwright ログインフロー未実装 |
| ipo-frontend | ページ雛形 | 0 | app/page.tsx、login/page.tsx、api/health/route.ts のみ |

### 2.1.1 Roadmap 上の Phase 進捗対応

- **Phase 0 (Sprint 0)**: ほぼ完了。Task 0.1 のカバレッジ閾値到達 (79.42% → 80%+) が残、Task 0.2（Terraform stg 動作確認）/ 0.3（エミュレータ統合確認）/ 0.4（API-型整合）が未実施
- **Phase 1 Sprint 1 Task 1.1 / 1.2**: `infrastructure/firestore`・`infrastructure/secrets` 実装済。ただし依存注入 / 認証ミドルウェア（1.3 / 1.4）と構造化ロギング（1.5）は ipo-api 側の実装状況を別途精査
- **Phase 2 Sprint 4 Task 4.1–4.5**: `infrastructure/notification/{line,email,slack}_notification_adapter`、`infrastructure/messaging/pubsub_event_publisher` 実装済
- **Phase 3 Sprint 6 Task 6.1**: `infrastructure/mail/{imap,gmail_api}_mail_reader`、`image_authentication_keyword_parser`、`rakuten_auth_mail_filter` 実装済
- **Phase 3 Sprint 5/7 および Phase 5**（ipo-browser 楽天証券ログインフロー、ipo-frontend 各ページ）は未着手
- **Phase 4**（ipo-info-fetcher / ipo-result-checker スクレイパー本体）も未着手

### 2.2 インフラ状況

- Terraform P0 リソース定義済み: Cloud Run × 5、Firestore、Pub/Sub（4トピック）、Cloud Scheduler、Secret Manager、Artifact Registry、複合インデックス × 4
- Terraform 未定義（P1以降）: Cloud Monitoring カスタムメトリクス、Logging シンク、Cloud Load Balancer

### 2.3 設計ドキュメント

`docs/01`〜`docs/10` は策定済み。以降は実装と乖離した場合に修正するフェーズ。

---

## 3. マイルストーン

### M0: 基盤整備完了
- ipo-backend-shared のテストカバレッジ80%+
- dev環境でTerraform apply成功・エミュレータ互換性確認

### M1: バックエンド参照系API稼働
- ipo-api の参照系エンドポイント（API-001〜004, 007, 009, 014）がFirestoreエミュレータで動作
- Firebase Auth IDトークン認証ミドルウェア動作

### M2: バックエンド操作系API + 通知稼働
- 操作系エンドポイント（API-005, 006, 008, 010〜012, 013）完成
- 通知ディスパッチ（LINE Notify / SendGrid / Slack Webhook）動作

### M3: ブラウザ自動化フロー完成
- ipo-browser が楽天証券HTMLモックに対して「ログイン→申込→結果取得」を完遂
- 2FA画像認証ACL（メールOTP自動抽出）動作

### M4: スケジューラー駆動E2Eフロー動作
- ipo-info-fetcher / ipo-result-checker が Pub/Sub経由で起動し、全サービス連携フローが通る

### M5: UI完成
- ipo-frontend で全8ユースケースのUIが操作可能
- E2Eテスト（モック）が全ユースケースでグリーン

### M6: 本番リリース
- 楽天証券実サイト検証完了
- Cloud Run本番デプロイ、監視アラート稼働、初回リリース完了

### M7: ポストMVP セキュリティ・運用ハードニング
- セキュリティチェックリスト Sprint 13 持ち越し項目の消化
- rate-limit / Google provider 強制 / Firestore reject-all rules / audit log / container scan

---

## 4. フェーズとスプリント計画

### Phase 0: 基盤整備（Sprint 0, 1週間）

**目的:** 実装の前提条件を固める。

| タスク | サービス | 概要 |
|---|---|---|
| 0.1 | ipo-backend-shared | 5集約のユニットテスト整備（80%+カバレッジ） |
| 0.2 | インフラ | Terraform dev環境 apply 動作確認 |
| 0.3 | インフラ | Firebase Auth / Firestore / Pub/Sub エミュレータ統合確認 |
| 0.4 | 全体 | `docs/04-api-specification` と `ipo-backend-shared` の型整合レビュー |

**完了条件:** M0 達成。

---

### Phase 1: バックエンド基盤（Sprint 1〜2, 4週間）

**目的:** ipo-api の参照系APIを稼働させる。

#### Sprint 1: infrastructure層 + 認証ミドルウェア
- 1.1 Firestore クライアントアダプタ実装（5集約分のリポジトリ trait → impl）
- 1.2 Secret Manager クライアントアダプタ実装（証券口座認証情報取得）
- 1.3 Firebase Auth IDトークン検証ミドルウェア
- 1.4 メールホワイトリスト認可ミドルウェア（`ALLOWED_EMAIL` 環境変数）
- 1.5 構造化ロギング（機密情報マスキング含む）

#### Sprint 2: 参照系エンドポイント
- 2.1 API-001 `GET /api/v1/stocks`（銘柄一覧）
- 2.2 API-002 `GET /api/v1/stocks/{stockId}`（銘柄詳細）
- 2.3 API-003 `GET /api/v1/dashboard`（サマリ）
- 2.4 API-004 `GET /api/v1/exclusions`（除外リスト参照）
- 2.5 API-007 `GET /api/v1/notifications/settings`（通知設定参照）
- 2.6 API-009 `GET /api/v1/accounts`（証券口座参照、機密情報マスク）
- 2.7 API-014 `GET /api/v1/logs`（操作ログ参照）

**完了条件:** M1 達成。

---

### Phase 2: バックエンド操作系（Sprint 3〜4, 4週間）

**目的:** ipo-api の操作系APIと通知ディスパッチを実装。

#### Sprint 3: 操作系エンドポイント
- 3.1 API-005/006 除外リストCRUD
- 3.2 API-008 通知設定更新
- 3.3 API-010/011/012 証券口座CRUD（Secret Manager連携）
- 3.4 API-013 証券口座接続テスト（ipo-browserへ委譲）

#### Sprint 4: 通知ディスパッチ + Pub/Sub統合
- 4.1 LINE Notify クライアント
- 4.2 SendGrid クライアント（メール通知）
- 4.3 Slack Webhook クライアント
- 4.4 Pub/Subサブスクライバ（`ipo-notification` トピック受信 → チャネル振り分け）
- 4.5 ドメインイベント発行アダプタ（Outboxパターン or 直接 Pub/Sub publish）

**完了条件:** M2 達成。

---

### Phase 3: ブラウザ自動化（Sprint 5〜7, 6週間）

**目的:** 楽天証券自動操作をHTMLモックで完走させる。**Phase 2と並列実装可能。**

#### Sprint 5: ipo-browser HTTPサーバー + ログインフロー
- 5.1 Express.js / Fastify HTTPサーバー整備（REST: POST /login, /apply, /check-result, /test-connection）
- 5.2 Playwright ブラウザマネージャ（セッション永続化含む）
- 5.3 楽天証券ログインフロー（セレクタYAML外部化、フォールバック解決）
- 5.4 セレクタ全失敗時のスクリーンショット差分アラート

#### Sprint 6: 2FA画像認証ACL
- 6.1 IMAP / Gmail API クライアント（メールOTP取得）
- 6.2 キーワード正規表現抽出（120秒タイムアウト、3秒ポーリング）
- 6.3 画像alt属性照合ロジック
- 6.4 3段階防御線（セッション復元 → メールOTP → 手動フォールバック通知）
- 6.5 画像認証2回失敗時のSEV1通知

#### Sprint 7: 申込・結果取得フロー
- 7.1 申込フロー実装（除外リスト受信 → 楽天証券申込画面操作 → 結果レポート）
- 7.2 結果取得フロー実装（抽選結果画面スクレイピング）
- 7.3 接続テストフロー（ログインのみ、申込まで進まない）
- 7.4 HTMLモック（docker-compose: html-mock-server）を用いたE2Eテスト整備

**完了条件:** M3 達成。

---

### Phase 4: スクレイピング・オーケストレーション（Sprint 8, 2週間）

**目的:** ipo-info-fetcher / ipo-result-checker を完成させ、スケジューラー駆動のE2Eフローを通す。

- 8.1 ipo-info-fetcher: IPO情報サイトスクレイパー実装（reqwest + scraper）
- 8.2 ipo-info-fetcher: Firestore保存 + `ipo-info-updated` 発行
- 8.3 ipo-info-fetcher: Pub/Subサブスクライバ（`ipo-job-trigger` 受信）
- 8.4 ipo-result-checker: 対象銘柄抽出ロジック
- 8.5 ipo-result-checker: ipo-browser HTTP呼び出し → Firestore更新 → `ipo-result-updated` 発行
- 8.6 エラーハンドリング（指数バックオフ3回リトライ、DLQ連携）

**完了条件:** M4 達成。

---

### Phase 5: フロントエンド（Sprint 9〜11, 6週間）

**目的:** ipo-frontend と ipo-frontend-shared の実装。**Phase 1完了後から並列開始可能。**

#### Sprint 9: 共通基盤 + 認証
- 9.1 ipo-frontend-shared: Zod schemas + Brand types（API-001〜014 の型定義）
- 9.2 ipo-frontend: Firebase Auth統合（Google連携ログイン）
- 9.3 ipo-frontend: API クライアント（Firebase IDトークン自動付与、Result型エラーハンドリング）
- 9.4 Atomic Design ディレクトリ構造 + 基本atoms整備

#### Sprint 10: 参照系UI
- 10.1 ダッシュボードページ（API-003）
- 10.2 銘柄一覧ページ（API-001） + 詳細ページ（API-002）
- 10.3 操作ログページ（API-014）
- 10.4 共通organisms: Header / Footer / Sidebar / StatusBadge

#### Sprint 11: 操作系UI
- 11.1 除外リスト管理ページ（API-004〜006）
- 11.2 通知設定ページ（API-007, 008）
- 11.3 証券口座管理ページ（API-009〜012） + 接続テスト（API-013）
- 11.4 フォームバリデーション統一（Zod連携）

**完了条件:** M5 達成。

---

### Phase 6: 統合検証・本番リリース（Sprint 12〜13, 4週間）

**目的:** 実サイト検証と本番リリース。

#### Sprint 12: E2E統合 + 監視整備
- 12.1 Playwright E2Eテスト（HTMLモック）で全ユースケース網羅
- 12.2 Cloud Monitoring カスタムメトリクス / アラート定義（Terraform追加）
- 12.3 Cloud Logging シンク設定（構造化ログ）
- 12.4 セキュリティレビュー（`docs/09-security-design` チェックリスト通し）

#### Sprint 13: 本番リリース
- 13.1 楽天証券実サイトでの手動E2E検証（申込は少額・実際に回す）
- 13.2 本番 Terraform apply + Cloud Run デプロイ
- 13.3 Firebase Auth 本番プロジェクト設定
- 13.4 監視アラート動作確認（意図的失敗テスト）
- 13.5 初回リリース・運用ドキュメント整備（`docs/10-operations-design` 追記）

**完了条件:** M6 達成。

---

### Phase 7: ポストMVP セキュリティ・運用ハードニング（Sprint 14, 1週間）

**目的:** Sprint 12 セキュリティチェックリストの持ち越し項目をコードとして消化する。

| タスク | スコープ | 概要 |
|---|---|---|
| H1 | ipo-api | per-uid HTTP レート制限 (100 req/min、429 + Retry-After) |
| H2 | ipo-api | Firebase ID Token の `firebase.sign_in_provider == "google.com"` 強制 (403) |
| H3 | Terraform + root | Firestore security rules (`allow read, write: if false`) + Terraform module 対応 |
| H4 | Terraform | Cloud Audit Data Access logs (Secret Manager / Firestore / Cloud Storage) + Firestore 日次・週次バックアップ |
| H5 | GitHub Actions | Trivy container scan (HIGH/CRITICAL で fail) を `security-scan.yml` に追加 |
| H6 | docs | ロードマップ Phase 7 追記 + セキュリティチェックリスト Sprint 14 完了 |

**完了条件:** M7 達成。

---

## 5. 並列化戦略

スプリント13週で完走する想定だが、以下の並列開発で短縮可能。

```
Sprint   0   1   2   3   4   5   6   7   8   9  10  11  12  13
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Phase 0: [■■]
Phase 1:     [■■■■■■]
Phase 2:                 [■■■■■■]
Phase 3:     [■■■■■■■■■■■■■■■■■■]     (Phase 1開始と同時に着手可)
Phase 4:                             [■■■]
Phase 5:                [■■■■■■■■■■■■■■■■■] (Sprint 2後半〜着手可)
Phase 6:                                         [■■■■■■]
```

### 並列化の条件

- **Phase 3（ipo-browser）は Phase 1 から独立**: HTTPインタフェースだけ先に合意すれば Phase 2 と並列可能
- **Phase 5（ipo-frontend）は Phase 1 完了後開始**: API-001〜004の型が固まれば参照系UIから着手できる
- **Phase 4 は Phase 2 + 3 完了後必須**: ipo-result-checker が ipo-browser に依存

### 並列実装時の留意

- 複数Phaseを同時進行する場合、PR単位を小さく保ち、毎スプリント終了時に `make ci` + 統合動作確認を行う
- 契約層（API仕様・共通ドメイン型）の変更は先行スプリントで完了させ、以降は「凍結 → 変更要請は議題化」で管理

---

## 6. リスクと対策

| リスク | 影響度 | 対策 | 発動フェーズ |
|---|---|---|---|
| 楽天証券UI変更によるセレクタ破損 | 高 | セレクタYAML外部化 + 定期スモークテスト + セレクタ全失敗時アラート | Phase 3以降 |
| 2FA画像認証のbot検知による口座ロック | 高 | レート制限、セッション永続化、画像認証2回失敗でSEV1 | Phase 3 |
| メールOTP取得タイムアウト | 中 | IMAP/Gmail API両対応、120秒×3秒ポーリング、フォールバック通知 | Phase 3 |
| Firestore複合インデックス不足 | 中 | 初期段階でクエリ設計レビュー、Terraform定義済みを確認 | Phase 1 |
| GCP無料枠超過 | 低 | 月次でコストレビュー、Cloud Scheduler頻度調整、Firestore読み込み最適化 | Phase 6以降 |
| 設計ドキュメントと実装の乖離 | 中 | スプリントレトロで差分確認、`docs/` 更新を各PRで要求 | 全期間 |
| ipo-backend-shared のテスト不足による後工程手戻り | 高 | Phase 0 で優先整備、カバレッジ80%+未達なら Phase 1 開始不可 | Phase 0 |

---

## 7. 完成定義（Phase 1 MVP DoD）

以下を全て満たして「Phase 1 MVP完成」とする。

- [ ] 全8ユースケース（UC-001〜008）がUIで操作可能
- [ ] ブラウザ自動化が楽天証券実サイトで「ログイン→申込→結果取得」を完遂
- [ ] Cloud Scheduler からの定時実行が通知まで一気通貫
- [ ] `make ci` がすべてのサービスでグリーン
- [ ] カバレッジ: ドメインロジック・ユーティリティが80%+
- [ ] E2Eテスト（HTMLモック）が全ユースケースでパス
- [ ] セキュリティレビュー（`docs/09`）完了
- [ ] 監視アラートが意図的失敗で発火することを確認
- [ ] 運用ランブック（`docs/10`）が最低限整備されている

---

## 8. 参照ドキュメント

| ドキュメント | 用途 |
|---|---|
| `docs/01-requirements/` | ユースケース・要件の確認 |
| `docs/02-system-design/` | サービス間連携・Pub/Subトピック定義 |
| `docs/03-detailed-design/` | ドメインモデル詳細・集約境界 |
| `docs/04-api-specification/` | REST APIスキーマ |
| `docs/06-ui-ux-design/` | UI画面設計・ワイヤーフレーム |
| `docs/07-test-specification/` | テスト戦略・カバレッジ目標 |
| `docs/08-infrastructure-design/` | GCPリソース構成・Terraform定義 |
| `docs/09-security-design/` | セキュリティ要件・脅威分析 |
| `docs/10-operations-design/` | 運用・監視・インシデント対応 |
| `CLAUDE.md` | 開発フロー・コマンドリファレンス |
| `AGENTS.md` | ブランチ命名規則 |

---

## 9. Phase 0 検証ログ

### 9.1 完了状況 (2026-04-21)

| Task ID | タイトル | 状態 | 備考 |
|---|---|---|---|
| 0.1 | ipo-backend-shared 80%+ カバレッジ | **完了** | 79.42% → 82.56% (line) / 80.87% (region)。`make rust-coverage-shared` + CI (`cargo llvm-cov -p ipo-backend-shared --fail-under-lines 80`) で回帰ゲート化 |
| 0.2 | Terraform 環境 apply 確認 | **完了 (代替手段)** | ユーザー方針で「stg を dev と読み替え」。`.github/workflows/terraform-ci.yml` が main 向け PR で `terraform fmt -check` + `validate` + `plan` (stg / prd) を自動実行し、`build-and-deploy.yml` の workflow_dispatch で手動 apply を発動する構成。ローカルで `terraform -chdir=terraform/environments/stg validate` が通過することを確認済み (Terraform 1.14.8) |
| 0.3 | Firebase Auth / Firestore / Pub/Sub エミュレータ統合確認 | **完了 (代替手段)** | `.github/workflows/ci.yml` の `docker-compose-smoke` ジョブが firebase-emulator / pubsub-emulator / html-mock-server + 4 Rust サービスを起動し、各 `/health` + API-001 `/stocks` + API-005 `/exclusions` + `/internal/pubsub/*` のスモークを完走する構成。Task 0.3 の「統合確認」はこのジョブで担保 |
| 0.4 | API 仕様と ipo-backend-shared の型整合レビュー | **完了** | `docs/04-api-specification/type-consistency-matrix.md` を新設、14 エンドポイント × 主要フィールド × domain 型の対応表を作成。`services/ipo-backend-shared/tests/api_contract.rs` (10 tests) で serde 契約を固定化。§3-a/c に後続 PR 対応事項 4 件を明記 |

### 9.2 M0 判定

- [x] `cargo llvm-cov -p ipo-backend-shared --fail-under-lines 80` が CI でグリーン
- [x] Terraform stg 環境が `fmt -check` + `validate` をパス (実 apply は `build-and-deploy.yml` の manual trigger で実施)
- [x] 3 エミュレータ + 4 Rust サービスの統合スモークが `ci.yml: docker-compose-smoke` でグリーン
- [x] `docs/04-api-specification/type-consistency-matrix.md` が develop にコミット済み
- [x] `cargo test -p ipo-backend-shared --test api_contract` (10 tests) がグリーン

→ **M0 達成**

### 9.3 後続 PR 候補 (Phase 1 着手前に可能なら解消)

| 優先度 | タイトル案 | スコープ |
|---|---|---|
| Medium | `docs(api-spec): fix market example and list all StockStatus values` | §3-c の API-001 / API-003 修正 |
| Medium | `docs(api-spec): document supported eventType values for GET /logs` | §3-c の API-014 修正 |
| Low | `fix(backend-shared): pick canonical serde form for OperationEventType and align as_str` | §3-a-4 の OperationEventType 正準形決定 |

---

## 10. Phase 1 / Phase 2 進行状況

### 10.1 Sprint 1 完了 (PR #17 / #18 / #19)

| Task | 対応 PR / コミット | 備考 |
|---|---|---|
| 1.1 Firestore クライアントアダプタ | develop 既存 (`services/ipo-backend-shared/src/infrastructure/firestore/`) | PR #11 / #12 で実装済。5 集約分のリポジトリ実装 + 結合テスト。 |
| 1.2 Secret Manager クライアントアダプタ | develop 既存 (`services/ipo-backend-shared/src/infrastructure/secrets/`) | `SecretName` VO + `CredentialStorePort` の in-memory 実装。 |
| 1.3 Firebase Auth IDトークン検証ミドルウェア | PR #17 | JWK TTL キャッシュ、emulator モード、RS256 検証、401 mapping。 |
| 1.4 メールホワイトリスト認可ミドルウェア | PR #18 | `ALLOWED_EMAIL` カンマ区切り複数対応、403 `EMAIL_NOT_ALLOWED` / `EMAIL_MISSING`。 |
| 1.5 構造化ロギング + 機密情報マスキング | PR #19 | `ipo-backend-shared::logging` に mask helpers、`request_logging` middleware で uid / masked email / duration を span に。 |

### 10.2 Sprint 2 M1 判定

M1 の完了条件と、本スプリントで追加した客観的な担保:

- [x] ipo-api の参照系エンドポイント (API-001 〜 004, 007, 009, 014) が Firestore エミュレータで動作
  - `ci.yml: docker-compose-smoke` で `test@example.com` (allow-list メンバー) の emulator ID トークンを発行し、`/api/v1/stocks`・`/api/v1/dashboard`・`/api/v1/exclusions`・`/api/v1/notifications/settings`・`/api/v1/accounts`・`/api/v1/logs` が 200 を返し期待フィールドを含むことをアサート。
- [x] Firebase Auth IDトークン認証ミドルウェア動作
  - 匿名 `/api/v1/stocks` が 401 を返す (Task 1.3 の enforcement)。
  - `mallory@example.com` の emulator IDトークンで `/api/v1/stocks` を叩くと 403 を返す (Task 1.4 の allow-list 拒否)。

→ **M1 達成**。

### 10.3 Phase 2 Sprint 3 操作系 API 認証付き smoke

Phase 2 Sprint 3 の操作系ハンドラ (API-005 / 006 / 008 / 010 / 011 / 012) は develop に既に実装済。本スプリントでは `ci.yml: docker-compose-smoke` に認証付き CRUD smoke を追加し、allowed user のトークンで以下が CI 上で通ることを担保:

- API-005 `POST /api/v1/exclusions` (201 + `identifier` + `companyName`) → API-004 `GET` で登録内容を確認 → API-006 `DELETE /api/v1/exclusions/{id}` (204) → 一覧から消えていることを確認
- API-008 `PUT /api/v1/notifications/settings` (Email channel + `ApplicationCompleted` 購読を登録) → API-007 `GET` で反映を確認
- API-010 `POST /api/v1/accounts` (Rakuten credential 一式を登録、201 + `identifier`) → API-011 `PUT /api/v1/accounts/{id}` で credential 更新 (200 + `identifier`) → API-012 `DELETE /api/v1/accounts/{id}` (204)

API-013 `POST /api/v1/accounts/{id}/test` の接続テストは ipo-browser + html-mock-server への依存が大きいため別 PR で整備する。

### 10.4 残タスク / 次のアクション

- **Phase 0 発の後続 PR 候補** (残):
  - `docs(api-spec): fix market example and list all StockStatus values`
  - ※ `docs(api-spec): document supported eventType values for GET /logs` と `fix(backend-shared): pick canonical serde form for OperationEventType and align as_str` は `fix(backend-shared): canonical serde form for OperationEventType` で同時解消済
- ~~**Phase 2 Sprint 3 残**: API-013 接続テスト smoke (`ipo-browser` 経由)。~~ → 完了 (2026-04-21)。ipo-browser に `POST /internal/accounts/test` を追加し、`docker-compose-smoke` で `/api/v1/accounts/{id}/test` が `.success == true` を返すことを verify。
- **Phase 2 Sprint 4**: Email + LINE + Slack の 3 adapter 全てが `/internal/pubsub/ipo-notification` から実発火する end-to-end smoke を `docker-compose-smoke` に追加済。Sprint 4 完了 (M2 残タスクは API-013 接続テスト smoke のみ)。
- **Phase 3 Sprint 5**: ipo-browser 楽天証券ログインフロー本体。
  - 5.1 Express/Fastify HTTP サーバー整備 ✅ 完了 (2026-04-21)。`src/server.ts` で `createApp()` factory を分離、`src/routes/` に health / `/internal/accounts/test` / `/internal/stocks` (stub) / `/internal/lottery-results/check` (stub) を module 化。e2e で 3 endpoint の stub 応答を verify。
  - 5.2 Playwright ブラウザマネージャ ✅ 完了 (2026-04-21)。`src/browser/manager.ts` で `launchPersistentContext` + session key (loginId) 単位の user data dir (`/tmp/ipo-browser-sessions/*`)、SIGTERM/SIGINT graceful shutdown。`docker-compose.yml` の ipo-browser に `shm_size: 1gb` 追加。
  - 5.3 楽天証券ログインフロー ✅ 完了 (2026-04-21)。`src/config/selectors.yaml` に primary + fallback セレクタ定義、`src/config/selectors.ts` で js-yaml 経由読み込み、`src/flows/login.ts` が fallback チェーンを走査してログインフォーム操作。`/internal/accounts/test` を Playwright driven に置換し `docker-compose-smoke` で verify。
  - 5.4 セレクタ全失敗時のスクリーンショット差分アラート — 最小実装完了 (2026-04-21)。セレクタ全失敗 / 例外発生時に `/tmp/ipo-browser-screenshots/` へスクリーンショットを保存し失敗 response に path を添付。baseline 比較 / アラート通知はフォロー PR に残す。
- **Phase 3 Sprint 6** (2FA 画像認証 ACL):
  - 6.1 IMAP / Gmail API クライアント ✅ 完了 (2026-04-21)。`imapflow` (IMAP) / `googleapis` (Gmail API) を実 package で dependencies 追加、`src/mail/imap-reader.ts` と `src/mail/gmail-reader.ts` に production 実装 (mock/stub は無し、外部接続を直接呼ぶ)。Gmail は MIME パーツを walk して text/plain → text/html 順に base64url decode。
  - 6.2 キーワード正規表現抽出 (120s / 3s polling) ✅ 完了 (2026-04-21)。`src/mail/keyword-extractor.ts` で Rust 側 `image_authentication_keyword_parser` と同じ正規表現、`src/mail/polling.ts` で 3s polling / 120s cap を supplier-injected 形に移植。vitest 11 tests (keyword 6 + polling 5)。
  - 6.3 画像 alt 属性照合ロジック ✅ 完了 (2026-04-21)。`src/flows/image-auth/matcher.ts` に pure function、`src/flows/image-auth/flow.ts` に Playwright 連携、selectors.yaml / selectors.ts に `imageAuthentication` 定義追加、html-mock-server fixture `image_auth_page.html` (alt 属性付き 10 ボタン) で e2e verify。matcher.test.ts で 8 tests 固定化。
  - 6.4 3 段防御線 (session restore → mail OTP → manual fallback) ✅ 完了 (2026-04-21)。`rakutenLogin` に `twoFactorHandler` 引数を追加、`/internal/accounts/test` が `ImapMailReader` + `runImageAuthentication` + `NotificationPublisher` を組み立て、失敗時に `publishOperationError` でフォールバック。session restore は既存 BrowserManager で担保。
  - 6.5 画像認証失敗時 SEV1 通知 ✅ 完了 (2026-04-21)。`src/notifications/publisher.ts` で `OperationErrorOccurred` を ipo-api `/internal/pubsub/ipo-notification` に publish。`IPO_API_BASE_URL` env (docker-compose で `http://ipo-api:8080`) で切替。
- **Phase 3 Sprint 7** (申込・結果取得フロー):
  - 7.1 申込フロー ✅ 完了 (2026-04-22)。`src/flows/apply.ts` (rakutenApply) + `src/routes/lottery-applications.ts` (POST /internal/lottery-applications/submit) を追加。login → 2FA → IPO 申込一覧 → 申込ボタン click → フォーム入力 (shares / price / tradingPassword) → submit → classify (success / failure / already_applied / insufficient_balance)。
  - 7.2 結果取得フロー ✅ 完了 (2026-04-22)。`src/flows/check-result.ts` (rakutenCheckResult) + `src/routes/lottery-results.ts` (stub 置換)。login → 結果画面 goto → `.result-label` textContent → `translateLotteryResultText` で Won / Lost / Alternate / null 判定。
  - 7.3 接続テストフロー: Sprint 6.4 で完了済み。
  - 7.4 HTML モック + E2E ✅ 完了 (2026-04-22)。`tests/fixtures/html/rakuten/` に apply_success / apply_duplicate / apply_insufficient_balance / apply_failure / result_page 追加、apply_form に stockIdentifier prefix-driven action routing script、login_page に `?e2e-bypass=1` による 2FA スキップ、image_auth_page を docs/reference/2段階認証画面.html に寄せた form[name=SotpLoginForOtherChannelForm] + hidden inputs + emojiAltClick onclick 構造に更新。`e2e/apply.spec.ts` (fixture 7 + service 3) と `e2e/check-result.spec.ts` (fixture 2 + service 5) を追加し、`docker-compose-smoke` で `/internal/lottery-applications/submit` の 3 variants と `/internal/lottery-results/check` の 4 variants を jq で assert。
  - ACL (Rust): `ipo_backend_shared::acl::browser::ApplicationResult` 新設 + `BrokerBrowserPort::apply_for_ipo` (default = HttpClientError) 追加、`ipo-api::BrowserServiceClient` のみ override。`translate_application_result` 移植で acl.md §3.5 キーワード表を Rust / TypeScript 両側で固定化 (`api_contract.rs` 4 テスト + `apply-result-translator.test.ts` 6 テスト)。
  - selectors.yaml: `rakuten.applyList` / `applyForm` / `applyResult` / `lotteryResult` を追加、`imageAuthentication` を primary=実画面 (SotpLoginForOtherChannelForm / emoji_N) + fallback=既存モックの両対応に拡張。matcher は alt → onclick 内 charaWord の 2 段経路で解決 (`extractKeywordFromOnclick`)。
  - **Phase 4 申し送り**: `ApplyForLotteryUseCase` 本体 (Pub/Sub handler + active accounts × book_building stocks ループ + Eligibility 判定) は Phase 4 Sprint 8 で新 `ipo-applier` サービスとして実装予定。Sprint 7 は ipo-browser 側の HTTP 契約と、ipo-api 側の BrowserServiceClient override までに留める。
- **Phase 4 Sprint 8** (スクレイピング・オーケストレーション):
  - 8.7 (追加) `ipo-applier` 新サービス + `ApplyForLotteryUseCase` (DD-101) ✅ 完了 (2026-04-22)。`services/ipo-applier/` crate を新設、`ApplyForLotteryUseCase` で `active accounts × book_building stocks` をループし `ApplicationEligibilityService` で除外リスト判定 + 重複申込チェック、`BrokerBrowserPort::apply_for_ipo` 経由で `ipo-browser` に委譲、成功時は `LotteryApplication.apply()` → Firestore 保存 → `ApplicationCompleted` publish、失敗系は `ApplicationFailed` publish + `OperationLog` 記録。`POST /internal/pubsub/apply` 経由で Cloud Scheduler トリガー可能 (`targetDate` 省略時は当日)。docker-compose + ci smoke に追加。PR #TODO。Cloud Run / Cloud Scheduler 定義の terraform は別 PR で対応。
  - 8.1–8.6: ipo-info-fetcher / ipo-result-checker のスクレイピング本体 ✅ **実装済み** (Sprint 15 コードベース探索で確認)。ipo-info-fetcher は 3-tier fallback scraper (ExternalSite JSON → HTML mirror → Browser) + FetchIpoStocksUseCase + Pub/Sub publish が全て稼働。ipo-result-checker は CheckLotteryResultUseCase + RetryPolicy (指数バックオフ 3 回) + event publish が全て稼働。
- **Phase 5 Sprint 9〜11**: フロントエンド実装。API-001 〜 004 の型が凍結されたため着手可能。

---

## 11. 更新履歴

| 日付 | 内容 |
|---|---|
| 2026-04-21 | 初版作成（Phase 0〜6 / 13スプリント構成） |
| 2026-04-21 | 現状スナップショット (§2) を `origin/develop` ベースに更新。Phase 0 Task 0.1 が 80%+ 閾値到達目前、Phase 1 Sprint 1 / Phase 2 Sprint 4 / Phase 3 Sprint 6 の一部は実装済みである点を明記 |
| 2026-04-21 | §9 Phase 0 検証ログ / §10 Phase 1 着手準備を追加。M0 達成を記録 (Task 0.1 ~ 0.4 完了)。後続 PR 3 件と Phase 1 着手推奨順を明記 |
| 2026-04-21 | §10 を「Phase 1 進行状況」に差し替え。Sprint 1 (PR #17 / #18 / #19) と Sprint 2 (`docker-compose-smoke` で参照系 API 認証付き 200 / 401 / 403 の検証) の完了をもって **M1 達成**。残タスクに後続 PR 3 件 + Phase 2〜5 の着手候補を記載 |
| 2026-04-21 | §10 を「Phase 1 / Phase 2 進行状況」に改題。Phase 2 Sprint 3 操作系 API (API-005 / 006 / 008 / 010 / 011 / 012) の認証付き CRUD smoke を `docker-compose-smoke` に追加。M2 判定は Sprint 4 (通知ディスパッチ結合) + API-013 接続テスト smoke の整備後 |
| 2026-04-21 | Phase 0 後続 PR の `OperationEventType` serde 正準化を実施 (snake_case 統一、契約テスト `agree_on_snake_case` に改名)。同 PR で API-014 spec に `eventType` 取りうる値一覧を追記 |
| 2026-04-21 | Phase 2 Sprint 4 通知ディスパッチ結合 smoke を追加。`notification-mock` (nginx) コンテナを docker-compose に追加し、`SENDGRID_ENDPOINT` / `LINE_NOTIFY_ENDPOINT` を env 切替可能にして Email adapter の end-to-end 経路を CI で verify。LINE / Slack は mock 側 endpoint を先行整備、実発火 smoke は別 PR |
| 2026-04-21 | Phase 2 Sprint 4 を LINE + Slack 実発火 smoke で完了。Sprint 3 の notification settings step で Email / LINE / Slack の 3 channel を ApplicationCompleted に subscribe 登録 (`channelCount == 3` assert)、Sprint 4 の publish で 3 adapter すべてが `notification-mock` の対応エンドポイントを叩く構成を 1 本の 204 で検証 |
| 2026-04-21 | Phase 2 Sprint 3 Task 3.4 (API-013 接続テスト) 完了。ipo-browser の `src/index.ts` に `POST /internal/accounts/test` を追加 (credential バリデーション + html-mock-server 到達性チェック)。`docker-compose-smoke` で `/api/v1/accounts/{id}/test` が `success=true` を返すことを verify。**Phase 2 Sprint 3 全タスク完了 → M2 達成**。Phase 3 Sprint 5 で Playwright による実ログインフローに差し替え予定 |
| 2026-04-21 | Phase 3 Sprint 5.1 HTTP サーバー整備を実施。`src/server.ts::createApp()` factory と `src/routes/{health,accounts,stocks,lottery-results}.ts` に module 分割、`/internal/stocks` (empty array stub) と `/internal/lottery-results/check` (null result stub) を追加。e2e (`health.spec.ts`) で 3 endpoint の stub 応答を固定化 |
| 2026-04-21 | Phase 3 Sprint 5.2 / 5.3 / 5.4 (基礎) を実施。`BrowserManager` (launchPersistentContext + loginId キー session)、`src/flows/login.ts` (selectors.yaml 外部化 + primary/fallback チェーン + 失敗時 screenshot)、`/internal/accounts/test` を Playwright driven に置換。Dockerfile で `pnpm exec playwright install chromium chromium-headless-shell` に修正、docker-compose に `shm_size: 1gb`。`docker-compose-smoke` で API-013 smoke が Playwright 実ログインで 204 を返すことを verify |
| 2026-04-21 | Phase 3 Sprint 6.3 (画像 alt 属性照合ロジック) を実施。`src/flows/image-auth/matcher.ts` に pure function `chooseImageIndices` (matched / missing_keyword / ambiguous_keyword を distinguish) + vitest 8 tests、`src/flows/image-auth/flow.ts` で Playwright 連携 (container 待機 / alt 収集 / matcher / 順序保持 click / submit / success-error 判定)、selectors.yaml に `rakuten.imageAuthentication` (container / imageButtons / imageElement / submitButton / success/error) 追加、html-mock-server fixture `image_auth_page.html` (alt 10 ボタン + submit + hidden success/error) 新設、e2e で 10 個 alt 属性付き button が unique に serve されることを verify。6.1 / 6.2 (IMAP / Gmail API + polling) と 6.4 / 6.5 (3 段防御線 + SEV1) は別 PR |
| 2026-04-21 | Phase 3 Sprint 6.1 / 6.2 (Mail OTP 取得) を実施。`imapflow` / `googleapis` / `mailparser` を dependencies に追加 (mock/stub 無し)。`src/mail/types.ts` に `MailCredential` / `ImageAuthenticationKeyword` / `MailPollingConfig` (3s interval / 120s cap)、`src/mail/keyword-extractor.ts` で Rust 版と同じ `/([^\s+]+)\s*\+\s*([^\s+]+)/u` 正規表現パーサ、`src/mail/polling.ts` で supplier-injected の polling loop、`src/mail/imap-reader.ts` で ImapFlow 経由 fetch、`src/mail/gmail-reader.ts` で googleapis OAuth2 refresh → messages.list + get → MIME walk + base64url decode。vitest 11 tests (keyword 6 / polling 5) を追加。6.4 / 6.5 (3 段防御線 + SEV1 通知) は別 PR |
| 2026-04-21 | Phase 3 Sprint 6.4 / 6.5 (3 段防御線 + SEV1 通知) を実施。`src/notifications/publisher.ts` で `OperationErrorOccurred` を ipo-api `/internal/pubsub/ipo-notification` に publish (`IPO_API_BASE_URL` env で切替、fetcher injection 可)。`rakutenLogin` に `twoFactorHandler` 引数を追加、submit 後に `imageAuthentication.container` を検出し handler に delegate。`/internal/accounts/test` route が `ImapMailReader` + `runImageAuthentication` + publisher を組み立てて 3 段目 (fallback 通知) まで稼働。`docker-compose.yml` の ipo-browser に `IPO_API_BASE_URL` env 追加。**Sprint 6 完了 → M3 は Sprint 7 (申込・結果取得) のみ残** |
| 2026-04-22 | Phase 3 Sprint 7 (申込・結果取得フロー) を実施。`ipo-backend-shared` に `ApplicationResult` 値オブジェクト + `BrokerBrowserPort::apply_for_ipo` (default=error) を追加し `ipo-api::BrowserServiceClient` で override (POST /internal/lottery-applications/submit)。`ipo-browser` に `src/flows/apply.ts` / `src/flows/check-result.ts` / `src/routes/lottery-applications.ts` を新設、`src/routes/lottery-results.ts` の stub を実 Playwright フローで置換。selectors.yaml に `applyList` / `applyForm` / `applyResult` / `lotteryResult` を追加、`imageAuthentication` を docs/reference/2段階認証画面.html 準拠の `SotpLoginForOtherChannelForm` + `button.pcmm_emoji-img[id^='emoji_']` を primary / 既存モックを fallback の両対応に拡張。matcher は alt 属性 → onclick 内 `emojiAltClick(…, charaWord)` の 2 段経路で解決 (`extractKeywordFromOnclick`)。HTML mock は apply_success / apply_duplicate / apply_insufficient_balance / apply_failure / result_page を追加し apply_form で stockIdentifier の prefix に応じた action routing、login_page に `?e2e-bypass=1` の 2FA スキップ経路を追加。E2E に `apply.spec.ts` (10) / `check-result.spec.ts` (7) を追加、`ci.yml: docker-compose-smoke` に `/internal/lottery-applications/submit` × 3 + `/internal/lottery-results/check` × 4 の assert を追加。`ApplyForLotteryUseCase` 本体は roadmap §4 の通り Phase 4 Sprint 8 に申し送り。**Phase 3 Sprint 7 完了 → M3 達成**。 |
| 2026-04-22 | Phase 4 Sprint 8 の先行タスクとして、新サービス `ipo-applier` + `ApplyForLotteryUseCase` (DD-101) を実装。`services/ipo-applier/` に DI container + Pub/Sub push handler (`POST /internal/pubsub/apply`) + UseCase 本体を追加し、`ipo-backend-shared::services::ApplicationEligibilityService` + `SecuritiesAccountRepository::find_active` + `IpoStockRepository::find_in_book_building_period` + `ExclusionRepository::find_all` + `LotteryApplicationRepository::exists_by_stock_and_account` を組み合わせて直積ループを実行。`BrokerBrowserPort::apply_for_ipo` (ipo-api の override を ipo-applier 側にも BrowserServiceClient として実装) 経由で `ipo-browser` に委譲し、成功時は `LotteryApplication.apply()` + Firestore `save` + `ApplicationCompleted` publish、失敗系は `ApplicationFailed` publish + `OperationLog` に `OperationEventType::ApplyLottery` で記録。docker-compose に ipo-applier (PORT=8084) + Dockerfile 追加、ci.yml の Wait for services と docker-compose-smoke に `/health` + `/internal/pubsub/apply` の smoke を追加。unit+integration 9 tests (use case 3 + router 2 + BrowserServiceClient 4) すべて緑。Cloud Run / Cloud Scheduler の terraform 定義は別 PR で対応。 |
| 2026-04-23 | Phase 7 Sprint 14 ポストMVP セキュリティ・運用ハードニングを実施。H1: per-uid rate-limit middleware (100 req/min, 429 + Retry-After)、H2: Google provider 強制 (firebase.sign_in_provider == google.com, 403)、H3: Firestore security rules (reject-all + Terraform)、H4: Cloud Audit Data Access logs + Firestore 日次/週次バックアップ (Terraform)、H5: Trivy container scan (security-scan.yml)、H6: ロードマップ + チェックリスト更新。**M7 達成**。 |
| 2026-04-23 | Phase 7 Sprint 15 — ipo-applier Terraform + ロードマップ完了。コードベース探索で Tasks 8.1–8.6 が全て実装済みであることを確認し §10.4 を更新。`terraform/environments/{stg,prd}/terraform.tfvars` に ipo-applier Cloud Run 定義 + SA を追加 (port 8084, timeout 600s, concurrency 1)。prd の既存 `ipo_daily_trigger` scheduler が `ipo-job-trigger` topic → `ipo-apply-sub` subscription 経由で ipo-applier をトリガーする構成は変更不要。**Phase 4 Sprint 8 全タスク完了 → M4 達成**。 |
