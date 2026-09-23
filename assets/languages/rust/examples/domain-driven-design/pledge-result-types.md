```rust
/// Reasons a pure domain service can determine on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainRejection {
    ContributorIneligible,
    NonPositiveAmount,
    CurrencyMismatch,
    ExceedsBudget,
    FundingClosed,
}

/// Outcomes only the surrounding use case can detect — the aggregate did not exist, or changed since it was
/// loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationRejection {
    NotFound,
    ConcurrentChange,
}

/// What a pure domain service can determine.
pub type PledgeDecision = Result<Pledged, DomainRejection>;

/// Either kind of refusal, kept apart so a caller can tell which layer said no.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    Domain(DomainRejection),
    Application(ApplicationRejection),
}

/// `PledgeDecision` widened with the application-level outcomes only the use case can detect.
pub type PledgeResult = Result<Pledged, Rejection>;

impl From<DomainRejection> for Rejection {
    fn from(reason: DomainRejection) -> Self {
        Rejection::Domain(reason)
    }
}

/// Lifts a domain decision into a result; `?` does the same inside a use case.
pub fn from_decision(decision: PledgeDecision) -> PledgeResult {
    decision.map_err(Rejection::from)
}
```
