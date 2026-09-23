```rust
#[test]
fn order_calculates_its_weight_in_grams() {
    let order = Order::new(vec![item_1, item_2]);

    assert_eq!(order.weight_grams(), 230);
}
```
