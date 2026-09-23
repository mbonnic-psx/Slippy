```rust
// ❌ Testing HOW (implementation detail)
#[test]
fn process_payment_calls_validate_amount() {
    let spy = ValidateAmountSpy::default();

    process_payment_with(&spy, payment);

    assert!(spy.called.get()); // tests HOW, not WHAT
}

// ❌ Testing a private helper directly
#[test]
fn validate_cvv_format() {
    assert!(validate_cvv("123")); // private helper, not the public contract
}

// ❌ Testing internal state
#[test]
fn process_payment_sets_validated_flag() {
    let mut processor = Processor::default();

    processor.process(payment);

    assert!(processor.is_validated); // internal state
}
```
