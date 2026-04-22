# リリースチェックリスト (Phase 6 Sprint 13)

> 作成: 2026-04-22
> 用途: 本番 rollout 時のサインオフ票。Go/No-Go 判定までのすべての確認事項を本書で trace する。

各タスク完了時に責任者 + 日時 + 結果 (OK / NG + 理由) を記入する。

---

## 13.0 前提条件

| 項目 | 責任者 | 日時 | 結果 |
|---|---|---|---|
| `develop` が `main` への merge 準備完了 (`release/*` ブランチ作成) | — | — | [ ] |
| CI 全 7 ジョブ緑 (develop 直近 commit) | — | — | [ ] |
| `release.yml` が triggered され `v0.x.y` タグ + GitHub Release 生成済 | — | — | [ ] |
| `RELEASE_NOTES.md` を本リリース向けに反映 | — | — | [ ] |
| `docs/09-security-design/checklist.md` Sprint 13 項目の整理完了 | — | — | [ ] |

---

## 13.1 楽天証券実サイト手動 E2E (stg 環境)

**事前**: 少額取引用の楽天証券テストアカウントを確保。申込は最小単位、公開価格の下限で行う。

| ステップ | コマンド / 操作 | 期待結果 | 責任者 | 日時 | 結果 |
|---|---|---|---|---|---|
| 1. アカウント接続テスト | frontend `/accounts/{id}/test` 押下 | `success: true` | — | — | [ ] |
| 2. 銘柄取得 | `curl -X POST http://<ipo-api>/internal/pubsub/fetch` | `fetchedCount >= 1` + Firestore 書込確認 | — | — | [ ] |
| 3. 少額申込 | frontend 銘柄詳細から「申込」ボタン | `ApplicationCompleted` event 発火 + LINE 通知到達 | — | — | [ ] |
| 4. 抽選結果確認 | 翌営業日に `curl -X POST <ipo-result-checker>/internal/pubsub/check` | `LotteryResultWon/Lost` event + Email 到達 | — | — | [ ] |
| 5. ログ確認 | Cloud Logging で構造化 JSON 閲覧 | `severity` / `request_id` / マスク済 email が見える | — | — | [ ] |

---

## 13.2 本番 Terraform apply + Cloud Run デプロイ

**前提**: `make pre-deploy-smoke-prd` 緑 + オペレーターが `gcloud auth login` 済。

| 項目 | コマンド | 責任者 | 日時 | 結果 |
|---|---|---|---|---|
| 1. `terraform -chdir=terraform/environments/prd init` | | — | — | [ ] |
| 2. `terraform -chdir=terraform/environments/prd plan -out=tfplan` | plan の差分を `docs/10-operations-design/release-runbook.md` と照合 | — | — | [ ] |
| 3. `terraform apply tfplan` | — | — | — | [ ] |
| 4. `make deploy-prd` でサービスを段階的に deploy | 5 サービス全て緑で出る | — | — | [ ] |
| 5. `curl -sf https://<ipo-api>/health` | `{"status":"ok"}` | — | — | [ ] |
| 6. 本番 docker-compose 向け build args 更新 + CI 再走 | build-and-deploy workflow run 緑 | — | — | [ ] |

---

## 13.3 Firebase Auth 本番プロジェクト設定

詳細: [firebase-auth-production-setup.md](./firebase-auth-production-setup.md)

| 項目 | 責任者 | 日時 | 結果 |
|---|---|---|---|
| 1. Firebase Console で `ipotto-prd` プロジェクト作成 | — | — | [ ] |
| 2. Google Sign-In provider 有効化 | — | — | [ ] |
| 3. Authorized Domain 追加 (本番 frontend URL) | — | — | [ ] |
| 4. OAuth redirect URI 設定 (Cloud Run URL) | — | — | [ ] |
| 5. `ALLOWED_EMAIL` を Secret Manager に登録 | — | — | [ ] |
| 6. `NEXT_PUBLIC_FIREBASE_*` build args を本番値に差替 | — | — | [ ] |
| 7. `/login` → Google Sign-In で本番ダッシュボード到達 | — | — | [ ] |

---

## 13.4 監視アラート動作確認

詳細: [alert-validation.md](./alert-validation.md)

| 項目 | 責任者 | 日時 | 結果 |
|---|---|---|---|
| 1. 5xx surge alert — `scripts/simulate-5xx.sh` 実行 → 運用メール到達 | — | — | [ ] |
| 2. Restart loop alert — `min-instances 0` + delete revisions で発火 → メール | — | — | [ ] |
| 3. ERROR log rate alert — ipo-browser に 400 連打 → ログ metric 発火 → メール | — | — | [ ] |
| 4. Pub/Sub DLQ alert — 壊れた payload を投入 → DLQ metric 発火 → メール | — | — | [ ] |
| 5. 各 alert 発火後に閾値内に戻ることを確認 | — | — | [ ] |

---

## 13.5 運用ドキュメント揃い確認

| 項目 | 責任者 | 日時 | 結果 |
|---|---|---|---|
| `docs/10-operations-design/release-checklist.md` 存在 | — | — | [ ] |
| `docs/10-operations-design/release-runbook.md` 存在 | — | — | [ ] |
| `docs/10-operations-design/firebase-auth-production-setup.md` 存在 | — | — | [ ] |
| `docs/10-operations-design/alert-validation.md` 存在 | — | — | [ ] |
| `RELEASE_NOTES.md` (root) 存在 | — | — | [ ] |
| `scripts/{deploy,rollback,pre-deploy-smoke}.sh` 存在 | — | — | [ ] |
| Makefile の deploy / rollback ターゲット動作確認済 | — | — | [ ] |

---

## Go/No-Go 判定会議

- 上記 13.0〜13.5 全項目が OK なら **Go**: `release/*` → `main` merge を実行
- 1 件でも NG なら **No-Go**: 原因除去 PR を develop に出し、再度本 checklist を走らせる

**判定結果**: [ ] Go / [ ] No-Go — 日時: ____ / 責任者: ____
