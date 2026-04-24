---
title: "野村證券対応 — 情報収集依頼書"
version: "1.0.0"
status: "awaiting-user"
created: "2026-04-24"
last_updated: "2026-04-24"
author: "lihs"
---

# 野村證券対応 — 情報収集依頼書

## 1. 概要

IPOtto は現状 Phase 1 MVP として **楽天証券のみ** に対応している。野村證券は IPO 引受幹事になることが多く、追加対応の優先度が最も高い証券会社の 1 社。野村證券アダプターの設計・実装に着手するため、開発者だけでは特定できないサイト構造・認証方式・利用規約の情報をユーザー（実利用口座保有者）から収集する。

本書の質問項目に回答が揃った段階で、以下の後続タスクが着手可能になる:

- `docs/03-detailed-design/nomura-broker-adapter.md` の確定（現状は仮構成）
- `docs/09-security-design/nomura-authentication.md` の脅威分析確定
- Phase 8 Sprint 16: ipo-browser に `nomura/` フローを実装
- `services/ipo-backend-shared/src/domain/account/securities_company.rs` に `Nomura` variant を追加し全 match の更新
- `docs/reference/nomura/` 配下に HTML / スクリーンショットを配置

## 2. 回答期限の目安

特に締切は設定しないが、Phase 8 着手判定（Sprint 16 開始）の前提条件となる。

## 3. 回答方法

以下のいずれか:

1. **本ファイル内に直接追記** — 各質問の「回答記入欄」セクションに記入し、対応する `- [ ]` を `- [x]` にする
2. **別ファイル添付** — HTML スナップショット・スクリーンショット等は `docs/reference/nomura/` 配下に配置し、本ファイルから相対リンクで参照
3. **口頭ヒアリング** — 開発者が聞き取って本ファイルに転記。回答者・日付を明記

## 4. 質問項目

### 4.1 野村證券のオンライントレード口座と権限

#### 質問

- [ ] **Q-N-001**: IPO 申込が可能な野村證券の契約種別を特定する
  - **目的**: 「ホームトレード」「野村ネット&コール」など複数のオンラインサービスがあり、それぞれで IPO 申込フローが異なる可能性があるため
  - **期待する成果物**: 利用予定のサービス名（例: ホームトレード）と、サービス選択後の IPO 申込可否
  - **想定回答例**: `ホームトレード（個人向けオンライントレード）でログインし、IPO 一覧から申込可能`

- [ ] **Q-N-002**: テスト用ダミー口座の有無
  - **目的**: 自動操作の動作検証で実口座を毎回操作するのは bot 検知・誤操作リスクが高いため
  - **期待する成果物**: ダミー口座の有無、ある場合はそのアクセス手順。無い場合は「実口座のみ・テスト時は read-only 操作に限定」のような運用方針の合意
  - **想定回答例**: `テスト用口座なし。HTML スナップショット + ローカルモックで E2E、本番接続テストは年に数回手動で実施`

- [ ] **Q-N-003**: 自動操作に関する利用規約上の制約
  - **目的**: 楽天證券は黙認的に許容されているが、野村證券の規約に「bot 利用禁止」の明文があると実装方針自体を見直す必要がある
  - **期待する成果物**: 該当規約の URL と該当条文の引用、または「規約上問題なし」の判定根拠
  - **想定回答例**: `https://www.nomura.co.jp/.../terms.html 第 X 条「自動化ツール…」に該当条文なし。利用は自己責任で許容`

#### 回答記入欄

```
（ここに回答を記入してください）
```

---

### 4.2 ログインフロー

#### 質問

- [ ] **Q-N-010**: ログインページの URL（PC ブラウザ向け）
  - **期待する成果物**: 完全な URL 文字列
  - **想定回答例**: `https://hometrade.nomura.co.jp/web/rmfIndexWebAction.do`

- [ ] **Q-N-011**: ログイン入力欄の DOM 構造
  - **期待する成果物**:
    - ログイン ID 入力欄の `id` / `name` / セレクタ
    - パスワード入力欄の `id` / `name` / セレクタ
    - ログインボタンのセレクタ
    - HTML スナップショットを `docs/reference/nomura/nomura-login.html` として配置

- [ ] **Q-N-012**: ログイン成功時のリダイレクト先 URL パターン
  - **目的**: ログイン成功判定に使う
  - **想定回答例**: `https://hometrade.nomura.co.jp/web/main.do` を含むパスへ遷移

- [ ] **Q-N-013**: ログイン失敗時のエラー表示
  - **期待する成果物**: エラーメッセージの文面 + DOM 上の表示位置（class / id）

#### 回答記入欄

```
（ここに回答を記入してください）
```

---

### 4.3 2FA / セキュリティ認証

#### 質問

- [ ] **Q-N-020**: 採用されている 2FA 方式
  - **目的**: 楽天証券はメール経由のキーワード + 画像 alt マッチングだが、野村證券は別方式の可能性が高い。方式によって追加実装の規模が変わる
  - **期待する成果物**: 以下の選択肢から該当するものを全て + その他特殊な方式があれば記述
    - SMS OTP（携帯番号宛にコード送信）
    - メール OTP（メール本文中の数字コード）
    - 物理トークン / アプリ認証コード（Google Authenticator 等）
    - 生体認証（PC 側の WebAuthn など）
    - 画像認証（楽天と類似）
    - 質問応答（秘密の質問）
    - その他: 自由記述

- [ ] **Q-N-021**: 2FA 画面の URL / DOM 構造
  - **期待する成果物**: 2FA 画面の URL + HTML スナップショット（`docs/reference/nomura/nomura-2fa.html`）

- [ ] **Q-N-022**: OTP 配信から入力までの猶予時間
  - **目的**: ipo-browser のタイムアウト設計に使用（楽天は `MAIL_OTP_TIMEOUT_MS=120000`）

- [ ] **Q-N-023**: OTP の取得経路と自動取得の可否
  - **期待する成果物**:
    - メール経由なら IMAP 接続情報（楽天と同じ MailCredential 構造で対応可）
    - SMS 経由なら SMS gateway や OTP 自動取得 API の有無
    - 物理トークン / アプリの場合は「ユーザー手動入力のみ」となる旨

- [ ] **Q-N-024**: 失敗時のロックアウト挙動・閾値
  - **目的**: 自動リトライの安全な上限を決めるため
  - **想定回答例**: `5 回連続失敗で 30 分ロック` 等

#### 回答記入欄

```
（ここに回答を記入してください）
```

---

### 4.4 IPO 銘柄一覧 / 詳細ページ

#### 質問

- [ ] **Q-N-030**: IPO 銘柄一覧ページの URL（ログイン後）
  - **想定回答例**: `https://hometrade.nomura.co.jp/web/ipoListAction.do`

- [ ] **Q-N-031**: 銘柄詳細ページの URL パターン
  - **想定回答例**: `?stockCode=4001` のようにクエリ文字列に銘柄コードが含まれる

- [ ] **Q-N-032**: 一覧 / 詳細の表示項目とその DOM 位置
  - **期待する成果物**:
    - 銘柄コード / 会社名 / 仮条件（価格帯）/ ブックビルディング期間 / 抽選日 / 上場日 / 主幹事 のセレクタ
    - HTML スナップショットを `docs/reference/nomura/nomura-ipo-list.html` および `nomura-ipo-detail.html` として配置

- [ ] **Q-N-033**: ページネーション・絞り込み UI の有無
  - **目的**: 取得時に loop が必要かどうかを判断

#### 回答記入欄

```
（ここに回答を記入してください）
```

---

### 4.5 IPO 申込フォーム

#### 質問

- [ ] **Q-N-040**: 申込フォームの URL
  - **想定回答例**: `https://hometrade.nomura.co.jp/web/ipoApplyAction.do?stockCode=4001`

- [ ] **Q-N-041**: 入力項目の DOM
  - **期待する成果物**:
    - 株数入力欄のセレクタ
    - 価格入力欄のセレクタ（成行・指値の選択肢があるか）
    - 取引パスワード or 別の認証要素のセレクタ
    - その他確認チェックボックス（規約同意等）のセレクタ
    - HTML スナップショットを `docs/reference/nomura/nomura-apply-form.html` として配置

- [ ] **Q-N-042**: 送信ボタンと確認ダイアログの構造
  - **目的**: 多段確認 UI（送信 → 確認画面 → 最終送信）の有無

- [ ] **Q-N-043**: 送信後の結果メッセージのバリエーション
  - **期待する成果物**: 想定される全ての結果文面と DOM 位置
    - 成功（例: 「お申込を受け付けました」）
    - 重複申込（例: 「すでにお申込済みです」）
    - 残高不足（例: 「お預り金が不足しています」）
    - 申込期間外
    - その他エラー
  - **目的**: `services/ipo-browser/src/flows/apply-result-translator.ts` の野村版を作るため

#### 回答記入欄

```
（ここに回答を記入してください）
```

---

### 4.6 抽選結果確認

#### 質問

- [ ] **Q-N-050**: 抽選結果ページの URL
  - **想定回答例**: `https://hometrade.nomura.co.jp/web/ipoResultListAction.do`

- [ ] **Q-N-051**: 結果表示項目の DOM
  - **期待する成果物**:
    - 銘柄識別子 / 結果（当選 / 落選 / 補欠）/ 購入意思確認期間 のセレクタ
    - HTML スナップショットを `docs/reference/nomura/nomura-lottery-result.html` として配置

- [ ] **Q-N-052**: 結果ステータスの言語表現
  - **期待する成果物**: 楽天証券では `当選 / 落選 / 補欠` を使用。野村證券で異なる表現があれば全パターン
  - **想定回答例**: `当選 / 落選 / 繰上当選 / 当選辞退` など

#### 回答記入欄

```
（ここに回答を記入してください）
```

---

### 4.7 レート制限・bot 検知対策

#### 質問

- [ ] **Q-N-060**: サイト側のレート制限の感触
  - **目的**: 連続アクセスでブロックされる閾値を把握
  - **想定回答例**: `1 分間に 30 回以上アクセスすると一時的に IP ブロック` 等

- [ ] **Q-N-061**: CAPTCHA / reCAPTCHA の有無
  - **目的**: 必要なら回避設計（手動介入 UI 等）の検討
  - **想定回答例**: `ログイン画面に reCAPTCHA v3 が設置されている`

- [ ] **Q-N-062**: User-Agent や Cookie の制約
  - **目的**: Playwright のデフォルト UA でブロックされないかの事前確認

#### 回答記入欄

```
（ここに回答を記入してください）
```

---

### 4.8 credential の構造

#### 質問

- [ ] **Q-N-070**: ログイン ID の形式
  - **想定回答例**: `8 桁の数字のみ` / `メールアドレス可` / `任意の英数字` 等

- [ ] **Q-N-071**: パスワードの最小文字数・許可文字種
  - **目的**: フロントエンドの入力バリデーション設計に使用

- [ ] **Q-N-072**: 取引暗証番号の有無と形式
  - **目的**: `AccountCredential` 構造を可変化する設計判断に直接影響
  - **想定回答例**: `4 桁の数字 PIN（楽天と同様）` / `不要・ログインパスワードのみ` / `ハードウェアトークンの 6 桁コード` 等

- [ ] **Q-N-073**: その他必要な認証要素
  - **期待する成果物**: 楽天 `MailCredential` のような追加 credential の有無

#### 回答記入欄

```
（ここに回答を記入してください）
```

---

### 4.9 提出形式（再掲）

回答時に以下の参照資料を併せて提出してください:

- [ ] **HTML スナップショット**: `docs/reference/nomura/<画面名>.html`
  - 命名例: `nomura-login.html`, `nomura-2fa.html`, `nomura-ipo-list.html`, `nomura-ipo-detail.html`, `nomura-apply-form.html`, `nomura-lottery-result.html`
  - 取得方法: ブラウザの「ページを保存」または DevTools でのソース表示をテキスト保存
  - **個人情報・取引情報は伏字に**: 口座番号 / 氏名 / 残高表示等は事前にマスクする

- [ ] **スクリーンショット**: `docs/reference/nomura/screenshots/<画面名>.png`
  - 命名例: `nomura-login.png`, `nomura-2fa-step1.png` 等
  - こちらも個人情報のマスクを行うこと

## 5. 後続タスク

回答完了後、以下のタスクを順次実施する:

| タスク | 関連ドキュメント | 担当層 |
|---|---|---|
| `securities_company.rs` の `Nomura` variant 追加 | `docs/03-detailed-design/domain.md` | ipo-backend-shared |
| `AccountCredential` の broker 別構造の確定（必要なら可変化） | `docs/03-detailed-design/nomura-broker-adapter.md` | ipo-backend-shared |
| `ipo-browser` に `flows/nomura/` ディレクトリを新設 | `docs/03-detailed-design/nomura-broker-adapter.md` | ipo-browser |
| `selectors.yaml` に `nomura:` namespace を追加 | 同上 | ipo-browser |
| 認証フロー実装（2FA 方式に応じて） | `docs/09-security-design/nomura-authentication.md` | ipo-browser |
| HTML mock の追加（`html-mock-server` に `nomura/` 配下） | `docs/07-test-specification/test-specification.md` | テスト基盤 |
| E2E シナリオの追加 | 同上 | ipo-browser, ipo-frontend |
| 証券口座登録 UI への「証券会社選択」追加 | `docs/06-ui-ux-design/ui-ux-design.md` | ipo-frontend |
| Phase 8 ロードマップ詳細化 | `docs/11-implementation-roadmap/README.md` | プロジェクト管理 |

## 6. 更新履歴

| 日付 | 内容 |
|---|---|
| 2026-04-24 | 初版作成 |
