```rust
fn test_user() -> User {
    User {
        id: "user-123".into(),
        name: "Test User".into(),
        email: "test@example.com".into(),
        role: Role::User,
    } // all required fields present
}
```
