use thiserror::Error;

/// Shared domain-level errors across backend services.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("invalid status transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("duplicate application: stock={stock}, account={account}")]
    DuplicateApplication { stock: String, account: String },

    #[error("duplicate exclusion: company_name={company_name}")]
    DuplicateExclusion { company_name: String },

    #[error("invalid identifier for {kind}: {reason}")]
    InvalidIdentifier { kind: String, reason: String },

    #[error("invalid company name: {reason}")]
    InvalidCompanyName { reason: String },

    #[error("invalid ticker symbol: {reason}")]
    InvalidTickerSymbol { reason: String },

    #[error("invalid market: {reason}")]
    InvalidMarket { reason: String },

    #[error("invalid industry: {reason}")]
    InvalidIndustry { reason: String },

    #[error("invalid schedule: {reason}")]
    InvalidSchedule { reason: String },

    #[error("invalid price range: {reason}")]
    InvalidPriceRange { reason: String },

    #[error("invalid yen amount: {reason}")]
    InvalidYen { reason: String },

    #[error("invalid shares: {reason}")]
    InvalidShares { reason: String },

    #[error("invalid exclusion reason: {reason}")]
    InvalidExclusionReason { reason: String },

    #[error("invalid securities company: {reason}")]
    InvalidSecuritiesCompany { reason: String },

    #[error("credential is incomplete")]
    IncompleteCredential,

    #[error("invalid mail address: {reason}")]
    InvalidMailAddress { reason: String },

    #[error("invalid imap host: {reason}")]
    InvalidImapHost { reason: String },

    #[error("invalid imap port: {reason}")]
    InvalidImapPort { reason: String },

    #[error("no active notification channel")]
    NoActiveChannel,

    #[error("duplicate notification channel type: {channel_type}")]
    DuplicateChannelType { channel_type: String },

    #[error("notification channel not found: {channel_id}")]
    NotificationChannelNotFound { channel_id: String },

    #[error("invalid channel destination for {channel_type}: {reason}")]
    InvalidChannelDestination {
        channel_type: String,
        reason: String,
    },

    #[error("firestore mapping error: {reason}")]
    FirestoreMappingError { reason: String },

    #[error("secret payload error: {reason}")]
    SecretPayloadError { reason: String },

    #[error("pubsub publish error: {reason}")]
    PubSubPublishError { reason: String },

    #[error("notification send error for {channel_type}: {reason}")]
    NotificationSendError {
        channel_type: String,
        reason: String,
    },

    #[error("mail retrieval timed out")]
    MailRetrievalTimeout,

    #[error("mail parse error: {reason}")]
    MailParseError { reason: String },

    #[error("gmail api error: {reason}")]
    GmailApiError { reason: String },

    #[error("imap error: {reason}")]
    ImapError { reason: String },

    #[error("scraping error from {scraper_source}: {reason}")]
    ScrapingError {
        scraper_source: String,
        reason: String,
    },

    #[error("operation log validation error: {reason}")]
    OperationLogValidationError { reason: String },
}
