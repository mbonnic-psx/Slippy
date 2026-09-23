```rust
// gifting/src/adapters/driven/postgres/pledge_persistence.rs

/// The concrete `PledgePersistence`, backed by Postgres. It translates between domain types and rows and
/// implements the atomic compare-and-save-with-outbox contract in one transaction.
pub struct PostgresPledgePersistence {
    pool: sqlx::PgPool,
}

impl PledgePersistence for PostgresPledgePersistence {
    async fn find_occasion_by_id(&self, id: &OccasionId) -> Result<Option<StoredOccasion>, sqlx::Error> {
        let row = sqlx::query_as::<_, OccasionRow>(
            "select name, budget_minor_units, budget_currency, total_pledged_minor_units, total_pledged_currency, \
             is_funding_closed, version from occasions where id = $1",
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| row.into_stored(id.clone())))
    }

    async fn save_with_outbox(
        &self,
        occasion: &Occasion,
        events: &[PledgeRecorded],
        expected_version: i64,
    ) -> Result<SaveOutcome, sqlx::Error> {
        let mut tx = self.pool.begin().await?; // rolled back on drop unless committed

        // The version predicate stops two readers overwriting each other; a conflict inserts no outbox row.
        let updated = sqlx::query(
            "update occasions set name = $2, budget_minor_units = $3, budget_currency = $4, \
             total_pledged_minor_units = $5, total_pledged_currency = $6, is_funding_closed = $7, version = $8 \
             where id = $1 and version = $9",
        )
        .bind(&occasion.id.0)
        .bind(&occasion.name)
        .bind(occasion.budget.minor_units)
        .bind(occasion.budget.currency.code())
        .bind(occasion.total_pledged.minor_units)
        .bind(occasion.total_pledged.currency.code())
        .bind(occasion.is_funding_closed)
        .bind(expected_version + 1)
        .bind(expected_version)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() != 1 {
            return Ok(SaveOutcome::Conflict);
        }

        for event in events {
            sqlx::query(
                "insert into outbox (event_id, occasion_id, contributor_id, amount_minor_units, amount_currency) \
                 values ($1, $2, $3, $4, $5)",
            )
            .bind(&event.id.0)
            .bind(&event.occasion_id.0)
            .bind(&event.contributor_id.0)
            .bind(event.amount.minor_units)
            .bind(event.amount.currency.code())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(SaveOutcome::Saved)
    }
}

// gifting/src/adapters/driven/postgres/pledge_projection.rs

/// The concrete `PledgeProjection`. It inserts on the event's unique id and does nothing on conflict, so
/// redelivery is idempotent and an existing projection is never overwritten with incoming data.
pub struct PostgresPledgeProjection {
    pool: sqlx::PgPool,
}

impl PledgeProjection for PostgresPledgeProjection {
    async fn record_from(&self, event: &PledgeRecorded) -> Result<(), sqlx::Error> {
        sqlx::query(
            "insert into pledge_projection (event_id, occasion_id, contributor_id, amount_minor_units, amount_currency) \
             values ($1, $2, $3, $4, $5) on conflict (event_id) do nothing",
        )
        .bind(&event.id.0)
        .bind(&event.occasion_id.0)
        .bind(&event.contributor_id.0)
        .bind(event.amount.minor_units)
        .bind(event.amount.currency.code())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```
