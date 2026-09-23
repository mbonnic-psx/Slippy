```rust
/// A specification: "can this eligible contributor pledge to this occasion?"
pub fn can_pledge(occasion: &Occasion, eligibility: &ContributorEligibility, amount: Money) -> bool {
    if occasion.total_pledged.minor_units > occasion.budget.minor_units {
        return false;
    }
    eligibility.may_pledge
        && !occasion.is_funding_closed
        && amount.minor_units > 0
        && amount.currency == occasion.total_pledged.currency
        && occasion.total_pledged.currency == occasion.budget.currency
        && amount.minor_units <= occasion.budget.minor_units - occasion.total_pledged.minor_units
}

/// Composes specifications into a more complex eligibility check.
pub fn is_gift_ready(occasion: &Occasion) -> bool {
    occasion.total_pledged.minor_units >= occasion.budget.minor_units
        && occasion.gift_ideas.iter().any(|idea| idea.status == GiftIdeaStatus::Selected)
}
```
