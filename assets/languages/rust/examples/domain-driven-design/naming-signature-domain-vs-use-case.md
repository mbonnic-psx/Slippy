```rust
/// A use case — it takes application-owned collaboration contracts (ports) as parameters.
pub async fn place_order(
    orders: &impl OrderRepository,
    gateway: &impl PaymentGateway,
    order: NewOrder,
) -> Result<PlaceOrderResult, PlaceError> {
    todo!()
}

/// A domain service — it takes only domain values.
pub fn pledge_contribution(occasion: Occasion, eligibility: &ContributorEligibility, pledge: Pledge) -> PledgeDecision {
    todo!()
}
```
