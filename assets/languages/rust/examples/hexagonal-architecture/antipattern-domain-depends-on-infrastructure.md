```rust
// ❌ Domain uses sqlx directly
use sqlx::PgPool;

pub async fn find_active_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as("SELECT id, email, name FROM users WHERE active = true").fetch_all(pool).await
}

// ✅ Application defines the port it consumes; an adapter implements it
pub trait UserRepository {
    async fn find_active(&self) -> Result<Vec<User>, RepositoryError>;
}
```
