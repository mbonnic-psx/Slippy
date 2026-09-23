```rust
#[test]
fn validator_validate_is_called() {
    let mut spy = ValidateSpy::default();

    spy.validate(&payment);

    assert!(spy.called); // meaningless assertion
}
```
