```rust
// Tests covering validation WITHOUT testing the validator directly.
#[test]
fn process_payment_validation() {
    let cases = [
        ("rejects negative amounts", Payment { amount_minor_units: -100, ..test_payment() }, false),
        ("rejects amounts over 10000", Payment { amount_minor_units: 1_500_000, ..test_payment() }, false),
        ("rejects invalid CVV", Payment { cvv: "12".into(), ..test_payment() }, false),
        (
            "processes valid payments",
            Payment { amount_minor_units: 10_000, currency: Currency::Gbp, cvv: "123".into(), ..test_payment() },
            true,
        ),
    ];

    for (name, payment, succeeds) in cases {
        let result = process_payment(payment);

        assert_eq!(result.is_ok(), succeeds, "{name}: {result:?}");
    }
}

// Result: validation branches are exercised through the public behaviour.
```
