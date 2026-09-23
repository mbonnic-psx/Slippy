```rust
/// A test-data factory for a default, valid `Occasion`. Override any field with struct-update syntax —
/// `Occasion { is_funding_closed: true, ..test_occasion() }` — which is Rust's own partial-overrides object,
/// so no option functions are needed.
fn test_occasion() -> Occasion {
    Occasion {
        id: OccasionId("occasion-1".into()),
        name: "Mum's Birthday".into(),
        gift_ideas: Vec::new(),
        budget: Money { minor_units: 10_000, currency: Currency::Gbp },
        total_pledged: Money { minor_units: 0, currency: Currency::Gbp },
        is_funding_closed: false,
    }
}

#[test]
fn an_override_changes_only_the_field_it_names() {
    let closed = Occasion { is_funding_closed: true, ..test_occasion() };

    assert!(closed.is_funding_closed);
    assert_eq!(Occasion { is_funding_closed: false, ..closed }, test_occasion());
}
```
