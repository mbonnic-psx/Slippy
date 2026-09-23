```rust
impl Money {
    /// Refuses an invariant violation — which would be a bug in the caller, not a business outcome.
    pub fn new(minor_units: i64, currency: Currency) -> Result<Self, NegativeMoney> {
        if minor_units < 0 {
            return Err(NegativeMoney(minor_units));
        }
        Ok(Self { minor_units, currency })
    }
}

/// Returns a result — an expected business outcome, not a bug. The rejection is in the signature, so no
/// caller can forget it.
pub fn pledge_contribution(occasion: Occasion, eligibility: &Eligibility, pledge: NewPledge) -> PledgeDecision {
    if !eligibility.may_pledge {
        return Err(PledgeRejection::ContributorIneligible);
    }
    // ...
}
```
