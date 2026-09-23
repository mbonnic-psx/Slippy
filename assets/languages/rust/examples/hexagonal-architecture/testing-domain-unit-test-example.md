```rust
#[test]
fn a_pledge_exceeding_the_available_balance_is_refused() {
    let occasion = test_occasion();
    let poor_contributor = Contributor { available_balance: Money::new(500, Currency::Gbp), ..test_contributor() };
    let large_pledge = Pledge { amount: Money::new(5_000, Currency::Gbp), ..test_pledge() };

    let result = pledge_contribution(&occasion, &poor_contributor, &large_pledge);

    assert!(result.is_err(), "a pledge exceeding the available balance should be refused");
}
```
