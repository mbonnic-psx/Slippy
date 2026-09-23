```rust
#[test]
fn handle_payment_calls_process() {
    let spy = ProcessSpy::default();

    handle_payment(&spy, payment);

    assert!(spy.called_with(&payment)); // so what?
}
```
