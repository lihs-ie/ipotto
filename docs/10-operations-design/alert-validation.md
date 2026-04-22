# 監視アラート動作確認 (Sprint 13.4)

> 作成: 2026-04-22
> 用途: Sprint 12 で追加した 4 つの Cloud Monitoring アラートが本当に発火し、通知チャネル (メール) に届くことを確認する手順。

前提:
- Sprint 13.2 の terraform apply 完了、observability モジュールが本番にデプロイ済
- `alert_email_recipients` に運用担当のメールアドレスが登録済
- 本テストは **stg 環境** で実施する (本番で 5xx 連打するとユーザー影響があるため)

各テスト終了後は必ず **原状回復** を行い、次のテストへ進む。

---

## 1. 5xx surge alert

### 目的
`IPOtto Cloud Run 5xx surge` が 5 分間で 5xx ≥ 10 件を検知して通知する。

### 手順
```bash
# ipo-api の存在しないエンドポイントに大量に POST
IPO_API_URL="https://ipo-api-xxxxx.a.run.app"
for i in $(seq 1 15); do
  curl -s -o /dev/null -w "%{http_code}\n" \
    -X POST "${IPO_API_URL}/api/v1/nonexistent"
done
```

### 期待値
- 各リクエストが 401 または 404 を返す
- 5xx ではないため実際には alert は発火しないことを確認 (negative smoke)
- → 本物の 5xx を誘発するには、意図的に panic を起こすデバッグエンドポイントを ipo-api に追加する必要がある (Phase 7 検討)

### 代替: Cloud Monitoring 直接テスト
GCP Console → Monitoring → Alerting → `IPOtto Cloud Run 5xx surge` → **Test** ボタンで疑似発火し、メール到達を確認する。

---

## 2. Restart loop alert

### 目的
`IPOtto Cloud Run restart loop` が 1 分間で container restart ≥ 3 を検知して通知する。

### 手順
```bash
# stg の ipo-api を min-instances=0 にし、直近 revision を delete
gcloud run services update ipo-api \
  --project=ipotto-stg --region=asia-northeast1 \
  --min-instances=0 --max-instances=1

# 手動で instance を落とす (何度か connect → disconnect を繰り返す)
for i in $(seq 1 5); do
  curl -sf "${IPO_API_URL}/health" > /dev/null || true
  sleep 10
done
```

### 期待値
- Cloud Monitoring dashboard で `instance_count` が振動する
- 1 分間に 3 回以上 restart が起きたら alert 発火、運用メール到達

### 原状回復
```bash
gcloud run services update ipo-api \
  --project=ipotto-stg --region=asia-northeast1 \
  --min-instances=0 --max-instances=5
```

---

## 3. ERROR log rate alert

### 目的
`IPOtto service ERROR logs` が log-based metric `ipotto/service_error` で 5 分間 ≥ 10 件を検知して通知する。

### 手順
```bash
# ipo-browser に意図的に壊れた JSON を POST して ERROR ログを誘発
IPO_BROWSER_URL="https://ipo-browser-xxxxx.a.run.app"
for i in $(seq 1 15); do
  curl -s -o /dev/null \
    -X POST "${IPO_BROWSER_URL}/internal/lottery-applications/submit" \
    -H 'content-type: application/json' \
    --data 'not json'
done
```

### 期待値
- ipo-browser が `routes.lottery_applications.submit_failed` で ERROR ログを出す (Sprint 12 で pino 化済)
- Log-based metric `ipotto/service_error` の件数が増加
- 5 分で 10 件を超えると alert 発火、運用メール到達

### 原状回復
- ログは Cloud Logging に残るため特に操作不要

---

## 4. Pub/Sub DLQ activity alert

### 目的
`IPOtto Pub/Sub DLQ activity` が log-based metric `ipotto/dlq_delivery_failed` で delivery warning を検知して通知する。

### 手順
```bash
# 壊れた envelope を ipo-notification topic に publish
gcloud pubsub topics publish ipo-notification \
  --project=ipotto-stg \
  --message='{"invalid_envelope": true}'

# ipo-api の /internal/pubsub/ipo-notification がこれを NACK し、
# max_delivery_attempts (5) を超えると DLQ に移動、その際に
# Pub/Sub 側が delivery warning ログを出す。
```

### 期待値
- `/internal/pubsub/ipo-notification` が 400 Bad Request を返す
- Pub/Sub が 5 回再配送 → `ipo-dead-letter` topic に移動
- Log-based metric `ipotto/dlq_delivery_failed` が非ゼロ
- Alert 発火、運用メール到達

### 原状回復
```bash
# DLQ topic の subscription をクリア (Sprint 12 で作成した ipo-dead-letter-sub)
# emulator ではなく本番なので、メッセージを ack するだけ
gcloud pubsub subscriptions pull ipo-dead-letter-sub \
  --project=ipotto-stg --auto-ack --limit=10
```

---

## 5. サインオフ

各テスト完了時に `release-checklist.md` §13.4 の該当行にチェックを入れる。すべて OK なら Go 判定へ進む。
