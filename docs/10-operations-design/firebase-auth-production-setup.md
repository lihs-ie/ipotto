# Firebase Auth 本番プロジェクト設定 (Sprint 13.3)

> 作成: 2026-04-22
> 用途: 本番 `ipotto-prd` Firebase プロジェクトを Google Sign-In 用にセットアップするオペレーター手順。

前提: Firebase Console / GCP Console / gcloud CLI にアクセスできる人間オペレーターが実行する。

---

## 1. Firebase プロジェクト作成

1. https://console.firebase.google.com に Google アカウントでログイン
2. **Create a project** をクリック
3. プロジェクト名: `IPOtto Production`
4. プロジェクト ID: **`ipotto-prd`** (Terraform と揃える)
5. Google Analytics は任意 (運用開始直後は無効で OK)

## 2. Google Sign-In provider の有効化

1. Firebase Console → **Authentication** → **Sign-in method**
2. **Google** → Enable
3. Project support email を運用メールに設定 (`ops@example.com` など)
4. Save

## 3. Authorized domains

1. Authentication → **Settings** → **Authorized domains**
2. 以下を追加:
   - `localhost` (既定, dev 用)
   - 本番 frontend ドメイン (例: `ipotto.example.com` または Cloud Run URL `ipo-frontend-xxxxx.a.run.app`)
3. Save

## 4. OAuth redirect URIs

1. GCP Console → **APIs & Services** → **Credentials** → 該当 OAuth 2.0 クライアント
2. **Authorized redirect URIs** に以下を追加:
   - `https://<frontend-domain>/__/auth/handler`
   - `https://<ipotto-prd>.firebaseapp.com/__/auth/handler`
3. Save

## 5. Web app の登録

1. Firebase Console → **Project overview** → **Add app** → Web
2. App nickname: `ipotto-frontend-prod`
3. Firebase Hosting は無効 (Cloud Run を使うため)
4. `firebaseConfig` オブジェクトを控える:
   ```js
   {
     apiKey: "AIza...",
     authDomain: "ipotto-prd.firebaseapp.com",
     projectId: "ipotto-prd",
     // ...
   }
   ```
   `apiKey` / `authDomain` / `projectId` の 3 つが必要。

## 6. `ALLOWED_EMAIL` を Secret Manager に登録

```bash
echo -n "ops@example.com,admin@example.com" | \
  gcloud secrets create allowed-email \
    --project=ipotto-prd \
    --data-file=-
```

Cloud Run サービスの `secret_environment_variables` で参照するよう Terraform を更新 (本書とは別 PR で):

```hcl
secret_environment_variables = {
  ALLOWED_EMAIL = { secret_id = "allowed-email" }
}
```

## 7. `NEXT_PUBLIC_FIREBASE_*` の build arg 差替

本番ビルド時に docker build へ渡す。GitHub Actions `build-and-deploy.yml` の `docker build` ステップで:

```yaml
docker build \
  --build-arg NEXT_PUBLIC_FIREBASE_API_KEY=${{ secrets.PROD_FIREBASE_API_KEY }} \
  --build-arg NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=ipotto-prd.firebaseapp.com \
  --build-arg NEXT_PUBLIC_FIREBASE_PROJECT_ID=ipotto-prd \
  --build-arg NEXT_PUBLIC_API_BASE_URL=https://ipo-api-xxxxx.a.run.app \
  -f docker/ipo-frontend/Dockerfile services/
```

GitHub secrets に `PROD_FIREBASE_API_KEY` を事前登録する (手順 5 で取得した apiKey)。

## 8. 動作確認

1. `make deploy-prd` で ipo-frontend を再デプロイ (新 build arg 反映)
2. 本番 frontend URL にアクセス → `/login` が表示される
3. Google Sign-In ボタンをクリック → 許可された Google アカウントでログイン
4. `/` (ダッシュボード) へリダイレクトされ、ユーザーメールが header に表示
5. `gcloud run services logs read ipo-api --project=ipotto-prd --limit=10` で構造化ログに `uid` + masked email が出ていることを確認

## 9. 許可ユーザー追加フロー (運用)

1. 申請を受付
2. `gcloud secrets versions add allowed-email --project=ipotto-prd --data-file=-` で新しいリストを add
3. Cloud Run を再起動 (`gcloud run services update ipo-api --project=ipotto-prd --clear-env-vars` → 次回リクエスト時に反映)
4. 申請者に "ログイン可能" を通知

## 10. トラブルシュート

| 症状 | 原因候補 | 対処 |
|---|---|---|
| `/login` で Google ポップアップが `auth/unauthorized-domain` | Authorized domains 未登録 | §3 を再確認 |
| Sign-in 成功するが `/api/v1/*` が 403 `EMAIL_NOT_ALLOWED` | `ALLOWED_EMAIL` 未反映 | §6 + Cloud Run 再起動 |
| `/login` で apiKey エラー | build arg 未反映 | §7 を再確認 + `make deploy-prd` |
