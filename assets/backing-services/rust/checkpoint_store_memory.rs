//! The in-memory checkpoint store, over the same [`Database`] as the in-memory event store, so a unit of work
//! rolled back there takes a checkpoint recorded here with it.

use std::time::Duration;

use time::OffsetDateTime;

use crate::adapters::driven::event_store_memory::{Database, InMemoryEventStore, LeaseRow};
use crate::application::ports::events::StoreError;
use crate::application::ports::read_models::{CheckpointStore, Lease};

pub struct InMemoryCheckpointStore {
    database: Database,
}

impl InMemoryCheckpointStore {
    /// The checkpoint store that follows `events`.
    pub fn new(events: &InMemoryEventStore) -> Self {
        Self {
            database: events.database().clone(),
        }
    }
}

impl CheckpointStore for InMemoryCheckpointStore {
    async fn position_of(&self, projection: &str) -> Result<i64, StoreError> {
        Ok(self.database.position_of(projection))
    }

    async fn record(&self, projection: &str, position: i64) -> Result<(), StoreError> {
        self.database.record_position(projection, position);
        Ok(())
    }

    async fn claim(
        &self,
        projection: &str,
        owner: &str,
        now: OffsetDateTime,
        ttl: Duration,
    ) -> Result<Option<Lease>, StoreError> {
        if let Some(held) = self.database.lease(projection)
            && held.owner != owner
            && held.expires_at > now
        {
            return Ok(None);
        }
        let expires_at = now + ttl;
        self.database.set_lease(
            projection,
            LeaseRow {
                owner: owner.to_owned(),
                expires_at,
            },
        );
        Ok(Some(Lease {
            projection: projection.to_owned(),
            owner: owner.to_owned(),
            expires_at,
        }))
    }

    async fn release(&self, lease: &Lease) -> Result<(), StoreError> {
        self.database.delete_lease(&lease.projection, &lease.owner);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint_store_contract;

    #[tokio::test]
    async fn holds_to_the_checkpoint_store_contract() {
        checkpoint_store_contract::run(|| async {
            let events = InMemoryEventStore::new();
            let checkpoints = InMemoryCheckpointStore::new(&events);
            (events, checkpoints)
        })
        .await;
    }
}
