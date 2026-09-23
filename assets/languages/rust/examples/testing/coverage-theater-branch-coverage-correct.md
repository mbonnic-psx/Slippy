```rust
#[test]
fn validate() {
    let cases = [
        ("rejects negative amounts", Payment { amount_minor_units: -100, ..test_payment() }, false),
        ("rejects amounts over limit", Payment { amount_minor_units: 1_500_000, ..test_payment() }, false),
        ("rejects invalid CVV", Payment { cvv: "12".into(), ..test_payment() }, false),
        ("accepts valid payments", test_payment(), true),
    ];

    for (name, payment, accepted) in cases {
        assert_eq!(validate(&payment).is_ok(), accepted, "{name}");
    }
}
```
