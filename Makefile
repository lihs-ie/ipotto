.DEFAULT_GOAL := help

# ============================================================
# Docker
# ============================================================

.PHONY: up down up-infra up-api up-frontend up-browser build ps logs clean

up: ## 全サービス起動
	docker compose up -d

down: ## 全サービス停止
	docker compose down

up-infra: ## エミュレータのみ起動（Firebase, Pub/Sub, HTMLモック）
	docker compose up -d firebase-emulator pubsub-emulator html-mock-server

up-api: up-infra ## ipo-api + エミュレータ起動
	docker compose up -d ipo-api

up-frontend: up-api ## ipo-frontend + API + エミュレータ起動
	docker compose up -d ipo-frontend

up-browser: up-infra ## ipo-browser + エミュレータ起動
	docker compose up -d ipo-browser

build: ## 全Dockerイメージ再ビルド
	docker compose build

ps: ## 稼働中サービス一覧
	docker compose ps

logs: ## 全サービスログ表示
	docker compose logs -f

clean: down ## 停止 + ボリューム・イメージ削除
	docker compose down --volumes --rmi local

# ============================================================
# Setup
# ============================================================

.PHONY: setup setup-pubsub setup-hooks install

setup: install setup-hooks up-infra setup-pubsub ## ローカル開発環境セットアップ
	@echo "Setup complete. Run 'make up' to start all services."

setup-pubsub: ## Pub/Subエミュレータのトピック・サブスクリプション作成
	bash scripts/setup-pubsub-emulator.sh

setup-hooks: ## Git hooks（lefthook）インストール
	lefthook install

install: ## 全Node.jsサービスの依存関係インストール
	cd services/ipo-browser && pnpm install
	cd services/ipo-frontend && pnpm install

# ============================================================
# Rust (services/)
# ============================================================

.PHONY: rust-build rust-test rust-lint rust-fmt rust-fmt-check rust-coverage rust-ci

rust-build: ## Rustワークスペース全体ビルド
	cd services && cargo build

rust-test: ## Rust全テスト実行
	cd services && cargo test --all

rust-lint: ## Rust Clippy実行
	cd services && cargo clippy --all-targets -- -D warnings

rust-fmt: ## Rustコード自動フォーマット
	cd services && cargo fmt --all

rust-fmt-check: ## Rustフォーマットチェック
	cd services && cargo fmt --all -- --check

rust-coverage: ## Rustカバレッジレポート生成
	cd services && cargo llvm-cov --all

rust-ci: rust-fmt-check rust-lint rust-test ## Rust CI相当（直接実行。actrun版: make ci-rust）

# ============================================================
# ipo-browser (services/ipo-browser/)
# ============================================================

.PHONY: browser-dev browser-test browser-e2e browser-lint browser-typecheck

browser-dev: ## ipo-browser開発サーバー起動
	cd services/ipo-browser && pnpm dev

browser-test: ## ipo-browserユニットテスト
	cd services/ipo-browser && pnpm test --run --passWithNoTests

browser-e2e: ## ipo-browser E2Eテスト（HTMLモックサーバー必要）
	cd services/ipo-browser && MOCK_SERVER_URL=http://localhost:8090 IPO_BROWSER_URL=http://localhost:8081 pnpm e2e

browser-lint: ## ipo-browser Lint
	cd services/ipo-browser && pnpm lint

browser-typecheck: ## ipo-browser型チェック
	cd services/ipo-browser && pnpm tsc --noEmit

# ============================================================
# ipo-frontend (services/ipo-frontend/)
# ============================================================

.PHONY: frontend-dev frontend-test frontend-e2e frontend-lint frontend-typecheck

frontend-dev: ## ipo-frontend開発サーバー起動
	cd services/ipo-frontend && pnpm dev

frontend-test: ## ipo-frontendユニットテスト
	cd services/ipo-frontend && pnpm test --run --passWithNoTests

frontend-e2e: ## ipo-frontend E2Eテスト（Docker起動必要）
	cd services/ipo-frontend && FRONTEND_URL=http://localhost:3000 pnpm e2e

frontend-lint: ## ipo-frontend Lint
	cd services/ipo-frontend && pnpm lint

frontend-typecheck: ## ipo-frontend型チェック
	cd services/ipo-frontend && pnpm tsc --noEmit

# ============================================================
# Node.js 横断
# ============================================================

.PHONY: node-test node-lint node-typecheck node-ci

node-test: browser-test frontend-test ## 全Node.jsサービスのユニットテスト

node-lint: browser-lint frontend-lint ## 全Node.jsサービスのLint

node-typecheck: browser-typecheck frontend-typecheck ## 全Node.jsサービスの型チェック

node-ci: node-lint node-typecheck node-test ## Node.js CI相当（直接実行。actrun版: make ci-node）

# ============================================================
# CI（actrun経由 — 設定はactrun.tomlで管理）
# ============================================================

.PHONY: ci ci-rust ci-node ci-e2e ci-e2e-browser ci-e2e-frontend ci-dry-run ci-lint

ci: ## CI全チェック一括実行（actrun: ci.yml + e2e.yml）
	actrun workflow run .github/workflows/ci.yml
	actrun workflow run .github/workflows/e2e.yml

ci-rust: ## Rust CIジョブのみ実行（actrun）
	actrun workflow run .github/workflows/ci.yml --job rust-lint-test

ci-node: ## Node.js CIジョブのみ実行（actrun）
	actrun workflow run .github/workflows/ci.yml --job node-lint-test

ci-e2e: ## E2E全ジョブ実行（actrun: e2e.yml）
	actrun workflow run .github/workflows/e2e.yml

ci-e2e-browser: ## E2E ipo-browserのみ実行（actrun）
	actrun workflow run .github/workflows/e2e.yml --job e2e-browser

ci-e2e-frontend: ## E2E ipo-frontendのみ実行（actrun）
	actrun workflow run .github/workflows/e2e.yml --job e2e-frontend

ci-dry-run: ## CI dry-run（実行計画のみ表示）
	actrun workflow run .github/workflows/ci.yml --dry-run

ci-lint: ## 全ワークフローのlint
	actrun lint .github/workflows/ci.yml .github/workflows/e2e.yml

# ============================================================
# Help
# ============================================================

.PHONY: help

help: ## このヘルプを表示
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
