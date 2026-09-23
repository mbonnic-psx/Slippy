//! The Postgres checkpoint store: the `projection_checkpoints` table the migrations create, reached
//! through that store's connection, so a checkpoint recorded inside its unit of work commits with the view.

use std::sync::Arc;
use std::time::Duration;

use time::OffsetDateTime;
use time::macros::format_description;

use crate::adapters::driven::event_store_postgres::PostgresEventStore;
use crate::application::ports::events::{EventStore, StoreError};
use crate::application::ports::read_models::{CheckpointStore, FROM_THE_BEGINNING, Lease};

/// Takes or renews a lease in one statement, so the decision cannot be split by another worker: the row is
/// updated only when nobody holds it, this owner does, or the holder's lease has lapsed.
const CLAIM: &str = "INSERT INTO projection_checkpoints (projection, position, lease_owner, lease_expires_at) \
    VALUES ($1, 0, $2, $3::timestamptz) \
    ON CONFLICT (projection) DO UPDATE \
      SET lease_owner = excluded.lease_owner, lease_expires_at = excluded.lease_expires_at \
      WHERE projection_checkpoints.lease_owner IS NULL \
         OR projection_checkpoints.lease_owner = excluded.lease_owner \
         OR projection_checkpoints.lease_expires_at <= $4::timestamptz";

/// An instant as ISO-8601 UTC text, which Postgres casts to `timestamptz` on the way in.
fn as_text(instant: OffsetDateTime) -> String {
    instant
        .to_offset(time::UtcOffset::UTC)
        .format(format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:9]Z"
        ))
        .unwrap_or_default()
}

pub struct PostgresCheckpointStore {
    events: Arc<PostgresEventStore>,
}

impl PostgresCheckpointStore {
    /// The checkpoint store that follows `events`, and shares its transaction.
    pub fn new(events: Arc<PostgresEventStore>) -> Self {
        Self { events }
    }
}

impl CheckpointStore for PostgresCheckpointStore {
    async fn position_of(&self, projection: &str) -> Result<i64, StoreError> {
        let position: Option<i64> = self
            .events
            .with_connection(async |connection| {
                sqlx::query_scalar(
                    "SELECT position FROM projection_checkpoints WHERE projection = $1",
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
                    "INSERT INTO projection_checkpoints (projection, position) VALUES ($1, $2) \
                     ON CONFLICT (projection) DO UPDATE SET position = excluded.position, updated_at = now()",
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
                     WHERE projection = $1 AND lease_owner = $2",
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
    use tokio::sync::Barrier;

    use super::*;
    use crate::checkpoint_store_contract::{self, NOW};

    // Against a real Postgres, so outside the default suite — `make test-integration` runs these.

    fn url() -> String {
        std::env::var("DATABASE_URL").expect("DATABASE_URL names the database to test against")
    }

    async fn checkpoints() -> Arc<PostgresCheckpointStore> {
        let events = Arc::new(PostgresEventStore::connect(&url()).await.expect("connect"));
        Arc::new(PostgresCheckpointStore::new(events))
    }

    #[tokio::test]
    #[ignore = "integration: needs Postgres at DATABASE_URL, migrated"]
    async fn holds_to_the_checkpoint_store_contract() {
        let url = url();
        checkpoint_store_contract::run(|| async {
            let events = Arc::new(PostgresEventStore::connect(&url).await.expect("connect"));
            let checkpoints = PostgresCheckpointStore::new(events.clone());
            (events, checkpoints)
        })
        .await;
    }

    /// The claim is one statement, so it cannot be split by another worker: of many simultaneous claims on a
    /// free projection, exactly one takes it.
    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    #[ignore = "integration: needs Postgres at DATABASE_URL, migrated"]
    async fn exactly_one_of_many_simultaneous_claims_wins() {
        let checkpoints = checkpoints().await;
        let projection = format!("race-{}", uuid::Uuid::new_v4().simple());
        let gate = Arc::new(Barrier::new(8));
        let claims: Vec<_> = (0..8)
            .map(|index| {
                let (checkpoints, projection, gate) =
                    (checkpoints.clone(), projection.clone(), gate.clone());
                tokio::spawn(async move {
                    gate.wait().await;
                    checkpoints
                        .claim(
                            &projection,
                            &format!("worker-{index}"),
                            NOW,
                            Duration::from_secs(30),
                        )
                        .await
                })
            })
            .collect();
        let mut winners = 0;
        for claim in claims {
            if claim.await.expect("task").expect("claim").is_some() {
                winners += 1;
            }
        }
        assert_eq!(winners, 1);
    }

    /// A lease is visible to every other connection the moment it is taken — which is what makes it a lease
    /// between processes rather than a flag one process keeps.
    #[tokio::test]
    #[ignore = "integration: needs Postgres at DATABASE_URL, migrated"]
    async fn a_lease_is_visible_to_another_connection_at_once() {
        let (mine, theirs) = (checkpoints().await, checkpoints().await);
        let projection = format!("visible-{}", uuid::Uuid::new_v4().simple());
        assert!(
            mine.claim(&projection, "worker-1", NOW, Duration::from_secs(30))
                .await
                .expect("claim")
                .is_some()
        );
        let refused = theirs
            .claim(&projection, "worker-2", NOW, Duration::from_secs(30))
            .await
            .expect("claim");
        assert!(
            refused.is_none(),
            "another connection could take a held lease"
        );
    }
}
