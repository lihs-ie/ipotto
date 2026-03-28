terraform {
  backend "gcs" {
    bucket = "ipotto-prd-tfstate"
    prefix = "terraform/state"
  }
}
