```rust
// ❌ Missing name, email, role — so it leans on Default and hides what a user needs
fn test_user() -> User {
    User { id: "user-123".into(), ..Default::default() }
}
```
