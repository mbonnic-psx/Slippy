```rust
#[test]
fn upcasts_order_placed_v1_to_v2() {
    let v1 = OrderPlacedV1 { order_id: "o-1".into(), total_minor_units: 4_000, currency: Currency::Eur };

    let current = upcast_order_placed(StoredOrderPlaced::V1(v1));

    assert_eq!(
        current,
        OrderPlacedV2 {
            order_id: "o-1".into(),
            total_amount: Money { minor_units: 4_000, currency: Currency::Eur },
        }
    );
}
```
