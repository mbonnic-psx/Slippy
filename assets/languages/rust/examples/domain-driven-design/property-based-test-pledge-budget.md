```rust
use proptest::prelude::*;

proptest! {
    // proptest generates inputs across the whole domain the property is defined over, and shrinks a failure
    // to the smallest case that still breaks it.
    #[test]
    fn an_accepted_pledge_never_exceeds_the_budget(already_pledged in 0i64..=10_000, pledge in 1i64..=10_000) {
        let occasion = test_occasion(Money::gbp(10_000), Money::gbp(already_pledged));
        let eligibility = ContributorEligibility { may_pledge: true, ..test_eligibility() };

        let result = pledge_contribution(occasion, &eligibility, Pledge { id: PledgeId("pledge-1".into()), amount: Money::gbp(pledge) });

        if let Ok(pledged) = result {
            prop_assert!(
                pledged.occasion.total_pledged.minor_units <= pledged.occasion.budget.minor_units,
                "accepted, but total {} exceeds budget {}",
                pledged.occasion.total_pledged.minor_units,
                pledged.occasion.budget.minor_units,
            );
        } // a rejected pledge is always valid
    }
}
```
