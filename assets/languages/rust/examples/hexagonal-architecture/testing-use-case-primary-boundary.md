```rust
#[test]
fn place_order() {
    for (name, gateway, succeeds, saved) in [
        ("saves the order and charges the payment on success", FakePaymentGateway::always_succeeds(), true, 1),
        ("does not save the order when the payment fails", FakePaymentGateway::always_fails(), false, 0),
    ] {
        let orders = FakeOrderRepository::default();
        let order_placement = OrderPlacement::new(&orders, gateway);

        let result = order_placement.place_order(test_order()).expect("no infrastructure failure");

        assert_eq!(result.is_placed(), succeeds, "{name}");
        assert_eq!(orders.saved.borrow().len(), saved, "{name}");
    }
}
```
