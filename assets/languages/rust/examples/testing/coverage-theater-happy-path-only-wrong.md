```rust
#[test]
fn validate_happy_path_only() {
    let result = validate(&test_payment());

    assert!(result.is_ok()); // only the happy path!
}

// Missing: negative amounts, invalid CVV, missing fields, etc.
```
