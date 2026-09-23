```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Gbp,
    Usd,
    Eur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    pub minor_units: i64,
    pub currency: Currency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccasionId(pub String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PledgeId(pub String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributorId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Occasion {
    pub id: OccasionId,
    pub budget: Money,
    pub total_pledged: Money,
    pub is_funding_closed: bool,
}

/// A read-only policy fact the caller supplies — typically from another bounded context.
#[derive(Debug, Clone, PartialEq)]
pub struct ContributorEligibility {
    pub contributor_id: ContributorId,
    pub may_pledge: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pledge {
    pub id: PledgeId,
    pub amount: Money,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PledgeRecorded {
    pub id: PledgeId,
    pub occasion_id: OccasionId,
    pub contributor_id: ContributorId,
    pub amount: Money,
}

/// The domain-level reasons a pledge can be refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PledgeRejection {
    ContributorIneligible,
    NonPositiveAmount,
    CurrencyMismatch,
    ExceedsBudget,
    FundingClosed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pledged {
    pub occasion: Occasion,
    pub events: Vec<PledgeRecorded>,
}

pub type PledgeDecision = Result<Pledged, PledgeRejection>;

// ❌ WRONG: cramming an external eligibility policy into one entity. A method on `Occasion` has no access to
// eligibility, so it can never make this decision correctly; it is here only to contrast with the service below.
impl Occasion {
    pub fn add_contribution(self, pledge: Pledge) -> Occasion {
        unimplemented!("an entity method has no access to eligibility")
    }
}

// ✅ CORRECT: a pure domain service that consumes a read-only policy fact supplied by the caller, rather than
// trying to know it itself.
pub fn pledge_contribution(occasion: Occasion, eligibility: &ContributorEligibility, pledge: Pledge) -> PledgeDecision {
    if !eligibility.may_pledge {
        return Err(PledgeRejection::ContributorIneligible);
    }
    if occasion.is_funding_closed {
        return Err(PledgeRejection::FundingClosed);
    }
    if pledge.amount.currency != occasion.total_pledged.currency || occasion.total_pledged.currency != occasion.budget.currency {
        return Err(PledgeRejection::CurrencyMismatch);
    }
    if pledge.amount.minor_units <= 0 {
        return Err(PledgeRejection::NonPositiveAmount);
    }
    assert!(occasion.total_pledged.minor_units <= occasion.budget.minor_units, "invalid Occasion funding invariant");
    if pledge.amount.minor_units > occasion.budget.minor_units - occasion.total_pledged.minor_units {
        return Err(PledgeRejection::ExceedsBudget);
    }
    let total_pledged = Money {
        minor_units: occasion.total_pledged.minor_units.checked_add(pledge.amount.minor_units).expect("money addition overflowed"),
        ..occasion.total_pledged
    };
    let event = PledgeRecorded {
        id: pledge.id,
        occasion_id: occasion.id.clone(),
        contributor_id: eligibility.contributor_id.clone(),
        amount: pledge.amount,
    };
    Ok(Pledged { occasion: Occasion { total_pledged, ..occasion }, events: vec![event] })
}
```
