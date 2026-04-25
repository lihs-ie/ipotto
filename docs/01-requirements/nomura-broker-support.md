---
title: "野村證券対応 — 要件定義書"
version: "0.1.0"
status: "draft"
created: "2026-04-24"
last_updated: "2026-04-24"
author: "lihs"
---

# 野村證券対応 — 要件定義書

## 1. 目的

IPOtto Phase 1 MVP は楽天證券のみに対応していたが、Phase 8 として野村證券対応を追加する。野村證券は IPO 引受幹事になることが多く、楽天證券だけでは申込機会を逃すケースを補うことが目的。

本書は野村證券対応の機能要件・非機能要件を定義する。詳細設計は [`docs/03-detailed-design/nomura-broker-adapter.md`](../03-detailed-design/nomura-broker-adapter.md) に、サイト構造の確認は [`docs/user-actions/nomura-broker-information-request.md`](../user-actions/nomura-broker-information-request.md) を参照。

## 2. スコープ

### 2.1 対応範囲

楽天證券で実現済みの機能のうち、以下を野村證券でも実現する:

- 野村證券口座の認証情報登録・更新・削除
- 野村證券への接続テスト（ログイン可否確認）
- 野村證券サイトへの IPO 抽選申込自動化
- 野村證券での抽選結果自動確認
- 野村證券関連の操作ログ記録
- 通知（申込完了 / 当選 / 落選 等）への broker 名称表示

### 2.2 対応範囲外

- 野村證券の **IPO 銘柄情報自動取得**（Phase 1 と同様、外部 IPO 情報サイトからの取得を継続。野村独自情報は対象外）
- 野村證券の **当選後購入意思表示の自動化**（楽天證券でも未対応の Phase 2 機能）
- 野村證券の **モバイルアプリ操作**（PC ブラウザ向けサイトのみ）
- 野村證券の **海外株 / 投資信託** 取引

### 2.3 既存要件との関係

[要件定義書](./requirements-specification.md) の以下要件を野村證券にも適用する:

| 既存要件ID | タイトル | 野村對応で追加考慮する点 |
|---|---|---|
| REQ-002 | IPO抽選申込自動化 | 野村サイト固有の DOM・認証フロー |
| REQ-003 | 抽選結果確認 | 野村サイトの結果表示文面 |
| REQ-007 | 証券口座認証情報の暗号化管理 | broker ごとに異なる credential 構造への対応 |
| REQ-008 | 操作ログの記録 | 野村関連操作を識別できるログ出力 |
| REQ-NF-020〜025 | セキュリティ要件 | 野村認証方式の脅威分析 |

## 3. 機能要件

### 3.1 REQ-100: 野村證券口座の登録

ユーザーは IPOtto の証券口座管理画面で、野村證券の認証情報を登録できる。

- 登録フォームには以下を含む:
  - 証券会社選択（楽天 / 野村）
  - ログイン ID / ログインパスワード
  - 取引暗証番号（野村側で必要な場合のみ。[Q-N-072](../user-actions/nomura-broker-information-request.md#q-n-072) で確定）
  - 2FA 関連の追加情報（[Q-N-023](../user-actions/nomura-broker-information-request.md#q-n-023) で確定）
- credential は楽天と同じく Secret Manager に AES-256 で保存
- Secret Manager のキー命名は `ipo-account-{accountId}` を維持（broker 名は文字列の中に含めない）

### 3.2 REQ-101: 野村證券への接続テスト

登録した野村証券口座に対して接続テスト（ログイン可否確認）を実行できる。

- 動作: ipo-browser がログイン → 2FA 突破 → ホーム画面到達 → ログアウト
- 結果は `ConnectionTestResult`（success / message / tested_at）として保存
- 既存 `POST /api/v1/accounts/{id}/test` を broker 別に分岐して再利用

### 3.3 REQ-102: 野村證券への IPO 抽選申込

ipo-applier の `ApplyForLotteryUseCase` が、活性化済みの野村證券口座に対して既存ロジックを適用する。

- 既存の銘柄フィルタ（exclusion / book_building_period）は変更なし
- broker dispatch によって ipo-browser が野村サイトへ自動操作する
- 結果（success / already_applied / insufficient_balance / failure）を `LotteryApplication` に記録

### 3.4 REQ-103: 野村證券の抽選結果自動確認

ipo-result-checker が、野村證券口座に紐づく `LotteryApplication` について抽選結果を取得する。

- broker dispatch で ipo-browser を経由し、野村抽選結果ページから取得
- 結果ステータスの言語表現（[Q-N-052](../user-actions/nomura-broker-information-request.md#q-n-052)）の翻訳テーブルを `services/ipo-browser/src/flows/lottery-result-translator.ts` に追加

### 3.5 REQ-104: ダッシュボード・操作ログでの broker 表示

- ダッシュボードの「直近のアクティビティ」の `securitiesCompany` 列に「楽天證券」「野村證券」と表示
- 操作ログに野村関連の操作が混在しても、フィルタで broker を絞り込めること

## 4. 非機能要件

| ID | カテゴリ | 要件 |
|---|---|---|
| REQ-NF-100 | 性能 | 野村證券への接続テスト応答時間は楽天と同等（30 秒以内）。2FA 待機時間は除く |
| REQ-NF-101 | 性能 | 野村サイトのレート制限（[Q-N-060](../user-actions/nomura-broker-information-request.md#q-n-060)）を超えないこと |
| REQ-NF-102 | セキュリティ | 野村認証情報の保管・送受信は楽天と同等の強度（AES-256 / TLS 1.2+） |
| REQ-NF-103 | セキュリティ | 野村サイト操作中のスクリーンショット・ログに credential が漏洩しないこと |
| REQ-NF-104 | 可用性 | 野村サイトの UI 変更時、楽天対応の処理に影響しないこと（broker 単位で疎結合） |
| REQ-NF-105 | 拡張性 | 第 3 の証券会社（SBI 等）追加時の改修コストが Phase 8 と同等以下になる構造を採用すること |
| REQ-NF-106 | 監視 | 野村サイト固有のエラーが既存アラートチャネルから検出可能なこと（broker 別ラベリング） |

## 5. 前提条件・制約

- 野村證券のオンライントレードサービス（[Q-N-001](../user-actions/nomura-broker-information-request.md#q-n-001)）契約があること
- 野村證券の利用規約上、自動操作が禁止されていないこと（[Q-N-003](../user-actions/nomura-broker-information-request.md#q-n-003)）
- ipo-browser が動作する Cloud Run リソースで Playwright が野村サイトに到達可能なこと（IP 制限等が無いこと）

## 6. 受け入れ基準

- [ ] 楽天 + 野村の両口座を登録した状態で、`make seed` 後ダッシュボードで両証券会社のデータが混在表示される
- [ ] 野村專用の E2E テストが CI で緑になる（HTML mock 経由）
- [ ] 既存の楽天 E2E テストが回帰しない
- [ ] [`docs/09-security-design/checklist.md`](../09-security-design/checklist.md) の野村関連項目が全て `[x]`
- [ ] 本番接続テスト（手動）で実野村口座にログイン可能

## 7. 関連文書

- [要件定義書](./requirements-specification.md) — Phase 1 全体要件
- [野村證券アダプター詳細設計書](../03-detailed-design/nomura-broker-adapter.md)
- [野村證券認証 セキュリティ設計](../09-security-design/nomura-authentication.md)
- [野村證券統合 運用 Runbook](../10-operations-design/nomura-integration-runbook.md)
- [野村證券情報収集依頼書](../user-actions/nomura-broker-information-request.md)
- [実装ロードマップ Phase 8](../11-implementation-roadmap/README.md#phase-8)

## 8. 更新履歴

| 日付 | 内容 |
|---|---|
| 2026-04-24 | 初版作成（draft）。情報収集依頼書の回答後に詳細を確定 |
