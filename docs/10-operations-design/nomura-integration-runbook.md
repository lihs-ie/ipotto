---
title: "野村證券統合 — 運用 Runbook"
version: "0.1.0"
status: "draft"
created: "2026-04-24"
last_updated: "2026-04-24"
author: "lihs"
---

# 野村證券統合 — 運用 Runbook

> 親文書: [運用設計書](./operations-design.md) / [リリース Runbook](./release-runbook.md)

## 1. 目的

野村證券対応（Phase 8）の本番投入に向けた運用手順を定義する。

- 野村口座の初回セットアップ手順
- 接続テスト・初回申込の手順
- ログイン失敗・ロックアウト発生時の切り分け
- 野村サイトの UI 変更検知後のホットフィックス手順

## 2. セットアップ手順

### 2.1 前提条件

- 野村證券のオンライントレード口座（[Q-N-001](../user-actions/nomura-broker-information-request.md#q-n-001) で確定の契約種別）
- ログイン ID / ログインパスワード / 取引暗証番号（必要なら）
- 2FA 用認証情報（[Q-N-023](../user-actions/nomura-broker-information-request.md#q-n-023) で確定の方式に応じて）

### 2.2 IPOtto 上での口座登録

1. http://127.0.0.1:3000 (ローカル) または本番 URL にログイン
2. `/accounts/new` へ遷移
3. 「証券会社」プルダウンで **野村證券** を選択
4. 必須項目を入力:
   - ログイン ID
   - ログインパスワード
   - 取引暗証番号（野村側で必要な場合）
   - 2FA 関連の追加情報（OTP メールアドレス / SMS 番号 / TOTP secret 等、選択した認証方式に応じて表示が変わる）
5. 「保存」ボタンをクリック
6. 一覧に戻り、登録した野村口座が `activation: ON` で表示されることを確認

### 2.3 接続テスト

1. 登録した野村口座の行で「接続テスト」ボタンをクリック
2. 30 秒以内に「接続成功」メッセージが表示されることを確認
3. 失敗時は § 4 を参照

## 3. 初回 IPO 申込の確認

### 3.1 ステージング環境での確認

ステージングでは HTML mock を使うため実野村サイトには接続しない。

```bash
# モックサイトへの自動申込テスト
make ci-e2e-frontend  # 野村関連 E2E が含まれる
```

### 3.2 本番初回申込の手動確認

実 IPO 銘柄に対する申込前に、以下を実施:

1. 本番口座を 1 件のみ activation: ON にする（楽天は OFF にして衝突回避）
2. 翌営業日の IPO ブックビルディング期間中の銘柄が存在することを `make seed` ベースで confirm
3. Cloud Scheduler の `ipo-job-trigger` を手動実行
4. ipo-applier ログを `gcloud logging` で監視
5. 野村サイトに手動ログインし、申込履歴に IPOtto 経由の申込が記録されていることを目視確認
6. 確認完了後、楽天 activation: ON に戻す

## 4. インシデント対応

### 4.1 接続テスト失敗

| エラー | 想定原因 | 対処 |
|---|---|---|
| `Login failed: invalid credential` | ログイン ID / パスワード誤り | ユーザーに credential 再確認依頼 |
| `2FA timeout` | OTP メール未到着 / 入力遅延 | OTP 取得経路（IMAP/SMS）の到達性確認 |
| `Account locked` | 連続失敗による野村側ロック | ユーザーが野村サイトで手動ロック解除 → IPOtto 側の口座 activation を一旦 OFF |
| `Selector not found` | 野村サイト UI 変更 | § 4.3 へ |
| `bot detection` | reCAPTCHA / IP 制限 | § 4.4 へ |

### 4.2 申込失敗

| 結果 | 対処 |
|---|---|
| `already_applied` | 重複申込（情報用ログのみ。アクション不要） |
| `insufficient_balance` | 野村口座への入金が必要。ユーザーに通知 |
| `failure` | エラーメッセージを操作ログから確認 → § 4.3 / § 4.4 へ |

### 4.3 野村サイト UI 変更検知

ホットフィックス手順:

1. CI の `docker-compose-smoke` で野村関連テストが落ちる、または本番アラート発火
2. ユーザーに野村サイトの最新 HTML スナップショット取得を依頼（[Q-N-011, 021, 032, 041, 051](../user-actions/nomura-broker-information-request.md) 形式）
3. `services/ipo-browser/src/config/selectors.yaml` の `nomura:` 節を更新
4. ローカル `make ci-e2e-browser` で確認
5. ホットフィックス PR → develop → main へ昇格

### 4.4 bot 検知発生時

1. ipo-browser のログを確認し、`navigator.webdriver` 検知 / reCAPTCHA challenge / IP block のいずれかを特定
2. `services/ipo-browser/src/` の Playwright 設定を見直し:
   - `playwright-extra` + `stealth-plugin` 導入
   - User-Agent 変更
   - リクエスト間隔の延長
3. それでも解消しない場合は、Cloud Run の egress IP 固定化を検討（Cloud NAT）

## 5. アラート設計

[アラート検証書](./alert-validation.md) に以下のアラートを追加:

| アラート名 | 条件 | 通知先 |
|---|---|---|
| `nomura_connection_test_failure` | 過去 1 時間に野村接続テスト失敗 ≥ 3 回 | LINE Notify |
| `nomura_apply_failure_streak` | 同一銘柄に対する野村申込が 2 回連続失敗 | LINE Notify + Email |
| `nomura_selector_unmatched` | `selectors.yaml: nomura` での要素検索失敗 | LINE Notify |

## 6. 監視ダッシュボード

[運用設計書 § 監視](./operations-design.md) の Cloud Monitoring ダッシュボードに以下のパネルを追加:

- 野村関連 OperationLog の件数（過去 24h、success / failure 分割）
- 野村接続テストの応答時間 P50 / P95
- 野村サイトへの HTTP リクエスト失敗率

## 7. ログクエリ

`gcloud logging read` 用のクエリ:

```bash
# 直近 1h の野村関連エラー
gcloud logging read 'resource.type="cloud_run_revision"
  AND jsonPayload.securitiesCompany="Nomura"
  AND severity>=ERROR' --limit 50 --format json
```

## 8. ロールバック手順

野村対応リリース直後に問題発生した場合:

1. 全野村口座を `activation: OFF` へ一括更新（管理 UI から、または Firestore 直接）
2. ipo-applier / ipo-result-checker は野村口座をスキップするため、楽天のみで稼働継続
3. 必要に応じて Cloud Run リビジョンを Phase 7 完了時点へ rollback（[release-runbook.md § ロールバック](./release-runbook.md)）

## 9. 関連文書

- [運用設計書](./operations-design.md)
- [リリース Runbook](./release-runbook.md)
- [リリースチェックリスト](./release-checklist.md)
- [アラート検証書](./alert-validation.md)
- [野村證券対応 要件定義書](../01-requirements/nomura-broker-support.md)
- [野村證券アダプター詳細設計書](../03-detailed-design/nomura-broker-adapter.md)

## 10. 更新履歴

| 日付 | 内容 |
|---|---|
| 2026-04-24 | 初版作成（draft）。Phase 8 着手後に手順を確定 |
