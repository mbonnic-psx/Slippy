```rust
#[test]
fn handle_payment_returns_a_transaction_id() {
    let payment = test_payment();

    let receipt = handle_payment(payment).expect("a valid payment succeeds");

    assert!(!receipt.transaction_id.is_empty());
}
```
