```rust
//! The gifting bounded context's pure business types — no ports, no infrastructure, no async.
//! gifting/src/hexagon/domain/types.rs

/// Distinct newtypes so the compiler refuses the wrong kind of id passed by mistake.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OccasionId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContributorId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PledgeId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Gbp,
    Usd,
    Eur,
}

/// An immutable value: minor units of a specific currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    pub minor_units: i64,
    pub currency: Currency,
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("invalid money: minor units must not be negative, got {0}")]
pub struct NegativeMoney(i64);

impl Money {
    /// Validates and constructs a `Money`.
    pub fn new(minor_units: i64, currency: Currency) -> Result<Self, NegativeMoney> {
        if minor_units < 0 {
            return Err(NegativeMoney(minor_units));
        }
        Ok(Self { minor_units, currency })
    }
}

/// The gift-giving-event aggregate.
#[derive(Debug, Clone, PartialEq)]
pub struct Occasion {
    pub id: OccasionId,
    pub name: String,
    pub budget: Money,
    pub total_pledged: Money,
    pub is_funding_closed: bool,
}

/// The domain event describing an accepted pledge.
#[derive(Debug, Clone, PartialEq)]
pub struct PledgeRecorded {
    pub id: PledgeId,
    pub occasion_id: OccasionId,
    pub contributor_id: ContributorId,
    pub amount: Money,
}

/// Why `record_pledge` can decline a pledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RejectionReason {
    NonPositiveAmount,
    CurrencyMismatch,
    ExceedsBudget,
    FundingClosed,
}

/// An accepted pledge: the updated occasion and what happened.
#[derive(Debug, Clone, PartialEq)]
pub struct Pledged {
    pub occasion: Occasion,
    pub events: Vec<PledgeRecorded>,
}

/// The result of attempting to record a pledge. There is no third, "error" case — `record_pledge` is pure and
/// every expected outcome is a returned value.
pub type PledgeDecision = Result<Pledged, RejectionReason>;
```
