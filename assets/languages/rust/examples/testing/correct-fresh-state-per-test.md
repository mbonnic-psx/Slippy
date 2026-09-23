```rust
#[test]
fn one() {
    let user = User { name: "Modified User".into(), ..test_user() }; // fresh state
    // ...
}

#[test]
fn two() {
    let user = test_user(); // fresh state, not affected by `one`

    assert_eq!(user.name, "Test User"); // ✅ passes
}
```
