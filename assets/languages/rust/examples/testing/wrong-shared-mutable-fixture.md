```rust
// ❌ One process-wide fixture that every test can change
static SHARED_USER: LazyLock<Mutex<User>> =
    LazyLock::new(|| Mutex::new(User { id: "user-123".into(), name: "Test User".into(), ..test_user() }));

#[test]
fn one() {
    SHARED_USER.lock().unwrap().name = "Modified User".into();
}

#[test]
fn two() {
    assert_eq!(SHARED_USER.lock().unwrap().name, "Test User"); // order-dependent failure
}
```
