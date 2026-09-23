```rust
fn test_user() -> User {
    User {
        id: "user-123".into(),
        name: "Test User".into(),
        email: "test@example.com".into(),
        role: Role::User,
    }
}

/// A fixture that breaks the real rules fails where it was built, not three assertions later.
fn valid(user: User) -> User {
    user.validate().expect("invalid test fixture");
    user
}

// Usage
#[test]
fn create_user_with_a_custom_email() {
    let user = valid(User { email: "custom@example.com".into(), ..test_user() });

    let result = create_user(user);

    assert!(result.is_ok(), "{result:?}");
}
```
