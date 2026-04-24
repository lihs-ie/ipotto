# 証券口座クレデンシャル Envelope 暗号化 詳細設計

## 1. 目的

証券口座のログイン ID / パスワード / 取引パスワード / メール資格情報は IPOtto が「生きた自動ログイン用認証情報」として保持する最高機密データ。Secret Manager のデフォルト暗号化（Google-managed key, AES-256-GCM）に加え、アプリケーション層で envelope 暗号化を施すことで以下の脅威を緩和する。

- Secret Manager の IAM 誤設定により第三者が secret にアクセス可能になった場合でも、アプリ層の DEK がなければ平文に到達できない
- GCP 運用者・`gcloud` CLI 経由で secret 内容を覗く操作に対しても、暗号文のまま返り平文は見えない
- 開発者がデバッグ目的で Secret Manager コンソールを閲覧しても平文は視認不可

## 2. アーキテクチャ

```
+------------------------------------------------------+
| Repository (FirestoreSecuritiesAccountRepository)    |
|   uses Arc<dyn CredentialStorePort>                  |
+------------------------------------------------------+
                     |
                     v  save / get / delete / exists
+------------------------------------------------------+
| EncryptedCredentialStore (decorator)                 |
|   - AES-256-GCM per-value DEK                        |
|   - AAD = secret key name                            |
|   - JSON envelope bundle                             |
+------------------------------------------------------+
        |                                |
        | (wrap/unwrap DEK)              | (store/fetch bundle)
        v                                v
+-------------------+         +-------------------------+
| KeyManagementPort |         | CredentialStorePort     |
|  - GoogleKms...   |         |  - GoogleSecretManager  |
|  - InMemoryKms    |         |  - InMemoryCredential   |
+-------------------+         +-------------------------+
         |                              |
         v                              v
    Cloud KMS                     Secret Manager
   (KEK, rotation                (暗号化バンドル
    period 90d)                   JSON を保存)
```

`CredentialStorePort` のトレイト境界は envelope 化前後で一貫しており、`EncryptedCredentialStore` は純粋な decorator。Repository 側のコードは何も変わらない。

## 3. バンドルフォーマット

Secret Manager に保存する値は以下の UTF-8 JSON:

```json
{
  "version": 1,
  "wrapped_dek": "<base64url>",
  "nonce":       "<base64url, 12 bytes>",
  "ciphertext":  "<base64url, GCM tag 込み>"
}
```

- `version`: スキーマ進化用の拡張点。未知 version は即エラー（silent downgrade を防止）
- `wrapped_dek`: Cloud KMS `Encrypt` で 32 byte DEK を wrap した結果
- `nonce`: `OsRng` で save 毎に生成する 12 byte（AES-GCM 推奨値）
- `ciphertext`: AES-256-GCM の ciphertext + 認証タグ。**AAD** は secret key 名（例: `ipo-account-01HZXY...`）を使用し、bundle 内には格納しない。これにより「ciphertext を別 key の entry に付け替える」攻撃を復号時に検知できる

## 4. 鍵管理

| 鍵 | 保管場所 | ローテーション | 対象操作 |
|---|---|---|---|
| KEK (Key Encryption Key) | Cloud KMS CryptoKey `ipo-credential-kek` | 90 日自動 | `Encrypt` / `Decrypt` を通じて DEK を wrap/unwrap |
| DEK (Data Encryption Key) | メモリ（`zeroize::Zeroizing`） + Secret Manager（wrapped 形式） | save 毎に新規発行 | AES-256-GCM の暗号鍵 |

- **KEK ローテーション**: Cloud KMS の自動ローテーションで新バージョンが作成される。旧バージョンは破棄せず「保持」される（デフォルト挙動）。旧 DEK で wrap された bundle も復号可能で、マイグレーションは不要
- **DEK ローテーション**: 口座情報の更新時に自動で新 DEK に切り替わる（Secret Manager の新バージョンを作成する動作と同時）
- **KEK 紛失対策**: `lifecycle { prevent_destroy = true }` で Terraform からの破壊を防ぐ。手動で鍵を破棄する場合は IaC 上の例外対応が必要

## 5. 実装コード

### 5.1 Trait（ACL レイヤー）

```rust
// services/ipo-backend-shared/src/acl/crypto/key_management_port.rs
pub struct GeneratedDataKey {
    pub plaintext: zeroize::Zeroizing<Vec<u8>>,  // 32 bytes
    pub wrapped: Vec<u8>,
}

#[async_trait::async_trait]
pub trait KeyManagementPort: Send + Sync {
    async fn generate_data_key(&self) -> Result<GeneratedDataKey, DomainError>;
    async fn unwrap_data_key(
        &self,
        wrapped: &[u8],
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, DomainError>;
}
```

### 5.2 Decorator（Infrastructure レイヤー）

`services/ipo-backend-shared/src/infrastructure/crypto/encrypted_credential_store.rs`

```rust
pub struct EncryptedCredentialStore {
    inner: Arc<dyn CredentialStorePort>,
    kms: Arc<dyn KeyManagementPort>,
}

#[async_trait]
impl CredentialStorePort for EncryptedCredentialStore {
    async fn save(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let data_key = self.kms.generate_data_key().await?;
        // nonce 生成、AES-256-GCM 暗号化、bundle JSON 化、inner.save
    }

    async fn get(&self, key: &str) -> Result<String, DomainError> {
        let stored = self.inner.get(key).await?;
        // bundle JSON parse、version 検証、kms.unwrap_data_key、AES-256-GCM 復号
    }
}
```

### 5.3 Cloud KMS 実装

`services/ipo-backend-shared/src/infrastructure/crypto/google_kms_key_management.rs`

- `GenerateRandomBytes` は使わず、DEK はローカル `OsRng` で生成 → KMS の `Encrypt` / `Decrypt` で wrap/unwrap するだけの構成。これにより SOFTWARE-protected KEK で動作可能（HSM key ring 不要）
- KMS 呼び出し時の AAD は `b"ipo-backend-shared/google-kms/v1"` 固定（将来のアルゴリズム変更時のバンプ点）

### 5.4 テスト用スタブ

`services/ipo-backend-shared/src/infrastructure/crypto/in_memory_key_management.rs` は本実装と同じ AES-256-GCM で wrap/unwrap を行う。プロセス内固定 KEK で決定論的に動くため、`firestore_repositories_integration.rs` など統合テストから decorator 越しの round-trip を再現できる。

## 6. 環境変数と運用

| 環境変数 | 値 | 用途 |
|---|---|---|
| `KEY_MANAGEMENT_BACKEND` | `google-kms` / `in-memory` | 本番は `google-kms`、ローカルは `in-memory` |
| `IPOTTO_CREDENTIAL_KEK_NAME` | `projects/.../cryptoKeys/ipo-credential-kek` | `google-kms` backend 使用時に必須 |
| `CREDENTIAL_STORE_BACKEND` | `google-secret-manager` / `in-memory` | 本番は `google-secret-manager`、ローカルは `in-memory` |

Cloud Run へは Terraform 側の `cloud_run_services_resolved` local で `credential_kms.encrypter_decrypter_account_keys` に列挙された SA のサービスにだけ環境変数を注入する。これにより credential 消費者でないサービス（`ipo-info-fetcher`, `ipo-frontend` 等）には KEK 名や backend 種別が渡らない。

## 7. ローカル開発

`docker-compose.yml` では `ipo-api` / `ipo-applier` / `ipo-result-checker` のすべてに `CREDENTIAL_STORE_BACKEND=in-memory` + `KEY_MANAGEMENT_BACKEND=in-memory` を設定済み。`make up` するだけで動作し、KMS への疎通は不要。`InMemoryKeyManagement::random_for_test()` が起動毎に新しい KEK を生成するため、再起動後は旧 DEK を復号できない点に注意（ローカルでは問題にならない）。

## 8. 脅威モデルと対策

| 脅威 | 対策 |
|---|---|
| Secret Manager IAM 誤設定で第三者が secret にアクセス | アプリ層 AES-256-GCM で暗号化済みのため平文取得不可 |
| Secret Manager 管理者・運用者による内容確認 | 暗号文のみ閲覧可能、平文は `secretAccessor` + `cloudkms.cryptoKeyEncrypterDecrypter` の両権限が必要 |
| ciphertext を別 account key の entry に付け替える | AAD に secret key 名を使うため復号時にエラー |
| bundle の wrapped_dek の改竄 | Cloud KMS の `Decrypt` が失敗し `KeyManagementError` |
| bundle の ciphertext の改竄 | AES-256-GCM の認証タグ検証で `EnvelopeDecryptionError` |
| 旧バージョンの envelope 形式にダウングレード | `version != 1` で即エラー |
| KEK 破壊・紛失 | `prevent_destroy = true` で IaC 経由の破壊を防止、手動破棄は運用 runbook の対象 |
| DEK がメモリに残留 | `zeroize::Zeroizing` で Drop 時に自動 0 クリア |

## 9. 参照

- `services/ipo-backend-shared/src/acl/crypto/key_management_port.rs` — Port 定義
- `services/ipo-backend-shared/src/infrastructure/crypto/` — 実装
- `terraform/modules/kms/` — KeyRing / CryptoKey / IAM binding
- `terraform/environments/{prd,stg}/main.tf` — 環境毎の結線
- `docs/09-security-design/security-design.md` — 全体の脅威モデルと対策
