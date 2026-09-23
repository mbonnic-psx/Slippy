```rust
// gifting/src/hexagon/domain/pledge.rs — aggregate operation, pure function

/// The aggregate operation. No ports, no infrastructure, no async: values in, a decision out. A broken
/// invariant (as opposed to an expected business rejection) panics, because it means an `Occasion` or a
/// `Money` was already corrupt before this function ran.
pub fn record_pledge(occasion: Occasion, pledge_id: PledgeId, contributor_id: ContributorId, amount: Money) -> PledgeDecision {
    if occasion.is_funding_closed {
        return Err(RejectionReason::FundingClosed);
    }
    if amount.currency != occasion.total_pledged.currency || occasion.total_pledged.currency != occasion.budget.currency {
        return Err(RejectionReason::CurrencyMismatch);
    }
    assert!(
        occasion.total_pledged.minor_units >= 0 && occasion.budget.minor_units >= 0 && amount.minor_units >= 0,
        "invalid Money invariant: minor units must not be negative"
    );
    if amount.minor_units == 0 {
        return Err(RejectionReason::NonPositiveAmount);
    }
    assert!(
        occasion.total_pledged.minor_units <= occasion.budget.minor_units,
        "invalid Occasion invariant: total pledged exceeds budget"
    );
    let remaining = occasion.budget.minor_units - occasion.total_pledged.minor_units;
    if amount.minor_units > remaining {
        return Err(RejectionReason::ExceedsBudget);
    }

    let total_pledged = Money {
        minor_units: occasion.total_pledged.minor_units.checked_add(amount.minor_units).expect("money addition overflowed"),
        ..occasion.total_pledged
    };
    let event = PledgeRecorded { id: pledge_id, occasion_id: occasion.id.clone(), contributor_id, amount };
    Ok(Pledged { occasion: Occasion { total_pledged, ..occasion }, events: vec![event] })
}
```
