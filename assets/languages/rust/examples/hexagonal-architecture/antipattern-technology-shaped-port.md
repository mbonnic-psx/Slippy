```rust
// ❌ Technology leaks into the port
pub trait UserRepository {
    async fn find_by_sql_query(&self, query: &str) -> Result<Vec<User>, RepositoryError>;
    async fn get_from_redis_cache(&self, key: &str) -> Result<User, RepositoryError>;
}

// ✅ Business language
pub trait UserRepository {
    async fn find_active(&self) -> Result<Vec<User>, RepositoryError>;
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError>;
}
```
