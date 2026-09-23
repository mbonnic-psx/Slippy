```rust
#[tokio::test]
async fn refuses_a_pledge_when_funding_is_closed() {
    let closed = Occasion { is_funding_closed: true, ..test_occasion() };
    let occasions = FakeOccasionRepository::with([closed.clone()]);
    let contributors = FakeContributorRepository::with([test_contributor()]);

    let result = handle_pledge(
        &Repos { occasions: &occasions, contributors: &contributors },
        PledgeDto {
            occasion_id: closed.id.clone(),
            contributor_id: test_contributor().id,
            amount: Money::gbp(2_500),
        },
    )
    .await
    .expect("no infrastructure failure");

    assert_eq!(result, PledgeResult::Decided(Err(PledgeRejection::FundingClosed)));
    assert!(occasions.saved().is_empty(), "nothing is saved when funding is closed");
}
```
