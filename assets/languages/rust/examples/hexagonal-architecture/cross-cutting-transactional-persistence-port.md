```rust
/// The result of an atomic compare-and-save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved,
    Conflict,
}

/// An occasion with the version used for optimistic concurrency.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredOccasion {
    pub value: Occasion,
    pub version: i64,
}

/// The application-owned driven port: one semantic operation saves both the occasion and its outbox events,
/// or neither.
pub trait PledgePersistence {
    async fn find_occasion_by_id(&self, id: &OccasionId) -> Result<Option<StoredOccasion>, sqlx::Error>;
    async fn save_with_outbox(
        &self,
        occasion: &Occasion,
        events: &[PledgeRecorded],
        expected_version: i64,
    ) -> Result<SaveOutcome, sqlx::Error>;
}

/// The driven adapter; it owns the transaction mechanics.
pub struct PostgresPledgePersistence {
    pool: sqlx::PgPool,
}

impl PledgePersistence for PostgresPledgePersistence {
    async fn find_occasion_by_id(&self, id: &OccasionId) -> Result<Option<StoredOccasion>, sqlx::Error> {
        let row = sqlx::query_as::<_, OccasionRow>("select ... from occasions where id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(OccasionRow::into_stored))
    }

    async fn save_with_outbox(
        &self,
        occasion: &Occasion,
        events: &[PledgeRecorded],
        expected_version: i64,
    ) -> Result<SaveOutcome, sqlx::Error> {
        let mut tx = self.pool.begin().await?; // rolled back on drop unless committed

        let updated = sqlx::query("update occasions set version = $2 where id = $1 and version = $3")
            .bind(&occasion.id)
            .bind(expected_version + 1)
            .bind(expected_version)
            .execute(&mut *tx)
            .await?;
        if updated.rows_affected() != 1 {
            return Ok(SaveOutcome::Conflict);
        }

        for event in events {
            sqlx::query("insert into outbox (...) values (...)").bind(&event.id).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(SaveOutcome::Saved)
    }
}
```
