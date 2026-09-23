```rust
// gifting/src/hexagon/application/pledging.rs — driving port + application result

/// An opaque, provider-free principal. Only the authentication adapter constructs one in production; the
/// request body cannot forge it.
#[derive(Debug, Clone)]
pub struct AuthenticatedPledger {
    pub contributor_id: ContributorId,
}

/// Every reason a pledge can be refused at the application boundary: the domain's own, plus the two only
/// this layer can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PledgeRejection {
    NonPositiveAmount,
    CurrencyMismatch,
    ExceedsBudget,
    FundingClosed,
    NotFound,
    ConcurrentChange,
}

impl From<RejectionReason> for PledgeRejection {
    fn from(reason: RejectionReason) -> Self {
        match reason {
            RejectionReason::NonPositiveAmount => Self::NonPositiveAmount,
            RejectionReason::CurrencyMismatch => Self::CurrencyMismatch,
            RejectionReason::ExceedsBudget => Self::ExceedsBudget,
            RejectionReason::FundingClosed => Self::FundingClosed,
        }
    }
}

pub type PledgeResult = Result<Pledged, PledgeRejection>;

/// The driving port's input.
pub struct PledgeToOccasionCommand {
    pub pledge_id: PledgeId,
    pub occasion_id: OccasionId,
    pub principal: AuthenticatedPledger,
    pub amount: Money,
}

/// The driving port: how the outside world asks to pledge to an occasion.
pub trait ForPledgingToOccasions {
    async fn pledge_to_occasion(&self, command: PledgeToOccasionCommand) -> Result<PledgeResult, PersistenceError>;
}

/// The application-owned, atomic driven port: it loads a versioned occasion and saves it with its outbox
/// events in one operation.
pub trait PledgePersistence {
    async fn find_occasion_by_id(&self, id: &OccasionId) -> Result<Option<StoredOccasion>, PersistenceError>;
    async fn save_with_outbox(&self, occasion: &Occasion, events: &[PledgeRecorded], expected_version: i64)
        -> Result<SaveOutcome, PersistenceError>;
}

/// The application-owned, idempotent driven port. The event is a validated, immutable log record: one id
/// permanently names one payload, and implementations must be safe against redelivery.
pub trait PledgeProjection {
    async fn record_from(&self, event: &PledgeRecorded) -> Result<(), PersistenceError>;
}
```
