//! A SQLite event store: a real append-only log in one file, with no server and no migration step.
//!
//! The schema lives here and is applied when the store opens, because a file-backed store has nobody else
//! to create it. Triggers refuse UPDATE and DELETE on the log, the same guarantee the Postgres migrations
//! give, so the log is append-only in the database and not merely by convention.
//!
//! One connection, always: SQLite serialises writers anyway, and a pool of several would turn a unit of
//! work that holds the write lock into another connection's "database is locked". The connection is the
//! queue.

use std::error::Error as _;
use std::future::Future;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use sqlx::sqlite::{
    SqliteConnectOptions, SqliteConnection, SqliteJournalMode, SqlitePoolOptions, SqliteRow,
};
use sqlx::{AssertSqlSafe, Row, Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::application::ports::events::{
    Actor, AppendResult, CausationId, CommittedEvent, Condition, ConditionalAppendResult,
    CorrelationId, DomainEvent, EventStore, StoreError, TagQuery, TaggedRead, TagsOf, Visit,
    default_tags_of,
};

const SCHEMA: &str = r#"
  CREATE TABLE IF NOT EXISTS events (
    global_position INTEGER PRIMARY KEY,
    stream_id       TEXT    NOT NULL,
    version         INTEGER NOT NULL,
    event_type      TEXT    NOT NULL,
    schema_version  INTEGER NOT NULL,
    payload         TEXT    NOT NULL,
    actor           TEXT    NOT NULL,
    -- SQLite has no UUID type, so the canonical text form is what is stored. Postgres uses a real uuid
    -- column; both round-trip through the same value type, and the adapter is where that difference stops.
    correlation_id  TEXT    NOT NULL,
    causation_id    TEXT,
    occurred_at     TEXT    NOT NULL,
    recorded_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CONSTRAINT events_stream_version_unique UNIQUE (stream_id, version),
    CONSTRAINT events_version_non_negative CHECK (version >= 0)
  );
  CREATE INDEX IF NOT EXISTS events_stream_id_version ON events (stream_id, version);
  CREATE TRIGGER IF NOT EXISTS events_reject_update BEFORE UPDATE ON events
  BEGIN SELECT RAISE(ABORT, 'events is append-only: UPDATE is rejected'); END;
  CREATE TRIGGER IF NOT EXISTS events_reject_delete BEFORE DELETE ON events
  BEGIN SELECT RAISE(ABORT, 'events is append-only: DELETE is rejected'); END;
  -- Where each projection has got to. Mutable by design, and deliberately with no trigger: a checkpoint is
  -- a position that moves, and everything derived from the log can be thrown away and rebuilt. The lease
  -- columns are how exactly one worker advances it, with an expiry so that survives the worker dying.
  CREATE TABLE IF NOT EXISTS projection_checkpoints (
    projection        TEXT    PRIMARY KEY,
    position          INTEGER NOT NULL DEFAULT 0,
    lease_owner       TEXT,
    lease_expires_at  TEXT,
    updated_at        TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CONSTRAINT projection_checkpoints_position_non_negative CHECK (position >= 0),
    CONSTRAINT projection_checkpoints_lease_is_whole CHECK ((lease_owner IS NULL) = (lease_expires_at IS NULL))
  );
  -- The tag index — the Dynamic Consistency Boundary's half of the log. Derived from the events by this
  -- project's tagging function, written inside the append's own transaction, and rebuildable at any time.
  CREATE TABLE IF NOT EXISTS event_tags (
    tag             TEXT    NOT NULL,
    global_position INTEGER NOT NULL REFERENCES events (global_position),
    PRIMARY KEY (tag, global_position)
  );
  CREATE INDEX IF NOT EXISTS event_tags_global_position ON event_tags (global_position);
"#;

const EVENT_COLUMNS: &str = "global_position, stream_id, version, event_type, schema_version, payload, actor, \
    correlation_id, causation_id, occurred_at, recorded_at";

const READ_ALL_BATCH: i64 = 500;

// ── The unit of work ─────────────────────────────────────────────────────────────────────────────────
//
// The open transaction travels with the task running the work, so every call the work makes to this store
// — or to the checkpoint store built from it — lands in it without the transaction being threaded through
// every signature. Scoped to one store by id, so two stores in one task cannot share each other's.

tokio::task_local! {
    static UNIT: Unit;
}

#[derive(Clone)]
struct Unit {
    store: usize,
    transaction: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
    depth: Arc<AtomicUsize>,
}

static NEXT_STORE: AtomicUsize = AtomicUsize::new(1);

/// The stale expected version an append found, carried out of the transaction it was found in.
#[derive(Debug, thiserror::Error)]
#[error("the stream is not where the caller thought it was")]
struct Stale;

/// The conditional append's guard, found broken inside its transaction.
#[derive(Debug, thiserror::Error)]
#[error("the condition no longer holds")]
struct ConditionBroken;

fn failed(context: &str) -> impl FnOnce(sqlx::Error) -> StoreError + '_ {
    move |error| StoreError::new(context, error)
}

/// The SQLite event store.
pub struct SqliteEventStore {
    pool: SqlitePool,
    id: usize,
    tags_of: RwLock<TagsOf>,
}

impl SqliteEventStore {
    /// Opens (creating if need be) the log at `location` — a path, or `sqlite::memory:` — indexing each
    /// event by its own stream.
    pub async fn open(location: &str) -> Result<Self, StoreError> {
        Self::open_tagged(location, default_tags_of()).await
    }

    /// Opens the log at `location`, indexing each event by what `tags_of` returns for it.
    pub async fn open_tagged(location: &str, tags_of: TagsOf) -> Result<Self, StoreError> {
        let options = SqliteConnectOptions::from_str(location)
            .map_err(failed("parse the sqlite location"))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(failed("open sqlite"))?;
        sqlx::raw_sql(SCHEMA)
            .execute(&pool)
            .await
            .map_err(failed("apply the sqlite schema"))?;
        Ok(Self {
            pool,
            id: NEXT_STORE.fetch_add(1, Ordering::Relaxed),
            tags_of: RwLock::new(tags_of),
        })
    }

    fn tags_of(&self) -> TagsOf {
        self.tags_of
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn unit(&self) -> Option<Unit> {
        UNIT.try_with(|unit| (unit.store == self.id).then(|| unit.clone()))
            .ok()
            .flatten()
    }

    /// Runs `operation` on the connection this call belongs to: the open unit of work's transaction when
    /// there is one, a connection of its own otherwise. Everything the adapters in this project do to the
    /// database goes through here, which is what puts a checkpoint in the same transaction as its view.
    pub(crate) async fn with_connection<T>(
        &self,
        operation: impl AsyncFnOnce(&mut SqliteConnection) -> Result<T, sqlx::Error>,
    ) -> Result<T, sqlx::Error> {
        if let Some(unit) = self.unit() {
            let mut guard = unit.transaction.lock().await;
            let transaction = guard.as_mut().ok_or(sqlx::Error::PoolClosed)?;
            operation(transaction).await
        } else {
            let mut connection = self.pool.acquire().await?;
            operation(&mut connection).await
        }
    }

    async fn current_version(&self, stream_id: &str) -> Result<i64, StoreError> {
        self.with_connection(async |connection| {
            sqlx::query_scalar("SELECT COALESCE(MAX(version), -1) FROM events WHERE stream_id = ?")
                .bind(stream_id)
                .fetch_one(&mut *connection)
                .await
        })
        .await
        .map_err(failed("read the current version"))
    }

    /// Writes one event at `version` only if the stream still stands at `guard_against`, and its tags with it.
    async fn insert(
        &self,
        event: DomainEvent,
        stream_id: &str,
        version: i64,
        guard_against: i64,
    ) -> Result<(), StoreError> {
        let payload = serde_json::to_string(&event.payload)
            .map_err(|error| StoreError::new("encode a payload", error))?;
        let actor = serde_json::to_string(&event.actor)
            .map_err(|error| StoreError::new("encode an actor", error))?;
        let mut tagged = event.clone();
        tagged.stream_id = stream_id.to_owned();
        let tags = (self.tags_of())(&tagged);
        let written = self
            .with_connection(async |connection| {
                let result = sqlx::query(
                    "INSERT INTO events (stream_id, version, event_type, schema_version, payload, actor, \
                     correlation_id, causation_id, occurred_at) \
                     SELECT ?, ?, ?, ?, ?, ?, ?, ?, ? \
                     WHERE (SELECT COALESCE(MAX(version), -1) FROM events WHERE stream_id = ?) = ?",
                )
                .bind(stream_id)
                .bind(version)
                .bind(&event.event_type)
                .bind(event.schema_version)
                .bind(&payload)
                .bind(&actor)
                .bind(event.correlation_id.to_string())
                .bind(event.causation_id.map(|id| id.to_string()))
                .bind(&event.occurred_at)
                .bind(stream_id)
                .bind(guard_against)
                .execute(&mut *connection)
                .await?;
                if result.rows_affected() == 0 {
                    return Ok(false);
                }
                let position = result.last_insert_rowid();
                for tag in &tags {
                    sqlx::query("INSERT OR IGNORE INTO event_tags (tag, global_position) VALUES (?, ?)")
                        .bind(tag)
                        .bind(position)
                        .execute(&mut *connection)
                        .await?;
                }
                Ok(true)
            })
            .await
            .map_err(failed("append an event"))?;
        if written {
            Ok(())
        } else {
            Err(StoreError::new("append an event", Stale))
        }
    }

    async fn head_now(&self) -> Result<i64, StoreError> {
        self.with_connection(async |connection| {
            sqlx::query_scalar("SELECT COALESCE(MAX(global_position), 0) FROM events")
                .fetch_one(&mut *connection)
                .await
        })
        .await
        .map_err(failed("read the store head"))
    }

    async fn select(
        &self,
        statement: &str,
        arguments: Vec<Argument>,
    ) -> Result<Vec<CommittedEvent>, StoreError> {
        let rows = self
            .with_connection(async |connection| {
                bind(query(statement), arguments)
                    .fetch_all(&mut *connection)
                    .await
            })
            .await
            .map_err(failed("read events"))?;
        rows.iter().map(scan).collect()
    }

    async fn anything_matching(&self, condition: &Condition) -> Result<bool, StoreError> {
        let (predicate, mut arguments) = tag_query_sql(&condition.query);
        arguments.insert(0, Argument::Integer(condition.after));
        let statement =
            format!("SELECT 1 FROM events e WHERE e.global_position > ? AND {predicate} LIMIT 1");
        let found = self
            .with_connection(async |connection| {
                bind(query(&statement), arguments)
                    .fetch_optional(&mut *connection)
                    .await
            })
            .await
            .map_err(failed("check a condition"))?;
        Ok(found.is_some())
    }
}

/// A statement built at run time. Every one here is assembled from this module's own constants and
/// placeholders — never from a caller's text, which travels only as a bound argument — so it is safe to hand
/// sqlx as SQL, and this is the one place that says so.
fn query(statement: &str) -> sqlx::query::Query<'static, Sqlite, sqlx::sqlite::SqliteArguments> {
    sqlx::query(AssertSqlSafe(statement.to_owned()))
}

/// A value bound into a statement built at run time.
enum Argument {
    Integer(i64),
    Text(String),
}

fn bind<'q>(
    mut query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments>,
    arguments: Vec<Argument>,
) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments> {
    for argument in arguments {
        query = match argument {
            Argument::Integer(value) => query.bind(value),
            Argument::Text(value) => query.bind(value),
        };
    }
    query
}

/// `TagQuery::matches`, translated. The contract suite is what holds the two to the same answer: a filter
/// matches an event carrying every tag it names (counted distinctly, so a repeated tag is one tag) and one of
/// its types where it names any; an empty filter or an empty query matches nothing.
fn tag_query_sql(query: &TagQuery) -> (String, Vec<Argument>) {
    if query.filters.is_empty() {
        return ("1 = 0".to_owned(), Vec::new());
    }
    let mut predicates = Vec::new();
    let mut arguments = Vec::new();
    for filter in &query.filters {
        if filter.tags.is_empty() && filter.types.is_empty() {
            predicates.push("1 = 0".to_owned());
            continue;
        }
        let mut parts = Vec::new();
        if !filter.tags.is_empty() {
            let mut distinct = filter.tags.clone();
            distinct.sort();
            distinct.dedup();
            parts.push(format!(
                "(SELECT COUNT(*) FROM event_tags t WHERE t.global_position = e.global_position AND t.tag IN ({})) = ?",
                placeholders(distinct.len())
            ));
            let count = distinct.len() as i64;
            arguments.extend(distinct.into_iter().map(Argument::Text));
            arguments.push(Argument::Integer(count));
        }
        if !filter.types.is_empty() {
            parts.push(format!(
                "e.event_type IN ({})",
                placeholders(filter.types.len())
            ));
            arguments.extend(filter.types.iter().cloned().map(Argument::Text));
        }
        predicates.push(format!("({})", parts.join(" AND ")));
    }
    (format!("({})", predicates.join(" OR ")), arguments)
}

fn placeholders(count: usize) -> String {
    vec!["?"; count].join(", ")
}

fn scan(row: &SqliteRow) -> Result<CommittedEvent, StoreError> {
    let corrupt =
        |error: &dyn std::fmt::Display| StoreError::new("read a stored event", error.to_string());
    let read = |error: sqlx::Error| StoreError::new("read a stored event", error);
    let payload: String = row.try_get("payload").map_err(read)?;
    let actor: String = row.try_get("actor").map_err(read)?;
    let correlation: String = row.try_get("correlation_id").map_err(read)?;
    let causation: Option<String> = row.try_get("causation_id").map_err(read)?;
    Ok(CommittedEvent {
        event: DomainEvent {
            event_type: row.try_get("event_type").map_err(read)?,
            schema_version: row.try_get("schema_version").map_err(read)?,
            stream_id: row.try_get("stream_id").map_err(read)?,
            payload: serde_json::from_str(&payload).map_err(|error| corrupt(&error))?,
            occurred_at: row.try_get("occurred_at").map_err(read)?,
            actor: serde_json::from_str::<Actor>(&actor).map_err(|error| corrupt(&error))?,
            correlation_id: CorrelationId::parse(&correlation).map_err(|error| corrupt(&error))?,
            causation_id: causation
                .as_deref()
                .map(CausationId::parse)
                .transpose()
                .map_err(|error| corrupt(&error))?,
        },
        version: row.try_get("version").map_err(read)?,
        global_position: row.try_get("global_position").map_err(read)?,
        recorded_at: row.try_get("recorded_at").map_err(read)?,
    })
}

/// Whether a failure is the append losing a race rather than a genuine failure: the guard found the stream
/// moved, or the unique (stream, version) constraint refused a concurrent writer.
fn lost_a_race(error: &StoreError) -> bool {
    match error.source() {
        Some(source) if source.is::<Stale>() => true,
        Some(source) => source
            .downcast_ref::<sqlx::Error>()
            .and_then(sqlx::Error::as_database_error)
            .is_some_and(|database| database.is_unique_violation()),
        None => false,
    }
}

impl EventStore for SqliteEventStore {
    async fn read(&self, stream_id: &str) -> Result<Vec<CommittedEvent>, StoreError> {
        let statement =
            format!("SELECT {EVENT_COLUMNS} FROM events WHERE stream_id = ? ORDER BY version ASC");
        self.select(&statement, vec![Argument::Text(stream_id.to_owned())])
            .await
    }

    async fn append(
        &self,
        stream_id: &str,
        expected_version: i64,
        events: Vec<DomainEvent>,
    ) -> Result<AppendResult, StoreError> {
        if events.is_empty() {
            return Ok(AppendResult::Appended {
                version: expected_version,
            });
        }
        let written = self
            .in_unit_of_work(|| async {
                let mut version = expected_version;
                for event in events {
                    self.insert(event, stream_id, version + 1, version).await?;
                    version += 1;
                }
                Ok(version)
            })
            .await;
        match written {
            Ok(version) => Ok(AppendResult::Appended { version }),
            Err(error) if lost_a_race(&error) => Ok(AppendResult::VersionConflict {
                actual_version: self.current_version(stream_id).await?,
            }),
            Err(error) => Err(error),
        }
    }

    async fn read_all(&self, from_position: i64, visit: Visit<'_>) -> Result<(), StoreError> {
        let statement = format!(
            "SELECT {EVENT_COLUMNS} FROM events WHERE global_position >= ? ORDER BY global_position ASC LIMIT ?"
        );
        let mut position = from_position;
        loop {
            let batch = self
                .select(
                    &statement,
                    vec![
                        Argument::Integer(position),
                        Argument::Integer(READ_ALL_BATCH),
                    ],
                )
                .await?;
            let Some(last) = batch.last().map(|event| event.global_position) else {
                return Ok(());
            };
            for event in batch {
                visit(event)?;
            }
            position = last + 1;
        }
    }

    async fn in_unit_of_work<T, F, Fut>(&self, work: F) -> Result<T, StoreError>
    where
        T: Send,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, StoreError>> + Send,
    {
        if let Some(unit) = self.unit() {
            // Nested: a savepoint, so a refused inner append undoes only itself and the outer work goes on.
            let name = format!("uow_{}", unit.depth.fetch_add(1, Ordering::Relaxed) + 1);
            let savepoint = |statement: String| async move {
                self.with_connection(async |connection| {
                    sqlx::raw_sql(AssertSqlSafe(statement))
                        .execute(&mut *connection)
                        .await
                })
                .await
                .map_err(failed("manage a savepoint"))
            };
            savepoint(format!("SAVEPOINT {name}")).await?;
            let outcome = work().await;
            let closing = match &outcome {
                Ok(_) => savepoint(format!("RELEASE SAVEPOINT {name}")).await,
                Err(_) => {
                    savepoint(format!(
                        "ROLLBACK TO SAVEPOINT {name}; RELEASE SAVEPOINT {name}"
                    ))
                    .await
                }
            };
            unit.depth.fetch_sub(1, Ordering::Relaxed);
            closing?;
            return outcome;
        }
        let transaction = self
            .pool
            .begin()
            .await
            .map_err(failed("begin a unit of work"))?;
        let unit = Unit {
            store: self.id,
            transaction: Arc::new(Mutex::new(Some(transaction))),
            depth: Arc::new(AtomicUsize::new(1)),
        };
        let outcome = UNIT.scope(unit.clone(), work()).await;
        let transaction = unit.transaction.lock().await.take();
        let Some(transaction) = transaction else {
            return Err(StoreError::new(
                "end a unit of work",
                "the transaction was already closed",
            ));
        };
        match outcome {
            Ok(value) => {
                transaction
                    .commit()
                    .await
                    .map_err(failed("commit a unit of work"))?;
                Ok(value)
            }
            Err(error) => {
                // The work's failure is what the caller needs; a rollback that also failed changes nothing.
                let _ = transaction.rollback().await;
                Err(error)
            }
        }
    }

    async fn head(&self) -> Result<i64, StoreError> {
        self.head_now().await
    }

    async fn read_tagged(
        &self,
        query: &TagQuery,
        after: i64,
        until: i64,
    ) -> Result<TaggedRead, StoreError> {
        let ceiling = if until == 0 {
            self.head_now().await?
        } else {
            until
        };
        let (predicate, arguments) = tag_query_sql(query);
        let statement = format!(
            "SELECT {EVENT_COLUMNS} FROM events e WHERE e.global_position > ? AND e.global_position <= ? AND {predicate} \
             ORDER BY e.global_position ASC"
        );
        let mut bound = vec![Argument::Integer(after), Argument::Integer(ceiling)];
        bound.extend(arguments);
        Ok(TaggedRead {
            events: self.select(&statement, bound).await?,
            head: ceiling,
        })
    }

    async fn append_if(
        &self,
        condition: &Condition,
        events: Vec<DomainEvent>,
    ) -> Result<ConditionalAppendResult, StoreError> {
        let written = self
            .in_unit_of_work(|| async {
                if self.anything_matching(condition).await? {
                    return Err(StoreError::new("conditional append", ConditionBroken));
                }
                for event in events {
                    let stream_id = event.stream_id.clone();
                    let at = self.current_version(&stream_id).await?;
                    self.insert(event, &stream_id, at + 1, at).await?;
                }
                Ok(())
            })
            .await;
        let head = self.head_now().await?;
        match written {
            Ok(()) => Ok(ConditionalAppendResult::Recorded { head }),
            Err(error)
                if error
                    .source()
                    .is_some_and(|source| source.is::<ConditionBroken>()) =>
            {
                Ok(ConditionalAppendResult::ConditionConflict { head })
            }
            Err(error) => Err(error),
        }
    }

    async fn reindex_tags(&self, from_position: i64) -> Result<usize, StoreError> {
        let tags_of = self.tags_of();
        self.in_unit_of_work(|| async {
            let statement = format!(
                "SELECT {EVENT_COLUMNS} FROM events WHERE global_position >= ? \
                 AND NOT EXISTS (SELECT 1 FROM event_tags WHERE global_position = events.global_position) \
                 ORDER BY global_position ASC"
            );
            let unindexed = self.select(&statement, vec![Argument::Integer(from_position)]).await?;
            let mut indexed = 0;
            for event in unindexed {
                // An event the tagging function returns nothing for is never findable by tag, so it is not
                // counted: a reindex that counted it would report work it did not do and never settle at zero.
                let tags = tags_of(&event.event);
                if tags.is_empty() {
                    continue;
                }
                self.with_connection(async |connection| {
                    for tag in &tags {
                        sqlx::query("INSERT OR IGNORE INTO event_tags (tag, global_position) VALUES (?, ?)")
                            .bind(tag)
                            .bind(event.global_position)
                            .execute(&mut *connection)
                            .await?;
                    }
                    Ok(())
                })
                .await
                .map_err(failed("reindex tags"))?;
                indexed += 1;
            }
            Ok(indexed)
        })
        .await
    }

    async fn retag(&self, tags_of: TagsOf) -> Result<usize, StoreError> {
        self.in_unit_of_work(|| async {
            *self
                .tags_of
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = tags_of;
            self.with_connection(async |connection| {
                sqlx::query("DELETE FROM event_tags")
                    .execute(&mut *connection)
                    .await
            })
            .await
            .map_err(failed("clear the tag index"))?;
            self.reindex_tags(0).await
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store_contract;

    /// A case that hangs — waiting on a connection its own unit of work holds, or paging the log forever — is
    /// a failure, and the deadline is what makes it one rather than a test run that never ends.
    const DEADLINE: std::time::Duration = std::time::Duration::from_secs(5);

    #[tokio::test]
    async fn holds_to_the_event_store_contract() {
        let contract = event_store_contract::run(async |tags_of| {
            SqliteEventStore::open_tagged("sqlite::memory:", tags_of)
                .await
                .expect("open an in-memory sqlite store")
        });
        tokio::time::timeout(DEADLINE, contract)
            .await
            .expect("the contract finished before its deadline");
    }

    /// An append that fails for any reason but a lost race is a failure, never a conflict: reporting it as a
    /// conflict sends the caller round a retry loop that can never succeed.
    #[tokio::test]
    async fn reports_a_failed_write_as_a_failure_not_a_conflict() {
        tokio::time::timeout(DEADLINE, a_failed_write_is_a_failure())
            .await
            .expect("the case finished before its deadline");
    }

    async fn a_failed_write_is_a_failure() {
        use crate::application::ports::events::{Actor, NO_STREAM, TagFilter};
        let store = SqliteEventStore::open("sqlite::memory:")
            .await
            .expect("open");
        store
            .with_connection(async |connection| {
                sqlx::query(
                    "CREATE TRIGGER refuse_every_write BEFORE INSERT ON events \
                     BEGIN SELECT RAISE(ABORT, 'refused'); END",
                )
                .execute(&mut *connection)
                .await
            })
            .await
            .expect("install the trigger");
        let event = || DomainEvent {
            event_type: "Started".to_owned(),
            schema_version: 1,
            stream_id: "refused".to_owned(),
            payload: serde_json::Map::new(),
            occurred_at: "2024-01-01T00:00:00Z".to_owned(),
            actor: Actor {
                kind: "test".to_owned(),
                id: "refused".to_owned(),
            },
            correlation_id: CorrelationId::generate(),
            causation_id: None,
        };

        let appended = store.append("refused", NO_STREAM, vec![event()]).await;
        assert!(appended.is_err(), "append answered {appended:?}");
        let condition = Condition {
            query: TagQuery {
                filters: vec![TagFilter {
                    tags: vec!["nothing:matches".to_owned()],
                    types: vec![],
                }],
            },
            after: 0,
        };
        let conditional = store.append_if(&condition, vec![event()]).await;
        assert!(
            conditional.is_err(),
            "conditional append answered {conditional:?}"
        );
    }

    #[tokio::test]
    async fn refuses_to_rewrite_the_log() {
        tokio::time::timeout(DEADLINE, the_log_cannot_be_rewritten())
            .await
            .expect("the case finished before its deadline");
    }

    async fn the_log_cannot_be_rewritten() {
        use crate::application::ports::events::{Actor, NO_STREAM};
        let store = SqliteEventStore::open("sqlite::memory:")
            .await
            .expect("open");
        let event = DomainEvent {
            event_type: "Started".to_owned(),
            schema_version: 1,
            stream_id: "rewrite".to_owned(),
            payload: serde_json::Map::new(),
            occurred_at: "2024-01-01T00:00:00Z".to_owned(),
            actor: Actor {
                kind: "test".to_owned(),
                id: "rewrite".to_owned(),
            },
            correlation_id: CorrelationId::generate(),
            causation_id: None,
        };
        store
            .append("rewrite", NO_STREAM, vec![event])
            .await
            .expect("append");
        // A row trigger fires per row, so the log needs a row in it for the refusal to be tested at all.
        for statement in [
            "DELETE FROM events",
            "UPDATE events SET event_type = 'Rewritten'",
        ] {
            let refused = store
                .with_connection(async |connection| {
                    sqlx::query(statement).execute(&mut *connection).await
                })
                .await;
            assert!(refused.is_err(), "the log accepted {statement}");
        }
    }
}
