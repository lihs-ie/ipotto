terraform {
  backend "gcs" {
    bucket = "ipotto-stg-tfstate"
    prefix = "terraform/state"
  }
}
