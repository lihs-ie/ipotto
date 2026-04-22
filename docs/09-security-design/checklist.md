# セキュリティチェックリスト

> 作成: 2026-04-22 (Phase 6 Sprint 12)
> 併読: [security-design.md](./security-design.md) / [test-specification](../07-test-specification/README.md)

IPOtto の本番運用開始 (Sprint 13) に先立って、`docs/09-security-design/security-design.md` の STRIDE 脅威モデル + OWASP Top 10 を実装に照らして棚卸しした結果。各項目は **現在の実装** での対応状況を明記している。

本チェックリストは Sprint 13 デプロイ前レビュー / 四半期ごとのセキュリティレビュー で追跡する。

---

## 1. 認証 / 認可

### 1.1 Firebase IDToken 検証
- [x] **実装済** (`services/ipo-api/src/middleware/firebase_token_verifier.rs`)
- RS256 署名検証 (production) / `alg:none` (emulator mode only)
- `iss` = `https://securetoken.google.com/{FIREBASE_PROJECT_ID}` を exact match
- `aud` = `FIREBASE_PROJECT_ID` を exact match
- `exp` / `iat` / `sub` (uid) 必須
- Leeway 0 秒 (厳密に expiration を enforce)
- **STRIDE 対応**: Spoofing (S-1)

### 1.2 ALLOWED_EMAIL 許可リスト
- [x] **実装済** (`services/ipo-api/src/middleware/email_allowlist.rs`)
- `ALLOWED_EMAIL` env はカンマ区切り、正規化 (lowercase + trim)
- 未登録メールは 403 Forbidden + `EMAIL_NOT_ALLOWED`
- 起動時に最低 1 件必須 (空だとクラッシュ)
- **STRIDE 対応**: Information Disclosure (I-1)

### 1.3 Firebase Auth Console の 2FA
- [ ] **運用側 TODO** — GCP console で Firebase Admin 権限を持つアカウントに 2FA を必須化 (本番リリース直前に手動設定)

### 1.4 Google Sign-In provider のみ許可
- [ ] **未実装** — バックエンドは provider 情報を検証していない (どの Firebase provider でも通る)。許可リスト + emulator フラグで実害は抑えられているが、将来的に `firebase.sign_in_provider == "google.com"` を claim から強制することを推奨。

---

## 2. シークレット管理

### 2.1 証券口座認証情報の Secret Manager 保存
- [x] **実装済** (`services/ipo-backend-shared/src/infrastructure/secrets/google_secret_manager_store.rs`)
- Production は `GoogleSecretManagerStore` (ADC / workload identity)
- Docker Compose (local) は `InMemoryCredentialStore` + `CREDENTIAL_STORE_BACKEND=in-memory`
- 暗号化は GCP 側で AES-256 自動
- **STRIDE 対応**: Tampering (T-2) / Information Disclosure (I-2)

### 2.2 Secret Manager へのアクセス権限
- [x] **実装済** (`terraform/environments/{stg,prd}/terraform.tfvars`)
- `roles/secretmanager.secretAccessor` を `ipo-api` / `ipo-browser` / `ipo-result-checker` に限定付与
- Scheduler / info-fetcher には付与しない (不要)

### 2.3 環境変数経由の秘匿情報
- [x] **実装済** — `ALLOWED_EMAIL`, `SENDGRID_API_KEY` 等は Cloud Run の Secret 環境変数で注入 (terraform `cloud_run_services[*].secret_environment_variables`)
- プレーンな env で秘匿情報を渡していないことを確認 (`terraform.tfvars` で `secret_environment_variables` を活用)

---

## 3. 入力バリデーション

### 3.1 API 入力検証 (Rust)
- [x] **実装済** — 各 Use Case で入力 DTO を検証し `DomainError::ValidationError` で 400 を返す
- CompanyName / TickerSymbol / MailAddress 等の値オブジェクトがドメイン層で範囲検査
- **STRIDE 対応**: Tampering (T-1)

### 3.2 API 入力検証 (Frontend)
- [x] **実装済** — `ipo-frontend` の form molecules が Zod (`validateWithZod`) で preflight validation、サーバー側は `apiErrorSchema` で details を受け取って描画

### 3.3 HTML / URL エスケープ (XSS)
- [x] **実装済** — React + Next.js の自動エスケープ、`dangerouslySetInnerHTML` 未使用。`grep -rn dangerouslySetInnerHTML services/ipo-frontend/` で 0 件確認
- **OWASP A03 対応**

### 3.4 SQL Injection
- [x] **該当なし** — 永続化は Firestore (NoSQL) のみ。SQL 文字列組立てなし

### 3.5 CSRF
- [x] **実装済** — 認証は Bearer IDToken のみ、Cookie 非依存なので CSRF 攻撃面なし

---

## 4. 通信セキュリティ

### 4.1 HTTPS 強制
- [x] **Cloud Run で自動 HTTPS** (terraform で `ingress = "internal"` / `"all"` 設定時も HTTPS のみ)
- Cloud Run への内部通信も HTTPS (Google Cloud のデフォルト)

### 4.2 TLS 証明書
- [x] **実装済** — Cloud Run が Google-managed cert を自動発行/更新

### 4.3 外部 API 呼出しの TLS
- [x] **実装済** — reqwest に rustls 固定 (`features = ["rustls"]`)、TLS 検証は default enabled

---

## 5. ログと監査

### 5.1 構造化ログ (JSON)
- [x] **実装済**
  - Rust: `services/ipo-backend-shared/src/http/service_runner.rs` で tracing-subscriber fmt.json
  - Node: `services/ipo-browser/src/logger.ts` で pino (Sprint 12 で対応)
- Cloud Logging が自動で `severity` / `timestamp` を解析

### 5.2 機密情報のログ redaction
- [x] **実装済**
  - Rust: `services/ipo-backend-shared/src/logging/secret_masking.rs` (`mask_prefix`, `mask_email`, `mask_secret`)
  - Node: pino `redact` で `loginPassword` / `tradingPassword` / `mailPassword` / `authorization` を自動 `[Redacted]`
- **OWASP A09 対応**

### 5.3 リクエストログの request_id / uid / email
- [x] **実装済** (`services/ipo-api/src/middleware/request_logging.rs`)
- 全リクエストに `info_span!("http_request", request_id, method, path, uid, email)` を付与
- email は `mask_email` 済み

### 5.4 アラート通知
- [x] **実装済** (`terraform/modules/observability/main.tf`, Sprint 12)
- 5xx 急増 / コンテナ再起動 / ERROR ログ流量 / Pub/Sub DLQ 活動の 4 種
- 通知先はメール (`alert_email_recipients`)

### 5.5 監査ログ (Cloud Audit Logs)
- [ ] **default 設定のまま** — GCP Admin Activity ログは自動だが、Data Access ログは未有効化。秘密情報へのアクセス追跡が必要なら Sprint 13 以降で検討

---

## 6. 依存ライブラリと CI

### 6.1 Rust 依存脆弱性
- [x] **実装済** — `.github/workflows/security-scan.yml` で `cargo audit` を毎週日曜実行

### 6.2 Node 依存脆弱性
- [x] **実装済** — `.github/workflows/security-scan.yml` で `pnpm audit` を毎週日曜実行

### 6.3 Docker image の脆弱性スキャン
- [ ] **未実装** — container layer に対する trivy / grype 等のスキャンは Sprint 13 以降で検討
- 当面は base image (node:24-bookworm-slim, rust:1.94.1-slim) のパッチ適用を月次で行う運用で代替

### 6.4 依存バージョンの固定
- [x] **実装済** — `Cargo.lock` と `pnpm-lock.yaml` を git 管理、`--frozen-lockfile` で CI 強制

---

## 7. レート制限とアビュース対策

### 7.1 HTTP レート制限
- [ ] **未実装** — 現在 Cloud Run + Firebase Auth の基本機能のみ。allowlist で攻撃面を縮小しているが、DDoS 防御としては不十分
- 将来的に `tower::ServiceBuilder` の `rate_limit` ミドルウェア、または Cloud Armor を検討

### 7.2 ブラウザ自動化のレート制限
- [x] **実装済** — `services/ipo-browser` は per-request で 1 Playwright context、並列数は Cloud Run の concurrency (1 req/instance) で抑制

### 7.3 リトライのエクスポネンシャルバックオフ
- [x] **実装済** (`services/ipo-backend-shared/src/infrastructure/http_client/retry_runner.rs`)
- 初期 1s → 最大 30s、jitter ±10%、最大 3 回

---

## 8. データ保護

### 8.1 Firestore セキュリティルール
- [ ] **default (reject-all) のまま** — クライアント直接アクセスは使わず、全て ipo-api 経由のため実害なし。ただし Firebase console で明示的に deny-all を宣言するとより防御的
- 対応: Sprint 13 でルールファイル (`firestore.rules`) を追加

### 8.2 Pub/Sub の暗号化
- [x] **デフォルト有効** — Pub/Sub メッセージは GCP 側で AES-256 at-rest
- [ ] **CMEK (customer-managed encryption key)** は未設定。要件次第で Sprint 13 以降で検討

### 8.3 Firestore のバックアップ
- [ ] **未実装** — Sprint 13 で Cloud Scheduler + Export ジョブを terraform 化する

### 8.4 PII 収集の最小化
- [x] **実装済** — アプリが保持する PII は `email` のみ (Firebase Auth 経由)。証券口座認証情報は Secret Manager で暗号化保管、平文でログに出さない

---

## 9. STRIDE トレース (security-design.md §3 対応表)

| STRIDE | 脅威 | 対策項目 | チェック |
|---|---|---|---|
| S-1 | Firebase IDToken なりすまし | 1.1 RS256 検証 + 1.2 allowlist | ✅ |
| S-2 | 証券口座資格情報なりすまし | 2.1 Secret Manager + 5.2 redaction | ✅ |
| T-1 | 不正な入力による状態改竄 | 3.1 / 3.2 入力検証 | ✅ |
| T-2 | シークレットのトランジットでの改竄 | 4.1 / 4.3 TLS | ✅ |
| R-1 | ログ改竄 | 5.1 Cloud Logging (append-only) | ✅ |
| I-1 | 認可外ユーザーへの情報露出 | 1.2 allowlist + 8.4 PII 最小化 | ✅ |
| I-2 | シークレット漏洩 | 2.1 Secret Manager + 5.2 redaction | ✅ |
| D-1 | DoS / レート超過 | 7.1 TODO / 7.2 browser concurrency | ⚠️ 部分的 |
| E-1 | 権限昇格 | 1.1 token + 1.2 allowlist + GCP IAM | ✅ |

⚠️ の項目は Sprint 13 以降の継続課題として運用設計書 (`docs/10-operations-design`) に繰り越す。

---

## 10. Sprint 13 持ち越し

- [ ] Firebase Auth console 2FA (運用手順) — §1.3
- [ ] Google provider 強制 — §1.4
- [ ] Cloud Audit Data Access logs 有効化 — §5.5
- [ ] Docker image 脆弱性スキャン (trivy) CI 組込 — §6.3
- [ ] HTTP レート制限 / Cloud Armor — §7.1
- [ ] Firestore security rules `reject-all` 宣言 — §8.1
- [ ] Pub/Sub CMEK 検討 — §8.2
- [ ] Firestore 自動バックアップ — §8.3

Sprint 13 の "本番リリース" チェックリスト開始時に、本セクションを再レビューして deployment-ready の判定を行う。
