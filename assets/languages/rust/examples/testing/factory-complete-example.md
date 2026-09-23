```rust
use crate::domain::{Role, User}; // the real type and its real validation

fn test_user() -> User {
    let user = User {
        id: "user-123".into(),
        name: "Test User".into(),
        email: "test@example.com".into(),
        role: Role::User,
        is_active: true,
        created_at: Timestamp::from_ymd(2024, 1, 1),
    };
    user.validate().expect("invalid test fixture");
    user
}
```
