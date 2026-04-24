# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

IPOtto - 個人投資家向けIPO抽選申し込み自動化・管理Webアプリケーション。複数の証券会社での手動申し込み作業を自動化する。Phase 1 MVPは楽天証券のみ対象。

## アーキテクチャ

マイクロサービス構成（6サービス）、GCP Cloud Run上で稼働。サービス間連携はGoogle Cloud Pub/Subによるイベント駆動。

| サービス | 技術 | 役割 |
|----------|------|------|
| ipo-frontend | Next.js 16 / TypeScript / CSS Modules | ダッシュボードUI |
| ipo-api | Rust / Axum | REST API・ビジネスロジック・Firestore操作 |
| ipo-browser | Node.js / Playwright | 楽天証券Webサイト自動操作（2FA対応含む） |
| ipo-info-fetcher | Rust | IPO情報スクレイピング |
| ipo-result-checker | Rust | 抽選結果確認・通知 |
| ipo-applier | Rust | 抽選申込の自動実行（Pub/Sub駆動） |

データストア: Firebase Firestore、認証: Firebase Authentication、スケジューリング: Cloud Scheduler → Pub/Sub

## ドメインモデル（DDD）

4つの境界づけられたコンテキスト:
- **IPO銘柄コンテキスト** - 銘柄情報管理（IpoStock, Exclusion）
- **抽選申込コンテキスト** - 申込実行・結果管理（LotteryApplication, LotteryResult）
- **通知コンテキスト** - イベント通知（NotificationChannel: LINE/Email/Slack）
- **証券口座コンテキスト** - 認証情報管理（SecuritiesAccount）

ステータスフロー: 情報取得済 → 申込可能 → 申込済 → 当選/落選/補欠 → 購入済 → 売却済

## アーキテクチャパターン

### Rustサービス — レイヤードアーキテクチャ

```
presentation/    ← HTTPハンドラ・ルーティング
application/     ← ユースケース・DTO
domain/          ← ドメインモデル・ポート（trait）
infrastructure/  ← 永続化・外部サービス実装
```

依存方向: presentation → application → domain ← infrastructure

### TypeScript/Node.jsサービス — オニオンアーキテクチャ

```
domain/          ← エンティティ・値オブジェクト・ポート（最内層、依存なし）
application/     ← ユースケース・ハンドラ（domainのみに依存）
infrastructure/  ← 外部サービス実装・アダプター（最外層）
```

依存方向: infrastructure → application → domain（内側への依存のみ）

### 共通パターン
- **Repositoryパターン** - データアクセス抽象化
- **Adapterパターン** - 証券会社・通知チャネルの追加を既存コード変更なしで実現

## プロジェクト構成

```
services/
├── Cargo.toml              # Rust workspace root
├── pnpm-workspace.yaml     # Node.js workspace root
├── rust-toolchain.toml     # Rustツールチェーン固定（1.94.1）
├── ipo-backend-shared/     # バックエンド共通ドメインモデル (Rust crate)
├── ipo-api/                # REST API (Rust/Axum, port 8080)
├── ipo-info-fetcher/       # IPO情報スクレイピング (Rust, port 8082)
├── ipo-result-checker/     # 抽選結果確認 (Rust, port 8083)
├── ipo-applier/            # 抽選申込の自動実行 (Rust, port 8084)
├── ipo-frontend-shared/    # フロントエンド共通ドメインモデル (@ipotto/shared)
├── ipo-browser/            # ブラウザ自動化 (Node.js/Playwright, port 8081)
└── ipo-frontend/           # ダッシュボードUI (Next.js 16, port 3000)
```

## ローカル開発

### 初回セットアップ
```bash
brew install lefthook       # Git hookツール（未インストールの場合）
make setup                  # 依存インストール + hooks設定 + エミュレータ起動 + Pub/Subセットアップ
```

### Makefileコマンド（主要）
```bash
make help                   # 全コマンド一覧
make up                     # 全サービス起動（Docker）
make down                   # 停止
make seed                   # ローカル Firestore Emulator にテストデータ投入
make seed-reset             # Firestore を空にしてから seed 投入
make ci                     # CI全チェック実行（actrun経由）
make ci-rust                # Rust CIのみ
make ci-node                # Node.js CIのみ
make ci-e2e-browser         # E2E ipo-browserのみ
make ci-e2e-frontend        # E2E ipo-frontendのみ
make ci-lint                # ワークフローlint
```

### Rust サービス（Cargo workspace: services/）
```bash
cd services
cargo build                    # 全サービスビルド
cargo test --all               # 全テスト実行
cargo test -p ipo-api          # 単一サービステスト
cargo test <test_name>         # 単一テスト実行
cargo clippy --all-targets     # Lint
cargo fmt --all -- --check     # フォーマットチェック
cargo llvm-cov --all           # カバレッジ
```

### Next.js フロントエンド（services/ipo-frontend/）
```bash
pnpm install                   # 依存関係インストール
pnpm dev                       # 開発サーバー起動
pnpm test                      # テスト実行
pnpm test <test_file>          # 単一テスト実行
pnpm e2e                       # E2Eテスト
pnpm lint                      # Lint
```

### Node.js ブラウザ自動化（services/ipo-browser/）
```bash
pnpm install                   # 依存関係インストール
pnpm test                      # テスト実行
pnpm e2e                       # E2Eテスト
pnpm lint                      # Lint
```

### インフラ
```bash
terraform init                 # Terraform初期化
terraform plan                 # 変更プレビュー
terraform apply                # 適用
```

## CI/CD

GitHub Actionsワークフロー:
- **ci.yml** - Lint + テスト + カバレッジ（全ブランチpush / mainへのPR）
- **e2e.yml** - E2Eテスト（mainへのPR）
- **build-and-deploy.yml** - Docker build + Cloud Runデプロイ（mainへのpush）
- **security-scan.yml** - cargo audit + pnpm audit（毎週日曜）

ローカルでのCI実行はactrun経由: `make ci`（設定は`actrun.toml`で管理）

## Git hooks（lefthook）

- **pre-commit**: `cargo fmt --check` + `pnpm lint`（各サービス並列実行）
- **pre-push**: `make ci`（CI全チェック）

## API仕様

ベースパス: `/api/v1`、認証: Firebase IDToken（Bearerトークン）

主要エンドポイント: `/stocks`, `/dashboard`, `/exclusions`, `/accounts`, `/settings/notifications`, `/logs`

詳細は `docs/04-api-specification/` 参照。

## セキュリティ上の注意点

- 証券口座の認証情報はアプリ層で envelope 暗号化（AES-256-GCM + Cloud KMS wrapped DEK）を施した上で GCP Secret Manager に保存する（多層防御）
  - 実装: `services/ipo-backend-shared/src/infrastructure/crypto/` の `EncryptedCredentialStore` / `GoogleKmsKeyManagement`
  - ローカル開発は `KEY_MANAGEMENT_BACKEND=in-memory` で `InMemoryKeyManagement` を使用（KMS への疎通不要）
  - KEK は Terraform `modules/kms/` で作成、90 日自動ローテーション
- メモリ上の認証情報は最小限の保持時間とする。DEK は `zeroize::Zeroizing` で Drop 時に確実にクリア
- 2FA対応: 楽天証券の画像認証はメール経由でキーワード抽出→画像alt属性マッチング
- ブラウザ自動操作はレート制限を設けてbot検知を回避
- 許可メールアドレスのホワイトリストによるアクセス制御

## 設計ドキュメント

`docs/` ディレクトリに包括的な設計ドキュメントが格納されている:
- `01-requirements/` - 要件定義・ユースケース
- `02-system-design/` - システムアーキテクチャ
- `03-detailed-design/` - 詳細設計・DDD・クラス設計
- `04-api-specification/` - REST API仕様
- `07-test-specification/` - テスト戦略
- `08-infrastructure-design/` - GCPインフラ（Terraform）
- `09-security-design/` - セキュリティ設計・脅威分析
