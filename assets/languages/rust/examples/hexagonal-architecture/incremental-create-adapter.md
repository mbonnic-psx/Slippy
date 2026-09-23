```rust
// postgres_user_repository.rs — driven adapter

/// The driven adapter backed by sqlx, with optimistic concurrency (a version column) on save.
pub struct PostgresUserRepository {
    pool: sqlx::PgPool,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    balance_minor_units: i64,
    currency: String,
    version: i64,
}

impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<StoredUser>, RepositoryError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, balance_minor_units, currency, version FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| StoredUser {
            version: row.version,
            value: User { id: row.id, balance: Money { minor_units: row.balance_minor_units, currency: row.currency.parse()? } },
        }))
    }

    async fn save(&self, user: &User, expected_version: i64) -> Result<SaveOutcome, RepositoryError> {
        let updated = sqlx::query(
            "UPDATE users SET balance_minor_units = $1, currency = $2, version = $3 WHERE id = $4 AND version = $5",
        )
        .bind(user.balance.minor_units)
        .bind(user.balance.currency.as_str())
        .bind(expected_version + 1)
        .bind(&user.id)
        .bind(expected_version)
        .execute(&self.pool)
        .await?;
        Ok(if updated.rows_affected() == 1 { SaveOutcome::Saved } else { SaveOutcome::Conflict })
    }
}
```
