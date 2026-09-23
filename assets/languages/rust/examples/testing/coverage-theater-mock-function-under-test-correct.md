```rust
#[test]
fn validate_rejects_an_invalid_payment() {
    let payment = Payment { amount_minor_units: -100, currency: Currency::Gbp, ..test_payment() };

    let error = validate(&payment).expect_err("a negative amount is refused");

    assert!(error.to_string().contains("amount must be positive"), "{error}");
}
```
