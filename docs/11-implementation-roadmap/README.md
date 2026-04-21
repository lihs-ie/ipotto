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

## 9. 更新履歴

| 日付 | 内容 |
|---|---|
| 2026-04-21 | 初版作成（Phase 0〜6 / 13スプリント構成） |
| 2026-04-21 | 現状スナップショット (§2) を `origin/develop` ベースに更新。Phase 0 Task 0.1 が 80%+ 閾値到達目前、Phase 1 Sprint 1 / Phase 2 Sprint 4 / Phase 3 Sprint 6 の一部は実装済みである点を明記 |
