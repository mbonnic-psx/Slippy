```rust
fn test_item() -> Item {
    Item { id: "item-1".into(), name: "Test Item".into(), weight_grams: 100 }
}

fn test_order() -> Order {
    Order {
        id: "order-1".into(),
        items: vec![test_item()],   // ✅ compose factories
        customer: test_customer(),  // ✅ compose factories
        payment: test_payment(),    // ✅ compose factories
    }
}

// Usage - override nested objects
#[test]
fn total_weight_of_several_items() {
    let order = Order {
        items: vec![
            Item { weight_grams: 100, ..test_item() },
            Item { weight_grams: 200, ..test_item() },
        ],
        ..test_order()
    };

    assert_eq!(total_weight_grams(&order), 300);
}
```
