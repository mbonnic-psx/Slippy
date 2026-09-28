//! The SQLite checkpoint store: the `projection_checkpoints` table the SQLite event store creates, reached
//! through that store's connection, so a checkpoint recorded inside its unit of work commits with the view.

use std::sync::Arc;
use std::time::Duration;

use time::OffsetDateTime;
use time::macros::format_description;

use crate::adapters::driven::event_store_sqlite::SqliteEventStore;
use crate::application::ports::events::{EventStore, StoreError};
use crate::application::ports::read_models::{CheckpointStore, FROM_THE_BEGINNING, Lease};

/// Takes or renews a lease in one statement, so the decision cannot be split by another worker: the row is
/// updated only when nobody holds it, this owner does, or the holder's lease has lapsed.
const CLAIM: &str = "INSERT INTO projection_checkpoints (projection, position, lease_owner, lease_expires_at) \
    VALUES (?, 0, ?, ?) \
    ON CONFLICT (projection) DO UPDATE \
      SET lease_owner = excluded.lease_owner, lease_expires_at = excluded.lease_expires_at \
      WHERE projection_checkpoints.lease_owner IS NULL \
         OR projection_checkpoints.lease_owner = excluded.lease_owner \
         OR projection_checkpoints.lease_expires_at <= ?";

/// An instant as fixed-width UTC text, so SQLite's string comparison orders instants correctly.
fn as_text(instant: OffsetDateTime) -> String {
    instant
        .to_offset(time::UtcOffset::UTC)
        .format(format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:9]Z"
        ))
        .unwrap_or_default()
}

pub struct SqliteCheckpointStore {
    events: Arc<SqliteEventStore>,
}

impl SqliteCheckpointStore {
    /// The checkpoint store that follows `events`, and shares its transaction.
    pub fn new(events: Arc<SqliteEventStore>) -> Self {
        Self { events }
    }
}

impl CheckpointStore for SqliteCheckpointStore {
    async fn position_of(&self, projection: &str) -> Result<i64, StoreError> {
        let position: Option<i64> = self
            .events
            .with_connection(async |connection| {
                sqlx::query_scalar(
                    "SELECT position FROM projection_checkpoints WHERE projection = ?",
                )
                .bind(projection)
                .fetch_optional(&mut *connection)
                .await
            })
            .await
            .map_err(|error| {
                StoreError::new(format!("read the position of {projection}"), error)
            })?;
        Ok(position.unwrap_or(FROM_THE_BEGINNING))
    }

    async fn record(&self, projection: &str, position: i64) -> Result<(), StoreError> {
        self.events
            .with_connection(async |connection| {
                sqlx::query(
                    "INSERT INTO projection_checkpoints (projection, position) VALUES (?, ?) \
                     ON CONFLICT (projection) DO UPDATE \
                       SET position = excluded.position, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
                )
                .bind(projection)
                .bind(position)
                .execute(&mut *connection)
                .await
                .map(drop)
            })
            .await
            .map_err(|error| StoreError::new(format!("record the position of {projection}"), error))
    }

    async fn claim(
        &self,
        projection: &str,
        owner: &str,
        now: OffsetDateTime,
        ttl: Duration,
    ) -> Result<Option<Lease>, StoreError> {
        let expires_at = now + ttl;
        let taken = self
            .events
            .in_unit_of_work(|| async {
                self.events
                    .with_connection(async |connection| {
                        let result = sqlx::query(CLAIM)
                            .bind(projection)
                            .bind(owner)
                            .bind(as_text(expires_at))
                            .bind(as_text(now))
                            .execute(&mut *connection)
                            .await?;
                        Ok(result.rows_affected() != 0)
                    })
                    .await
                    .map_err(|error| StoreError::new(format!("claim {projection}"), error))
            })
            .await?;
        Ok(taken.then(|| Lease {
            projection: projection.to_owned(),
            owner: owner.to_owned(),
            expires_at,
        }))
    }

    async fn release(&self, lease: &Lease) -> Result<(), StoreError> {
        self.events
            .with_connection(async |connection| {
                sqlx::query(
                    "UPDATE projection_checkpoints SET lease_owner = NULL, lease_expires_at = NULL \
                     WHERE projection = ? AND lease_owner = ?",
                )
                .bind(&lease.projection)
                .bind(&lease.owner)
                .execute(&mut *connection)
                .await
                .map(drop)
            })
            .await
            .map_err(|error| StoreError::new(format!("release {}", lease.projection), error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint_store_contract;

    #[tokio::test]
    async fn holds_to_the_checkpoint_store_contract() {
        let contract = checkpoint_store_contract::run(|| async {
            let events = Arc::new(
                SqliteEventStore::open("sqlite::memory:")
                    .await
                    .expect("open"),
            );
            let checkpoints = SqliteCheckpointStore::new(events.clone());
            (events, checkpoints)
        });
        // A unit of work that waits on the connection it holds hangs rather than failing; the deadline is what
        // makes that a failure.
        tokio::time::timeout(std::time::Duration::from_secs(5), contract)
            .await
            .expect("the contract finished before its deadline");
    }
}
