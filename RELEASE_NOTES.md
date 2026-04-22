# Release Notes

> 本ファイルは各リリースのハイライトを要約する。GitHub Release ページと併用する前提。
> 手順: `.github/workflows/release.yml` が自動生成する Release Notes を元に、主要変更 + アップグレード手順を本ファイルに追記する。

---

## v0.1.0 — YYYY-MM-DD (テンプレート)

### Highlights

- (1〜3 行で何ができるようになったかを簡潔に)
- 例: 楽天証券 IPO 抽選申込を自動化。Firebase Auth 経由のダッシュボードから銘柄一覧 / 申込履歴 / 除外リスト / 通知設定 / 証券口座管理が操作可能。

### Features

- #PR: <短い要約>
- ...

### Fixes

- #PR: <短い要約>
- ...

### Breaking changes

- None / 影響する API path の変更 等

### Upgrade notes

- `terraform apply` 順序: `ipotto-stg` → 安定確認後 `ipotto-prd`
- 新 env var: `NEXT_PUBLIC_FIREBASE_API_KEY` (既存 Cloud Run は再デプロイ必要)
- Firestore composite index: 追加なし / 追加あり (apply 必要)
- Secret Manager: `allowed-email` / `sendgrid-api-key` を事前登録
- Pub/Sub: 新 topic / subscription は Terraform が作成

### 監視

- Sprint 12 で追加した 4 アラート (`IPOtto Cloud Run 5xx surge` / `restart loop` / `service ERROR logs` / `Pub/Sub DLQ activity`) が本番でも発火することを `docs/10-operations-design/alert-validation.md` の手順で確認済。

---

## リリース時コミュニケーション手順

### 1. 社内告知 (Slack #announcements)

```
📢 IPOtto v0.x.y リリース完了

**主な変更**
- (1 行)
- (1 行)

**動作確認**
- 本番 URL: https://ipotto.example.com
- 監視ダッシュボード: https://console.cloud.google.com/monitoring/dashboards

**問い合わせ**
- 運用担当: @____
- インシデント: #ops-incident
```

### 2. オペレーター通知 (Slack #ops)

```
🚀 IPOtto v0.x.y Deploy 完了

- 担当: @____
- 時刻: YYYY-MM-DD HH:MM JST
- ロールアウト結果: Canary 10% → 50% → 100% 異常なし
- 監視: 次回 HH:MM まで 30 分間隔で dashboard を watch
- ロールバック手順: `make rollback-prd SERVICE=<service>`
- Runbook: docs/10-operations-design/release-runbook.md
```

### 3. Firebase Auth 許可メール追加 (運用リクエスト対応)

1. 申請を受付 (Slack DM / Issue / メール)
2. `ALLOWED_EMAIL` Secret Manager を更新:
   ```bash
   gcloud secrets versions add allowed-email \
     --project=ipotto-prd --data-file=<(echo -n "new-email@example.com,existing-emails,...")
   ```
3. Cloud Run を更新して反映:
   ```bash
   gcloud run services update ipo-api \
     --project=ipotto-prd --region=asia-northeast1 \
     --update-secrets="ALLOWED_EMAIL=allowed-email:latest"
   ```
4. 申請者に Slack DM で "ログイン可能になりました" を通知

### 4. インシデント対応

- Cloud Monitoring alert 発火 → 運用メール到達
- on-call が `docs/10-operations-design/release-runbook.md §3` のロールバック手順を実行
- 原因除去 PR を develop に出し、次回リリース時に本ファイル Fixes セクションに記載
