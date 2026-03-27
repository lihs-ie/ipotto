# Terraform

IPOtto の GCP インフラを Terraform で管理するためのディレクトリです。

## 構成

```text
terraform/
├─ environments/
│  ├─ prd/
│  └─ stg/
└─ modules/
   ├─ artifact-registry/
   ├─ cloud-run/
   ├─ firestore/
   ├─ iam/
   ├─ pubsub/
   ├─ scheduler/
   └─ secret-manager/
```

## 使い方

前提:

```bash
terraform version
```

このプロジェクトでは Terraform `1.14.8` を使用します。

PRD の初期化と検証:

```bash
terraform -chdir=terraform/environments/prd init -backend=false
terraform -chdir=terraform/environments/prd validate
terraform -chdir=terraform/environments/prd plan
```

STG の初期化と検証:

```bash
terraform -chdir=terraform/environments/stg init -backend=false
terraform -chdir=terraform/environments/stg validate
terraform -chdir=terraform/environments/stg plan
```

`backend.tf` は GCS backend を前提にしています。初回は対象バケットを先に作成してください。
