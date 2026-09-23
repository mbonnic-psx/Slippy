```rust
/// What creating a user came to, in the port's own words.
#[derive(Debug, PartialEq, Eq)]
pub enum CreateUserOutcome {
    Created,
    AlreadyExists,
}

pub trait UserCreator {
    async fn create(&self, user: &User) -> Result<CreateUserOutcome, sqlx::Error>;
}

/// Translates an expected storage condition into port vocabulary. Unexpected infrastructure errors
/// (connection lost, disk full) propagate unchanged.
pub struct PostgresUserCreator {
    pool: sqlx::PgPool,
}

impl UserCreator for PostgresUserCreator {
    async fn create(&self, user: &User) -> Result<CreateUserOutcome, sqlx::Error> {
        let inserted = sqlx::query(INSERT_USER)
            .bind(&user.id)
            .bind(&user.email)
            .execute(&self.pool)
            .await;
        match inserted {
            Ok(_) => Ok(CreateUserOutcome::Created),
            Err(sqlx::Error::Database(error)) if error.is_unique_violation() => Ok(CreateUserOutcome::AlreadyExists),
            Err(error) => Err(error), // unexpected: connection lost, disk full, ...
        }
    }
}
```
