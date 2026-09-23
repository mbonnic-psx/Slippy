```rust
// user_repository.rs — application-owned port

/// The domain value with its version, for optimistic concurrency on save.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredUser {
    pub value: User,
    pub version: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved,
    Conflict,
}

/// A driven port — application-owned because the use case consumes it.
pub trait UserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<StoredUser>, RepositoryError>;
    async fn save(&self, user: &User, expected_version: i64) -> Result<SaveOutcome, RepositoryError>;
}
```
