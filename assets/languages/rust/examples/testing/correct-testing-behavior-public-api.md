```rust
#[test]
fn process_payment() {
    let cases = [
        (
            "rejects negative amounts",
            Payment { amount_minor_units: -100, currency: Currency::Gbp, ..test_payment() },
            Err("amount must be positive"),
        ),
        (
            "rejects invalid CVV",
            Payment { cvv: "12".into(), ..test_payment() }, // only 2 digits
            Err("invalid CVV"),
        ),
        (
            "processes valid payments",
            Payment { amount_minor_units: 10_000, currency: Currency::Gbp, cvv: "123".into(), ..test_payment() },
            Ok(()),
        ),
    ];

    for (name, payment, want) in cases {
        let result = process_payment(payment);

        match (want, result) {
            (Ok(()), Ok(receipt)) => assert!(!receipt.transaction_id.is_empty(), "{name}"),
            (Err(message), Err(error)) => assert!(error.to_string().contains(message), "{name}: {error}"),
            (want, got) => panic!("{name}: wanted {want:?}, got {got:?}"),
        }
    }
}
```
