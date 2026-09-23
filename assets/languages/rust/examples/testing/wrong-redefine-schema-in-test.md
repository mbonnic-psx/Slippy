```rust
// ❌ User is already defined in src/domain/user.rs!
#[derive(Debug, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
}

fn test_user() -> User {
    User { id: "user-123".into(), name: "Test User".into(), email: "test@example.com".into() }
}
```
