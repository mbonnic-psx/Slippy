```rust
/// The outcome of attempting a pledge — no events, just the updated occasion or why not.
pub type PledgeDecision = Result<Occasion, PledgeRejection>;

/// Returns its result directly — simpler than the Decider pattern for a domain that needs no cross-aggregate
/// coordination.
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
    Ok(Occasion { total_pledged, ..occasion })
}
```
