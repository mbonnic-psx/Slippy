```rust
// gifting/src/hexagon/domain/pledge.rs — its #[cfg(test)] module

fn test_occasion(total_pledged_minor_units: i64) -> Occasion {
    Occasion {
        id: OccasionId("occasion-1".into()),
        name: "Alex's birthday".into(),
        budget: Money { minor_units: 10_000, currency: Currency::Gbp },
        total_pledged: Money { minor_units: total_pledged_minor_units, currency: Currency::Gbp },
        is_funding_closed: false,
    }
}

// A pure function: values in, a decision out. No fakes, no ports — a complement to the use-case tests for the
// rules with many edge cases.
#[test]
fn adds_the_exact_amount_and_records_what_happened() {
    let three_thousand = Money { minor_units: 3_000, currency: Currency::Gbp };

    let pledged = record_pledge(
        test_occasion(5_000),
        PledgeId("pledge-1".into()),
        ContributorId("contributor-1".into()),
        three_thousand,
    )
    .expect("the pledge is recorded");

    assert_eq!(pledged.occasion.total_pledged, Money { minor_units: 8_000, currency: Currency::Gbp });
    assert_eq!(pledged.events.len(), 1);
    assert_eq!(pledged.events[0].amount, three_thousand);
}
```
