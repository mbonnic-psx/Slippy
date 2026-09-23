//! An in-memory event store: a real implementation of the port, not a mock.
//!
//! It is what the shared contract suite runs against in `make verify`, where there is no database, and it
//! is held to exactly the same contract as the real adapters — so a slice that works against it works
//! against them. It keeps nothing between runs, which is the one thing it is not for.
//!
//! The log, the tag index, the checkpoints and the leases live in one [`Database`], shared by this store
//! and the in-memory checkpoint store built from it, because a unit of work has to cover both: rolled back,
//! it restores all four together, the way one SQL transaction would.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::application::ports::events::{
    AppendResult, CommittedEvent, Condition, ConditionalAppendResult, DomainEvent, EventStore,
    StoreError, TagQuery, TaggedRead, TagsOf, Visit, current_version, default_tags_of,
};

/// One projection's lease, as the database holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseRow {
    pub owner: String,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Default)]
struct State {
    log: Vec<CommittedEvent>,
    tags: HashMap<i64, Vec<String>>,
    checkpoints: HashMap<String, i64>,
    leases: HashMap<String, LeaseRow>,
}

/// Everything the in-memory adapters keep, behind one lock, shared by every adapter built from it.
#[derive(Debug, Clone, Default)]
pub struct Database {
    state: Arc<Mutex<State>>,
}

impl Database {
    fn lock(&self) -> MutexGuard<'_, State> {
        // A panic while holding the lock leaves nothing half-written that matters to a test double, so a
        // poisoned lock is recovered rather than propagated.
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Runs `work`, restoring everything it changed if it fails — the in-memory equivalent of a rolled-back
    /// transaction. Nested calls each restore their own changes, as savepoints would.
    pub async fn in_unit_of_work<T, F, Fut>(&self, work: F) -> Result<T, StoreError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, StoreError>>,
    {
        let snapshot = self.lock().clone();
        let outcome = work().await;
        if outcome.is_err() {
            *self.lock() = snapshot;
        }
        outcome
    }

    pub fn position_of(&self, projection: &str) -> i64 {
        self.lock()
            .checkpoints
            .get(projection)
            .copied()
            .unwrap_or(0)
    }

    pub fn record_position(&self, projection: &str, position: i64) {
        self.lock()
            .checkpoints
            .insert(projection.to_owned(), position);
    }

    pub fn lease(&self, projection: &str) -> Option<LeaseRow> {
        self.lock().leases.get(projection).cloned()
    }

    pub fn set_lease(&self, projection: &str, row: LeaseRow) {
        self.lock().leases.insert(projection.to_owned(), row);
    }

    /// Removes the lease only if `owner` still holds it.
    pub fn delete_lease(&self, projection: &str, owner: &str) {
        let mut state = self.lock();
        if state
            .leases
            .get(projection)
            .is_some_and(|row| row.owner == owner)
        {
            state.leases.remove(projection);
        }
    }
}

/// The in-memory event store.
pub struct InMemoryEventStore {
    database: Database,
    tags_of: RwLock<TagsOf>,
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryEventStore {
    /// A store over a fresh database, indexing each event by its own stream.
    pub fn new() -> Self {
        Self::tagged(default_tags_of(), Database::default())
    }

    /// A store over `database`, indexing each event by what `tags_of` returns for it.
    pub fn tagged(tags_of: TagsOf, database: Database) -> Self {
        Self {
            database,
            tags_of: RwLock::new(tags_of),
        }
    }

    /// The database this store writes to, which the in-memory checkpoint store is built from.
    pub fn database(&self) -> &Database {
        &self.database
    }

    fn tags_of(&self) -> TagsOf {
        self.tags_of
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

fn stream_of(state: &State, stream_id: &str) -> Vec<CommittedEvent> {
    state
        .log
        .iter()
        .filter(|event| event.event.stream_id == stream_id)
        .cloned()
        .collect()
}

fn head_of(state: &State) -> i64 {
    state.log.last().map_or(0, |event| event.global_position)
}

fn record(
    state: &mut State,
    tags_of: &TagsOf,
    mut event: DomainEvent,
    stream_id: &str,
    version: i64,
) {
    event.stream_id = stream_id.to_owned();
    let position = head_of(state) + 1;
    let tags = tags_of(&event);
    if !tags.is_empty() {
        state.tags.insert(position, tags);
    }
    let recorded_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default();
    state.log.push(CommittedEvent {
        event,
        version,
        global_position: position,
        recorded_at,
    });
}

fn read_tagged_in(state: &State, query: &TagQuery, after: i64, until: i64) -> TaggedRead {
    let ceiling = if until == 0 { head_of(state) } else { until };
    let events = state
        .log
        .iter()
        .filter(|event| event.global_position > after && event.global_position <= ceiling)
        .filter(|event| {
            let tags = state
                .tags
                .get(&event.global_position)
                .map_or(&[][..], Vec::as_slice);
            query.matches(event, tags)
        })
        .cloned()
        .collect();
    TaggedRead {
        events,
        head: ceiling,
    }
}

fn reindex_in(state: &mut State, tags_of: &TagsOf, from_position: i64) -> usize {
    let mut indexed = 0;
    let unindexed: Vec<CommittedEvent> = state
        .log
        .iter()
        .filter(|event| {
            event.global_position >= from_position
                && !state.tags.contains_key(&event.global_position)
        })
        .cloned()
        .collect();
    for event in unindexed {
        // An event the tagging function returns nothing for is never findable by tag, so it is neither
        // recorded as indexed nor counted: a later run under a real tagging function must still see it.
        let tags = tags_of(&event.event);
        if tags.is_empty() {
            continue;
        }
        state.tags.insert(event.global_position, tags);
        indexed += 1;
    }
    indexed
}

impl EventStore for InMemoryEventStore {
    async fn read(&self, stream_id: &str) -> Result<Vec<CommittedEvent>, StoreError> {
        Ok(stream_of(&self.database.lock(), stream_id))
    }

    async fn append(
        &self,
        stream_id: &str,
        expected_version: i64,
        events: Vec<DomainEvent>,
    ) -> Result<AppendResult, StoreError> {
        let tags_of = self.tags_of();
        let mut state = self.database.lock();
        let actual_version = current_version(&stream_of(&state, stream_id));
        if actual_version != expected_version {
            return Ok(AppendResult::VersionConflict { actual_version });
        }
        let mut version = actual_version;
        for event in events {
            version += 1;
            record(&mut state, &tags_of, event, stream_id, version);
        }
        Ok(AppendResult::Appended { version })
    }

    async fn read_all(&self, from_position: i64, visit: Visit<'_>) -> Result<(), StoreError> {
        // A snapshot, so a visitor that appends does not deadlock on the lock or see its own writes.
        let snapshot = self.database.lock().log.clone();
        for event in snapshot
            .into_iter()
            .filter(|event| event.global_position >= from_position)
        {
            visit(event)?;
        }
        Ok(())
    }

    async fn in_unit_of_work<T, F, Fut>(&self, work: F) -> Result<T, StoreError>
    where
        T: Send,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, StoreError>> + Send,
    {
        self.database.in_unit_of_work(work).await
    }

    async fn head(&self) -> Result<i64, StoreError> {
        Ok(head_of(&self.database.lock()))
    }

    async fn read_tagged(
        &self,
        query: &TagQuery,
        after: i64,
        until: i64,
    ) -> Result<TaggedRead, StoreError> {
        Ok(read_tagged_in(&self.database.lock(), query, after, until))
    }

    async fn append_if(
        &self,
        condition: &Condition,
        events: Vec<DomainEvent>,
    ) -> Result<ConditionalAppendResult, StoreError> {
        let tags_of = self.tags_of();
        let mut state = self.database.lock();
        if !read_tagged_in(&state, &condition.query, condition.after, 0)
            .events
            .is_empty()
        {
            return Ok(ConditionalAppendResult::ConditionConflict {
                head: head_of(&state),
            });
        }
        for event in events {
            let stream_id = event.stream_id.clone();
            let version = current_version(&stream_of(&state, &stream_id)) + 1;
            record(&mut state, &tags_of, event, &stream_id, version);
        }
        Ok(ConditionalAppendResult::Recorded {
            head: head_of(&state),
        })
    }

    async fn reindex_tags(&self, from_position: i64) -> Result<usize, StoreError> {
        let tags_of = self.tags_of();
        Ok(reindex_in(
            &mut self.database.lock(),
            &tags_of,
            from_position,
        ))
    }

    async fn retag(&self, tags_of: TagsOf) -> Result<usize, StoreError> {
        let mut state = self.database.lock();
        *self
            .tags_of
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = tags_of.clone();
        state.tags.clear();
        Ok(reindex_in(&mut state, &tags_of, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store_contract;

    /// Two stores over one database, the second tagging what the first did not: a reindex finds exactly the
    /// events the first wrote, which is the adoption path a running project takes.
    #[tokio::test]
    async fn reindexes_what_another_store_over_the_same_database_left_untagged() {
        use crate::application::ports::events::{
            Actor, CorrelationId, NO_STREAM, TagFilter, stream_tag,
        };
        let database = Database::default();
        let untagged =
            InMemoryEventStore::tagged(event_store_contract::no_tags(), database.clone());
        let event = |event_type: &str| DomainEvent {
            event_type: event_type.to_owned(),
            schema_version: 1,
            stream_id: "reindexed".to_owned(),
            payload: serde_json::Map::new(),
            occurred_at: "2024-01-01T00:00:00Z".to_owned(),
            actor: Actor {
                kind: "test".to_owned(),
                id: "reindex".to_owned(),
            },
            correlation_id: CorrelationId::generate(),
            causation_id: None,
        };
        untagged
            .append(
                "reindexed",
                NO_STREAM,
                vec![event("Started"), event("Continued")],
            )
            .await
            .expect("append");
        let tagged = InMemoryEventStore::tagged(default_tags_of(), database);

        assert_eq!(tagged.reindex_tags(0).await.expect("reindex"), 2);
        let query = TagQuery {
            filters: vec![TagFilter {
                tags: vec![stream_tag("reindexed")],
                types: vec![],
            }],
        };
        assert_eq!(
            tagged
                .read_tagged(&query, 0, 0)
                .await
                .expect("read")
                .events
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn holds_to_the_event_store_contract() {
        event_store_contract::run(async |tags_of| {
            InMemoryEventStore::tagged(tags_of, Database::default())
        })
        .await;
    }
}
