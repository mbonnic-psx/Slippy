```rust
fn test_occasion() -> Occasion {
    Occasion {
        id: OccasionId("occasion-1".into()),
        is_funding_closed: false,
        total_pledged: Money { minor_units: 0, currency: Currency::Gbp },
        budget: Money { minor_units: 50_000, currency: Currency::Gbp },
    }
}

fn test_eligibility(may_pledge: bool) -> ContributorEligibility {
    ContributorEligibility { contributor_id: ContributorId("contributor-1".into()), may_pledge }
}

#[test]
fn refuses_an_ineligible_contributor() {
    let pledge = Pledge { id: PledgeId("pledge-1".into()), amount: Money { minor_units: 5_000, currency: Currency::Gbp } };

    let decision = pledge_contribution(test_occasion(), &test_eligibility(false), pledge);

    assert_eq!(decision, Err(PledgeRejection::ContributorIneligible));
}
```
