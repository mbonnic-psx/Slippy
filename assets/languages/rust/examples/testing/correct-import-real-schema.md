```rust
use crate::domain::User; // the real type, with its real invariants

fn test_user() -> User {
    User {
        id: "user-123".into(),
        name: "Test User".into(),
        email: "test@example.com".into(),
        ..User::default()
    }
}
```
