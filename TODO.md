# TODO

## DD-101 `ipo-browser` を 100% にするための残タスク

### 1. Workflow 完了条件

- [x] 1口座・1銘柄の失敗で batch 全体を止めないようにする
  - [x] 口座単位で処理継続できることを実装する
  - [x] 銘柄単位で処理継続できることを実装する
  - [x] 全件失敗と部分失敗を区別できるようにする
- [x] 口座単位、銘柄単位、全体単位の集計を分けて返す
  - [x] 口座ごとの成功件数、失敗件数、スキップ件数を返せるようにする
  - [x] 銘柄ごとの成功件数、失敗件数、スキップ件数を返せるようにする
  - [x] 全体集計との整合性を test で固定する
- [x] `appliedCount`、`skippedCount`、`failedCount` の定義を明文化する
  - [x] `already_applied` は `skippedCount` に含める
  - [x] `excluded` は `skippedCount` に含める
  - [x] `insufficient_balance` は `failedCount` に含める
  - [x] `mail_retrieval` は `failedCount` に含める
  - [x] `image_authentication` は `failedCount` に含める
- [x] `OperationErrorOccurred` を「全体停止エラー」だけでなく「継続したが失敗した個別処理」にも使うかどうかを決める
  - [x] 採用する場合は発火条件を明文化する（今回は非採用のため不要）
  - [x] 採用しない方針とし、`ApplicationFailed` / `ImageAuthenticationFailed` / `OperationLog` で扱う
- [x] retry するケースと retry しないケースを固定する
  - [x] 認証コード失効: 1回だけ再送
  - [x] IMAP timeout: retry なし
  - [x] selector not found: retry なし
  - [x] 楽天側一時エラー: retry なしで固定する

### 2. Failure 分類の完成

- [x] `ApplicationFailed` と `ImageAuthenticationFailed` の payload を最終確定する
  - [x] 必須項目を固定する
  - [x] 内部向け情報と外部通知向け情報を分離する
- [x] `mail_retrieval`、`image_authentication`、`application` の 3分類で十分か再確認する
  - [x] `login_failed` を `application` に含める
  - [x] `2FA page not reached` を `application` に含める
  - [x] `unexpected page transition` を `application` に含める
- [x] `insufficient_balance` を `ApplicationFailed` の `result_status` で表現する方針を最終化する
- [x] `already_applied` は event を出さない方針で確定する
- [x] 例外文言をそのまま外部通知に出さないようにする
  - [x] 内部向け `error_message` を定義する
  - [x] 外部向け `message` を定義する
  - [x] browser / IMAP / Secret の raw error をそのまま出さないことを test で固定する

### 3. OperationLog の完成

- [x] `eventType` を文字列ベタ書きではなく定数か union に寄せる
- [x] `message` を category ごとに固定文へ寄せる
  - [x] success
  - [x] already_applied
  - [x] insufficient_balance
  - [x] image_authentication
  - [x] mail_retrieval
  - [x] application
- [x] `errorMessage` は raw error 全文ではなく、保存してよい内容だけに制限する
- [x] `application` に ULID を入れる条件を明確にする
  - [x] 成功時のみ
  - [x] 失敗時は `null`
- [x] `executedAt` の取得タイミングを統一する
  - [x] browser 実行前（採用しない）
  - [x] result 判定時
  - [x] event publish 時（採用しない）
  - [x] どの時点を採用するか決めて docs と test に反映する
- [x] skip を `Success` として記録する現方針を docs と test に反映する

### 4. Browser Automation の完成

- [x] 楽天ログイン画面の selector を実 HTML ベースで最終固定する
- [x] ダッシュボード到達判定を `title` 依存だけでなく DOM marker でも二重化する
- [x] IPO一覧から対象銘柄を選ぶ導線で、同名銘柄や複数ステータス表示時の優先順位を決める
- [x] 注意事項画面、入力画面、確認画面、完了画面の各ページ遷移に `waitForURL` または DOM marker を入れて安定化する
- [x] `passwordInputText`、`orderValueInput`、`priceSpinnerComBox` が消えた時の fallback selector を用意する
- [x] 残高不足画面とその他失敗画面の detector を完成させる
  - [x] 対象画面が取得不能な場合は mock fixture で代替可とする
  - [x] 残高不足は mock fixture ベースで判定を固定する
  - [x] その他失敗画面は mock fixture ベースで判定を固定する
- [x] `RAKUTEN_DRY_RUN` と本番挙動の分岐を整理する
  - [x] dry-run 専用コードを test fixture 用に閉じ込める
  - [x] production path と fixture path の責務を分離する

### 5. Mail Retrieval の完成

- [x] IMAP 接続失敗、認証失敗、検索0件、本文 parse 失敗を個別に test で固定する
- [x] `receivedAfter` の扱いを秒単位で見直し、同一時刻の古いメールを拾わないことを保証する
- [x] `from` と `subject` の filter を実運用のメールパターンに合わせて見直す
- [x] `multipart mail`、`quoted-printable`、`base64` 本文の対応有無を確認する
- [x] Gmail API をサポートするか、今回は IMAP 専用とするかを決める
  - [x] DD-101 は IMAP 専用で固定する
  - [x] Gmail API はこのトランシェの対象外とする
- [x] IMAP credential が欠損している口座を apply 対象からどう除外するか決める
  - [x] 欠損口座は apply 対象から除外する
  - [x] 実行可能口座が 0 件なら `OperationErrorOccurred` を publish する

### 6. Security / Secret Handling

- [x] Secret Manager から取得した credential を log に出さないことを再確認する
- [x] account credential と mail credential を error message に含めないようにする
- [x] session directory に機密データが残り続けないように cleanup 方針を決める
  - [x] 24時間超過分を削除する
  - [x] 起動時に cleanup を実行する
- [x] Playwright persistent context の保存先と retention を決める
  - [x] `BROWSER_SESSION_BASE_DIR` 配下に保存する
  - [x] `BROWSER_SESSION_RETENTION_HOURS` で retention を制御する
- [x] reference HTML や実画面 dump を Git に乗せない運用を docs 化する

### 7. Service 契約の完成

- [x] `/internal/pubsub/apply` の input schema を固定する
  - [x] `targetDate` 必須
  - [x] 将来 `accountIds` や `stockIds` を許可するか決める
    - [x] このトランシェでは未対応として 400 を返す
- [x] response schema を固定する
  - [x] `results[].result`
  - [x] `results[].reason`
  - [x] `results[].failureCategory`
- [x] handler の 4xx/5xx 方針を決める
  - [x] input 不正は `400`
  - [x] broker 一部失敗でも workflow 完了なら `200`
  - [x] 全件失敗でも workflow 完了なら `200`
- [x] Pub/Sub push retry と handler の status code の関係を明確にする
  - [x] 部分失敗でも `200` にする
  - [x] 全体停止時だけ `503` にする

### 8. Cross-service 接続の完成

- [x] `ipo-browser` が publish する `ApplicationCompleted` を `ipo-api` が受け取れることを integration test で確認する
- [x] `ApplicationFailed` と `ImageAuthenticationFailed` の payload 契約を `ipo-api` 側 consumer test で固定する
- [x] shared 側の event 契約と `ipo-browser` の event shape の差異がないか見直す
- [x] topic 名、service name、aggregate type を shared 方針に合わせて統一する
- [x] ULID と UUID の使い分けを最終確認する
  - [x] application / account / stock: ULID
  - [x] messageId / correlationId: UUIDv4

### 9. Test の完成

- [x] use case test に「複数口座、複数銘柄、部分失敗」を追加する
- [x] `RakutenBrokerAdapter` test に login failure、unexpected page、selector missing を追加する
- [x] `ImageAuthenticationPage` test に resend 不可ケースを追加する
- [x] `ImapRakutenAuthMailSource` test に IMAP error path を追加する
- [x] handler test に partial failure response を追加する
- [x] `pnpm type-check`、`pnpm lint`、`pnpm test --run` に加えて、本番相当 env を使う smoke test を足す

### 10. Feature test の作成

- [x] モックを利用しないシステム・サービス単位の test を作成する
  - [x] `ipo-browser` 単体の feature test を作る
  - [x] 可能なら `ipo-api` との接続まで含む end-to-end に近い test を作る
  - [x] Secret / Firestore / PubSub をどう代替するか方針を決める
