# リリース Runbook (Phase 6 Sprint 13)

> 作成: 2026-04-22
> 用途: `main` merge → 本番ロールアウト → 初日安定化までの手順書。`release-checklist.md` と連動。

---

## 0. ロールアウトフェーズ概観

```
[stg] → [prd canary 10%] → [prd 50%] → [prd 100%]
   |        |                |              |
   30min   30min            60min          watch 24h
   健全   健全             健全           問題なし → 完了
```

- 各フェーズで `docs/10-operations-design/release-checklist.md` の該当項目にサインオフ
- 問題発生時は **即時ロールバック** (§3 参照)

---

## 1. STG ロールアウト

```bash
# 1. Sprint 13 branch から stg 向けに deploy
make deploy-stg

# 2. 30 分安定稼働を目視確認
#    - Cloud Monitoring dashboard: ipo-api / ipo-browser の latency + error rate
#    - 5xx / ERROR log rate が閾値以下
#    - ジョブは動作 (Cloud Scheduler の次回実行ログを待つ、または手動発火)

# 3. release-checklist.md の 13.1 (楽天証券実 E2E) を stg で実行
#    実際に 1 銘柄申込 → Won/Lost まで 1 サイクル回す

# 4. 問題なければ PRD フェーズへ進む
```

---

## 2. PRD ロールアウト (Blue-Green + Canary)

### 2.1 Canary (10% traffic)

```bash
# Deploy の traffic を 0% (新 revision を使わない状態) で push
gcloud run deploy ipo-api \
  --project=ipotto-prd --region=asia-northeast1 --no-traffic

# 新 revision 名を取得
NEW_REV=$(gcloud run revisions list --project=ipotto-prd --region=asia-northeast1 \
  --service=ipo-api --format="value(metadata.name)" \
  --sort-by="~metadata.creationTimestamp" --limit=1)

# 10% traffic を新 revision に流す
gcloud run services update-traffic ipo-api \
  --project=ipotto-prd --region=asia-northeast1 \
  --to-revisions="${NEW_REV}=10,LATEST=90"
```

30 分間、Cloud Monitoring で以下を watch:
- `ipotto/service_error` metric 変化なし
- 5xx レートが通常時と同等
- Cloud Run latency (P50 / P95 / P99) が通常時と同等

問題あれば §3 ロールバック。

### 2.2 段階的 traffic 増加

```bash
# 50%
gcloud run services update-traffic ipo-api \
  --project=ipotto-prd --region=asia-northeast1 \
  --to-revisions="${NEW_REV}=50,LATEST=50"

# 30 分 watch。問題なければ

# 100%
gcloud run services update-traffic ipo-api \
  --project=ipotto-prd --region=asia-northeast1 \
  --to-revisions="${NEW_REV}=100"
```

他の 4 サービス (ipo-browser / info-fetcher / result-checker / frontend) も同様に段階切替。

### 2.3 `make deploy-prd` をそのまま使う場合

`scripts/deploy.sh` は `gcloud run deploy` で 100% traffic を即流すため、canary プラクティスは手動。Phase 7 以降でスクリプト側に `--canary` フラグを追加することを検討。

---

## 3. ロールバック

### 3.1 自動ロールバック

`scripts/deploy.sh` は health check 失敗時に `scripts/rollback.sh` を自動呼び出しする。

### 3.2 手動ロールバック

```bash
# 単一サービス
make rollback-prd SERVICE=ipo-api

# 全サービス
for s in ipo-api ipo-browser ipo-info-fetcher ipo-result-checker ipo-frontend; do
  bash scripts/rollback.sh prd "${s}"
done
```

rollback.sh は直前 revision を特定し、100% traffic を戻す。

### 3.3 インシデント対応トリガ

Sprint 12 で設定した 4 alert の発火条件:
- HTTP 5xx 急増 (`IPOtto Cloud Run 5xx surge`)
- Cloud Run restart loop
- ERROR log rate 急増
- Pub/Sub DLQ activity

**いずれかが発火したら即時 rollback → 原因調査 → fix PR → 再 rollout**。

---

## 4. 初日 (24 時間) モニタリング

Cloud Monitoring dashboard を最低 30 分に 1 回目視する。担当者不在時は on-call を設定。

- 時刻: 09:00 / 12:00 / 15:00 / 18:00 / 翌 09:00
- 確認項目:
  - `ipotto/service_error` metric の 24 時間累計
  - Cloud Scheduler の jobs 実行結果 (全て SUCCESS)
  - `ipotto/dlq_delivery_failed` metric がゼロ
  - Firestore の書込数 / 読込数が通常時プラマイ 20%

---

## 5. リリース後コミュニケーション

### 5.1 Slack #ops 投稿テンプレート

```
🚀 IPOtto v0.x.y デプロイ完了

- 担当: @____
- 時刻: YYYY-MM-DD HH:MM JST
- ロールアウト結果: Canary 10% → 50% → 100% 全フェーズ異常なし
- 監視: 次回 ____ まで 30 分間隔で dashboard watch
- ロールバック手順: `make rollback-prd SERVICE=<service>` (Sprint 12 で整備済)
- 主な変更: (Release Notes 要約 3 行)
```

### 5.2 Firebase Auth 許可メール追加フロー

1. オペレーターが受付 → Sprint 13 の所有者が `ALLOWED_EMAIL` Secret Manager 値を更新
2. ipo-api を再デプロイ (`make deploy-prd`) または `gcloud run services update-env-vars` で反映
3. 申請者に Slack DM で "ログイン可能" を通知

---

## 6. 関連文書

- [release-checklist.md](./release-checklist.md) — Go/No-Go サインオフ
- [firebase-auth-production-setup.md](./firebase-auth-production-setup.md) — 13.3 手順
- [alert-validation.md](./alert-validation.md) — 13.4 手順
- [operations-design.md](./operations-design.md) — 平時運用 / DR / KPI
