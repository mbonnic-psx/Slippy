```rust
// gifting/tests/pledge_to_occasion.rs

fn test_occasion(budget: i64, total_pledged: i64, is_funding_closed: bool) -> Occasion {
    Occasion {
        id: OccasionId("occasion-1".into()),
        name: "Alex's birthday".into(),
        budget: Money { minor_units: budget, currency: Currency::Gbp },
        total_pledged: Money { minor_units: total_pledged, currency: Currency::Gbp },
        is_funding_closed,
    }
}

fn command(pledge: &str, occasion: &Occasion, amount: Money) -> PledgeToOccasionCommand {
    PledgeToOccasionCommand {
        pledge_id: PledgeId(pledge.into()),
        occasion_id: occasion.id.clone(),
        principal: AuthenticatedPledger { contributor_id: ContributorId("contributor-1".into()) },
        amount,
    }
}

fn gbp(minor_units: i64) -> Money {
    Money { minor_units, currency: Currency::Gbp }
}

#[tokio::test]
async fn updates_one_aggregate_and_records_its_outbox_event() {
    let occasion = test_occasion(10_000, 0, false);
    let pledging = PledgingToOccasions { persistence: FakePledgePersistence::with([occasion.clone()]) };

    let pledged = pledging.pledge_to_occasion(command("pledge-1", &occasion, gbp(2_500))).await.unwrap().unwrap();

    assert_eq!(pledged.occasion.total_pledged, gbp(2_500));
    assert_eq!(pledging.persistence.saved().len(), 1);
    assert_eq!(pledging.persistence.outbox().iter().map(|e| e.amount).collect::<Vec<_>>(), vec![gbp(2_500)]);
}

#[tokio::test]
async fn refuses_an_invalid_pledge_and_saves_nothing() {
    let usd = Money { minor_units: 2_500, currency: Currency::Usd };
    for (name, occasion, amount, reason) in [
        ("exceeds budget", test_occasion(10_000, 9_000, false), gbp(2_500), PledgeRejection::ExceedsBudget),
        ("currency mismatch", test_occasion(10_000, 0, false), usd, PledgeRejection::CurrencyMismatch),
        ("funding closed", test_occasion(10_000, 0, true), gbp(2_500), PledgeRejection::FundingClosed),
    ] {
        let pledging = PledgingToOccasions { persistence: FakePledgePersistence::with([occasion.clone()]) };

        let result = pledging.pledge_to_occasion(command("pledge-1", &occasion, amount)).await.unwrap();

        assert_eq!(result.err(), Some(reason), "{name}");
        assert!(pledging.persistence.saved().is_empty() && pledging.persistence.outbox().is_empty(), "{name}");
    }
}

// Not a table case: two tasks racing from one starting version earn a test of their own.
#[tokio::test(flavor = "multi_thread")]
async fn refuses_one_of_two_concurrent_writes_from_the_same_version() {
    let occasion = test_occasion(10_000, 0, false);
    let pledging = std::sync::Arc::new(PledgingToOccasions { persistence: FakePledgePersistence::with([occasion.clone()]) });

    let (first, second) = tokio::join!(
        pledging.pledge_to_occasion(command("pledge-1", &occasion, gbp(2_500))),
        pledging.pledge_to_occasion(command("pledge-2", &occasion, gbp(3_000))),
    );

    let outcomes = [first.unwrap(), second.unwrap()];
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(outcomes.iter().filter(|result| result.as_ref().err() == Some(&PledgeRejection::ConcurrentChange)).count(), 1);
    assert_eq!((pledging.persistence.saved().len(), pledging.persistence.outbox().len()), (1, 1));
}

#[tokio::test]
async fn handles_a_redelivered_event_once() {
    let projection = FakePledgeProjection::default();
    let event = PledgeRecorded {
        id: PledgeId("pledge-1".into()),
        occasion_id: OccasionId("occasion-1".into()),
        contributor_id: ContributorId("contributor-1".into()),
        amount: gbp(2_500),
    };

    handle_pledge_recorded(&projection, &event).await.unwrap();
    handle_pledge_recorded(&projection, &event).await.unwrap();

    assert_eq!(projection.records(), vec![event]);
}
```
