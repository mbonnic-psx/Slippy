```rust
use time::macros::datetime;

#[test]
fn decide_order_placement() {
    let item = OrderItem { sku: "mug-01".into(), quantity: 2 };
    let now = datetime!(2026-03-20 0:00 UTC);

    let cases = [
        (
            "produces OrderPlaced when placing a draft order",
            OrderState::Draft { items: vec![item.clone()] },
            Ok(vec![OrderEvent::OrderPlaced { items: vec![item.clone()], placed_at: now }]),
        ),
        (
            "refuses to place an already placed order, with a reason",
            OrderState::Placed { items: vec![item.clone()], placed_at: datetime!(2026-03-18 0:00 UTC) },
            Err(OrderRejection::OrderNotDraft),
        ),
    ];

    for (name, state, expected) in cases {
        assert_eq!(decide(&OrderCommand::Place, &state, now), expected, "{name}");
    }
}
```
