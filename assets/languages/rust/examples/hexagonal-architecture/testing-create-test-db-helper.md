```rust
/// A fresh, isolated database per test — no state leaks between tests, and nothing to clean up: the pool
/// and its in-memory database are dropped with the test.
async fn test_db() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.expect("open test db"); // or Testcontainers for real Postgres
    sqlx::migrate!("./migrations").run(&pool).await.expect("apply migrations");
    pool
}
```
