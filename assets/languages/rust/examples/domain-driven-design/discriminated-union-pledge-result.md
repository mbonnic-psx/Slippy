```rust
/// The business reasons a pledge can be refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PledgeReason {
    ContributorIneligible,
    NonPositiveAmount,
    CurrencyMismatch,
    ExceedsBudget,
    FundingClosed,
}

/// The domain-level result: acceptance carries the updated aggregate and its events, rejection carries one
/// specific business reason.
pub type PledgeDecision = Result<Pledged, PledgeReason>;

/// Outcomes that belong to the use case, not the domain: the aggregate could not be found, or it changed
/// underneath us.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationReason {
    NotFound,
    ConcurrentChange,
}

/// The domain decision joined with the outcomes only the use case can produce.
#[derive(Debug, PartialEq)]
pub enum PledgeResult {
    Decision(PledgeDecision),
    ApplicationFailure(ApplicationReason),
}
```
