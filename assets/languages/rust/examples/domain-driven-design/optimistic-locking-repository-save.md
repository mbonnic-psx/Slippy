```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved,
    Conflict,
}

/// A compare-and-swap on the version: the WHERE clause matches the row only if nobody else has saved a newer
/// version in the meantime.
pub async fn save(pool: &sqlx::PgPool, occasion: &Occasion) -> Result<SaveOutcome, sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE occasions SET name = $1, budget_minor_units = $2, version = $3 WHERE id = $4 AND version = $5",
    )
    .bind(&occasion.name)
    .bind(occasion.budget.minor_units)
    .bind(occasion.version + 1)
    .bind(&occasion.id.0)
    .bind(occasion.version)
    .execute(pool)
    .await?;
    Ok(if updated.rows_affected() == 1 { SaveOutcome::Saved } else { SaveOutcome::Conflict })
}
```
