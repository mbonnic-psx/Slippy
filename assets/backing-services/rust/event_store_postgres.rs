//! A Postgres event store: the production log.
//!
//! The schema is the migrations' (`migrations/`, applied by `make migrate`), not this module's: a shared
//! database is changed deliberately and once, never by whichever process happens to open it first. The
//! migrations make the log append-only in the database itself — a trigger refuses UPDATE, DELETE and
//! TRUNCATE — so nothing here relies on callers behaving.
//!
//! A conditional append runs at SERIALIZABLE, because its guard is the *absence* of matching rows, and
//! absence is what no row lock can hold: Postgres detects the conflict at commit instead, and that failure
//! comes back as a condition conflict — a value, like every other kind of contention here.

use std::error::Error as _;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use sqlx::postgres::{PgConnection, PgPoolOptions, PgRow};
use sqlx::{AssertSqlSafe, PgPool, Postgres, Row, Transaction};
use tokio::sync::Mutex;

use crate::application::ports::events::{
    Actor, AppendResult, CausationId, CommittedEvent, Condition, ConditionalAppendResult,
    CorrelationId, DomainEvent, EventStore, StoreError, TagQuery, TaggedRead, TagsOf, Visit,
    default_tags_of,
};

/// Every column, read back as text in one fixed shape, so an instant round-trips as the same string whatever
/// the server's time zone and the driver never needs to know the column types.
const EVENT_COLUMNS: &str = "global_position, stream_id, version, event_type, schema_version, payload::text AS payload, \
    actor::text AS actor, correlation_id::text AS correlation_id, causation_id::text AS causation_id, \
    to_char(occurred_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.USOF') AS occurred_at, \
    to_char(recorded_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.USOF') AS recorded_at";

const READ_ALL_BATCH: i64 = 500;

/// The log's own advisory lock, which is how a reader learns that a position is settled.
///
/// A global position is assigned when a row is inserted and becomes visible when its transaction commits,
/// and those are not the same moment: two appends overlapping can take 5 and 6 and commit 6 first. A reader
/// that sees 6 and records "next is 7" has skipped 5 for good. So an append takes this lock in shared mode
/// before its first insert — shared holders never block each other — and a reader takes it exclusively for
/// one `MAX(global_position)`, which waits out every append in flight: every position at or below that
/// maximum is then final. The number is arbitrary, must be the same in every adapter that opens this log,
/// and must not be reused for anything else in the same database.
const EVENTS_LOCK: i64 = 8_317_231;

/// Postgres' SQLSTATE for a SERIALIZABLE transaction that could not be serialised.
const SERIALIZATION_FAILURE: &str = "40001";

tokio::task_local! {
    static UNIT: Unit;
}

#[derive(Clone)]
struct Unit {
    store: usize,
    transaction: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
    depth: Arc<AtomicUsize>,
}

static NEXT_STORE: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, thiserror::Error)]
#[error("the stream is not where the caller thought it was")]
struct Stale;

#[derive(Debug, thiserror::Error)]
#[error("the condition no longer holds")]
struct ConditionBroken;

fn failed(context: &str) -> impl FnOnce(sqlx::Error) -> StoreError + '_ {
    move |error| StoreError::new(context, error)
}

/// A statement built at run time, from this module's own constants and placeholders only — a caller's text
/// travels as a bound argument, never as SQL — which is what makes it safe to hand sqlx as SQL.
fn query(statement: &str) -> sqlx::query::Query<'static, Postgres, sqlx::postgres::PgArguments> {
    sqlx::query(AssertSqlSafe(statement.to_owned()))
}

/// The SQLSTATE of a database error, where the failure was one.
fn sqlstate(error: &StoreError) -> Option<String> {
    let database = error
        .source()?
        .downcast_ref::<sqlx::Error>()?
        .as_database_error()?;
    database.code().map(|code| code.into_owned())
}

/// The Postgres event store.
pub struct PostgresEventStore {
    pool: PgPool,
    id: usize,
    tags_of: RwLock<TagsOf>,
}

impl PostgresEventStore {
    /// Connects to the database at `url`, indexing each event by its own stream.
    pub async fn connect(url: &str) -> Result<Self, StoreError> {
        Self::connect_tagged(url, default_tags_of()).await
    }

    /// Connects to the database at `url`, indexing each event by what `tags_of` returns for it.
    pub async fn connect_tagged(url: &str, tags_of: TagsOf) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .connect(url)
            .await
            .map_err(failed("connect to postgres"))?;
        Ok(Self::from_pool(pool, tags_of))
    }

    /// A store over a pool somebody else opened.
    pub fn from_pool(pool: PgPool, tags_of: TagsOf) -> Self {
        Self {
            pool,
            id: NEXT_STORE.fetch_add(1, Ordering::Relaxed),
            tags_of: RwLock::new(tags_of),
        }
    }

    /// Whether the database answers, for a readiness probe.
    pub async fn ping(&self) -> Result<(), StoreError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(drop)
            .map_err(failed("ping postgres"))
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
    /// there is one, a pooled connection otherwise. The checkpoint store built from this store goes through
    /// here too, which is what puts a checkpoint in the same transaction as the view it accounts for.
    pub(crate) async fn with_connection<T>(
        &self,
        operation: impl AsyncFnOnce(&mut PgConnection) -> Result<T, sqlx::Error>,
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

    async fn execute(&self, statement: &'static str, context: &str) -> Result<(), StoreError> {
        self.with_connection(async |connection| {
            sqlx::raw_sql(statement)
                .execute(&mut *connection)
                .await
                .map(drop)
        })
        .await
        .map_err(failed(context))
    }

    /// Holds the log in shared mode until this transaction ends — see [`EVENTS_LOCK`]. Before the first
    /// insert, always; re-entrant, so taking it twice in one transaction is free.
    async fn hold_the_log(&self) -> Result<(), StoreError> {
        self.with_connection(async |connection| {
            sqlx::query("SELECT pg_advisory_xact_lock_shared($1)")
                .bind(EVENTS_LOCK)
                .execute(&mut *connection)
                .await
                .map(drop)
        })
        .await
        .map_err(failed("hold the log"))
    }

    async fn head_now(&self) -> Result<i64, StoreError> {
        self.with_connection(async |connection| {
            sqlx::query_scalar("SELECT COALESCE(MAX(global_position), 0)::bigint FROM events")
                .fetch_one(&mut *connection)
                .await
        })
        .await
        .map_err(failed("read the store head"))
    }

    /// The highest position nothing earlier can still be committed behind: an exclusive hold on the log,
    /// long enough to prove nothing is in flight, in a transaction of its own so the hold lasts microseconds.
    async fn settled_position(&self) -> Result<i64, StoreError> {
        self.in_unit_of_work(|| async {
            self.with_connection(async |connection| {
                sqlx::query("SELECT pg_advisory_xact_lock($1)")
                    .bind(EVENTS_LOCK)
                    .execute(&mut *connection)
                    .await
                    .map(drop)
            })
            .await
            .map_err(failed("wait for the log to settle"))?;
            self.head_now().await
        })
        .await
    }

    async fn current_version(&self, stream_id: &str) -> Result<i64, StoreError> {
        self.with_connection(async |connection| {
            sqlx::query_scalar(
                "SELECT COALESCE(MAX(version), -1)::bigint FROM events WHERE stream_id = $1",
            )
            .bind(stream_id)
            .fetch_one(&mut *connection)
            .await
        })
        .await
        .map_err(failed("read the current version"))
    }

    /// Writes one event at `version` only if the stream still stands at `guard_against`, and its tags with it
    /// — in the same transaction, because an index written afterwards could be missing when the next
    /// conditional append checks it.
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
                let position: Option<i64> = sqlx::query_scalar(
                    "INSERT INTO events (stream_id, version, event_type, schema_version, payload, actor, \
                     correlation_id, causation_id, occurred_at) \
                     SELECT $1, $2, $3, $4, $5::jsonb, $6::jsonb, $7::uuid, $8::uuid, $9::timestamptz \
                     WHERE (SELECT COALESCE(MAX(version), -1) FROM events WHERE stream_id = $1) = $10 \
                     RETURNING global_position",
                )
                .bind(stream_id)
                .bind(i32::try_from(version).unwrap_or(i32::MAX))
                .bind(&event.event_type)
                .bind(event.schema_version)
                .bind(&payload)
                .bind(&actor)
                .bind(event.correlation_id.to_string())
                .bind(event.causation_id.map(|id| id.to_string()))
                .bind(&event.occurred_at)
                .bind(guard_against)
                .fetch_optional(&mut *connection)
                .await?;
                let Some(position) = position else {
                    return Ok(false);
                };
                for tag in &tags {
                    sqlx::query("INSERT INTO event_tags (tag, global_position) VALUES ($1, $2) ON CONFLICT DO NOTHING")
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
        let (predicate, arguments) = tag_query_sql(&condition.query, 2);
        let statement =
            format!("SELECT 1 FROM events e WHERE e.global_position > $1 AND {predicate} LIMIT 1");
        let mut bound = vec![Argument::Integer(condition.after)];
        bound.extend(arguments);
        let found = self
            .with_connection(async |connection| {
                bind(query(&statement), bound)
                    .fetch_optional(&mut *connection)
                    .await
            })
            .await
            .map_err(failed("check a condition"))?;
        Ok(found.is_some())
    }

    /// A unit of work opened at `isolation`, where it is the outermost one; nested, it is a savepoint in the
    /// transaction already open, at that transaction's level, because an isolation level can only be set as a
    /// transaction opens.
    async fn unit_of_work<T, F, Fut>(
        &self,
        isolation: Option<&'static str>,
        work: F,
    ) -> Result<T, StoreError>
    where
        T: Send,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, StoreError>> + Send,
    {
        if let Some(unit) = self.unit() {
            let name = format!("uow_{}", unit.depth.fetch_add(1, Ordering::Relaxed) + 1);
            let savepoint = |statement: String| async move {
                self.with_connection(async |connection| {
                    sqlx::raw_sql(AssertSqlSafe(statement))
                        .execute(&mut *connection)
                        .await
                        .map(drop)
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
        // One connection for the whole unit of work: reaching back into the pool from inside it is what
        // deadlocks under contention, every in-flight caller holding one connection and waiting for a second.
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(failed("begin a unit of work"))?;
        if let Some(level) = isolation {
            sqlx::raw_sql(AssertSqlSafe(format!(
                "SET TRANSACTION ISOLATION LEVEL {level}"
            )))
            .execute(&mut *transaction)
            .await
            .map_err(failed("set the isolation level"))?;
        }
        let unit = Unit {
            store: self.id,
            transaction: Arc::new(Mutex::new(Some(transaction))),
            depth: Arc::new(AtomicUsize::new(1)),
        };
        let outcome = UNIT.scope(unit.clone(), work()).await;
        let Some(transaction) = unit.transaction.lock().await.take() else {
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
                let _ = transaction.rollback().await;
                Err(error)
            }
        }
    }
}

enum Argument {
    Integer(i64),
    Texts(Vec<String>),
}

fn bind(
    mut query: sqlx::query::Query<'static, Postgres, sqlx::postgres::PgArguments>,
    arguments: Vec<Argument>,
) -> sqlx::query::Query<'static, Postgres, sqlx::postgres::PgArguments> {
    for argument in arguments {
        query = match argument {
            Argument::Integer(value) => query.bind(value),
            Argument::Texts(values) => query.bind(values),
        };
    }
    query
}

/// `TagQuery::matches`, translated, with placeholders numbered from `next`. Each SQL adapter carries its own
/// copy, deliberately: an adapter that imports another is two adapters that cannot be pruned apart. A
/// filter's tags are a conjunction — the distinct tags found against the event must number as many as it
/// named, because `= ANY` alone is an "any of" and a much weaker guard; its types narrow within that filter;
/// and a filter naming neither, or a query with none, matches nothing.
fn tag_query_sql(query: &TagQuery, next: usize) -> (String, Vec<Argument>) {
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
            let count = distinct.len() as i64;
            arguments.push(Argument::Texts(distinct));
            let tags_at = next + arguments.len() - 1;
            arguments.push(Argument::Integer(count));
            let count_at = next + arguments.len() - 1;
            parts.push(format!(
                "(SELECT COUNT(*) FROM event_tags t WHERE t.global_position = e.global_position AND t.tag = ANY(${tags_at})) = ${count_at}"
            ));
        }
        if !filter.types.is_empty() {
            arguments.push(Argument::Texts(filter.types.clone()));
            parts.push(format!(
                "e.event_type = ANY(${})",
                next + arguments.len() - 1
            ));
        }
        predicates.push(format!("({})", parts.join(" AND ")));
    }
    (format!("({})", predicates.join(" OR ")), arguments)
}

fn scan(row: &PgRow) -> Result<CommittedEvent, StoreError> {
    let corrupt =
        |error: &dyn std::fmt::Display| StoreError::new("read a stored event", error.to_string());
    let read = |error: sqlx::Error| StoreError::new("read a stored event", error);
    let payload: String = row.try_get("payload").map_err(read)?;
    let actor: String = row.try_get("actor").map_err(read)?;
    let correlation: String = row.try_get("correlation_id").map_err(read)?;
    let causation: Option<String> = row.try_get("causation_id").map_err(read)?;
    let version: i32 = row.try_get("version").map_err(read)?;
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
        version: i64::from(version),
        global_position: row.try_get("global_position").map_err(read)?,
        recorded_at: row.try_get("recorded_at").map_err(read)?,
    })
}

impl EventStore for PostgresEventStore {
    async fn read(&self, stream_id: &str) -> Result<Vec<CommittedEvent>, StoreError> {
        let rows = self
            .with_connection(async |connection| {
                query(&format!(
                    "SELECT {EVENT_COLUMNS} FROM events WHERE stream_id = $1 ORDER BY version ASC"
                ))
                .bind(stream_id)
                .fetch_all(&mut *connection)
                .await
            })
            .await
            .map_err(failed("read a stream"))?;
        rows.iter().map(scan).collect()
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
                self.hold_the_log().await?;
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
            // The guard catches a stale expectation; the unique constraint catches the true race, where two
            // transactions both passed the guard. Either is contention, which is a value.
            Err(error)
                if error.source().is_some_and(|source| source.is::<Stale>())
                    || sqlstate(&error).as_deref() == Some("23505") =>
            {
                Ok(AppendResult::VersionConflict {
                    actual_version: self.current_version(stream_id).await?,
                })
            }
            Err(error) => Err(error),
        }
    }

    async fn read_all(&self, from_position: i64, visit: Visit<'_>) -> Result<(), StoreError> {
        // The ceiling is taken once, at the start: an event appended while a long replay runs belongs to the
        // next pass, and a checkpoint that stopped short of it loses nothing.
        let settled = self.settled_position().await?;
        let statement = format!(
            "SELECT {EVENT_COLUMNS} FROM events WHERE global_position >= $1 AND global_position <= $2 \
             ORDER BY global_position ASC LIMIT $3"
        );
        let mut position = from_position;
        while position <= settled {
            let batch = self
                .select(
                    &statement,
                    vec![
                        Argument::Integer(position),
                        Argument::Integer(settled),
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
        Ok(())
    }

    async fn in_unit_of_work<T, F, Fut>(&self, work: F) -> Result<T, StoreError>
    where
        T: Send,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, StoreError>> + Send,
    {
        self.unit_of_work(None, work).await
    }

    /// The last *settled* position, which is the only kind a decision may be guarded at — a boundary past an
    /// event still in flight is one the guard cannot check. Inside a unit of work it is that transaction's own
    /// view instead: a caller reading its own writes, where asking for the settled position would wait on a
    /// lock the transaction itself holds.
    async fn head(&self) -> Result<i64, StoreError> {
        if self.unit().is_some() {
            self.head_now().await
        } else {
            self.settled_position().await
        }
    }

    async fn read_tagged(
        &self,
        query: &TagQuery,
        after: i64,
        until: i64,
    ) -> Result<TaggedRead, StoreError> {
        let ceiling = if until == 0 {
            self.head().await?
        } else {
            until
        };
        let (predicate, arguments) = tag_query_sql(query, 3);
        let statement = format!(
            "SELECT {EVENT_COLUMNS} FROM events e WHERE e.global_position > $1 AND e.global_position <= $2 AND {predicate} \
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
            .unit_of_work(Some("SERIALIZABLE"), || async {
                self.hold_the_log().await?;
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
                    .is_some_and(|source| source.is::<ConditionBroken>())
                    || sqlstate(&error).as_deref() == Some(SERIALIZATION_FAILURE) =>
            {
                Ok(ConditionalAppendResult::ConditionConflict { head })
            }
            Err(error) => Err(error),
        }
    }

    /// Indexes, a batch at a time, what this store's tagging function has not indexed yet. Resumable: a run
    /// over a long log that dies halfway is continued by running it again.
    async fn reindex_tags(&self, from_position: i64) -> Result<usize, StoreError> {
        let tags_of = self.tags_of();
        let statement = format!(
            "SELECT {EVENT_COLUMNS} FROM events WHERE global_position >= $1 \
             AND NOT EXISTS (SELECT 1 FROM event_tags WHERE global_position = events.global_position) \
             ORDER BY global_position ASC LIMIT $2"
        );
        let mut indexed = 0;
        let mut position = from_position;
        loop {
            let (made_findable, next) = self
                .in_unit_of_work(|| async {
                    let unindexed = self.select(&statement, vec![Argument::Integer(position), Argument::Integer(READ_ALL_BATCH)]).await?;
                    let next = unindexed.last().map(|event| event.global_position + 1);
                    let mut made_findable = 0;
                    for event in unindexed {
                        // Counted only when it made the event findable: the alternative is a count that
                        // never reaches zero.
                        let tags = tags_of(&event.event);
                        if tags.is_empty() {
                            continue;
                        }
                        self.with_connection(async |connection| {
                            for tag in &tags {
                                sqlx::query(
                                    "INSERT INTO event_tags (tag, global_position) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                                )
                                .bind(tag)
                                .bind(event.global_position)
                                .execute(&mut *connection)
                                .await?;
                            }
                            Ok(())
                        })
                        .await
                        .map_err(failed("reindex tags"))?;
                        made_findable += 1;
                    }
                    Ok((made_findable, next))
                })
                .await?;
            indexed += made_findable;
            match next {
                Some(next) => position = next,
                None => return Ok(indexed),
            }
        }
    }

    async fn retag(&self, tags_of: TagsOf) -> Result<usize, StoreError> {
        // Inside one unit of work, so the old index is never visible as gone: the whole re-index commits, or
        // the old one stands.
        self.in_unit_of_work(|| async {
            *self
                .tags_of
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = tags_of;
            self.execute("DELETE FROM event_tags", "clear the tag index")
                .await?;
            self.reindex_tags(0).await
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::Barrier;

    use super::*;
    use crate::application::ports::events::{NO_STREAM, TagFilter};
    use crate::event_store_contract;

    // Against a real Postgres, so outside the default suite: `make test-integration` runs these, after
    // `make services-up migrate`, with `DATABASE_URL` pointing at that database. The log is shared with every
    // other case and every earlier run, so each case works in streams and tags of its own.

    fn url() -> String {
        std::env::var("DATABASE_URL").expect("DATABASE_URL names the database to test against")
    }

    async fn store(tags_of: TagsOf) -> Arc<PostgresEventStore> {
        Arc::new(
            PostgresEventStore::connect_tagged(&url(), tags_of)
                .await
                .expect("connect to postgres"),
        )
    }

    fn unique(prefix: &str) -> String {
        format!("{prefix}-{}", uuid::Uuid::new_v4().simple())
    }

    fn event(stream: &str, event_type: &str, payload: serde_json::Value) -> DomainEvent {
        DomainEvent {
            event_type: event_type.to_owned(),
            schema_version: 1,
            stream_id: stream.to_owned(),
            payload: payload.as_object().cloned().unwrap_or_default(),
            occurred_at: "2024-01-01T00:00:00.000000+00:00".to_owned(),
            actor: Actor {
                kind: "test".to_owned(),
                id: "postgres".to_owned(),
            },
            correlation_id: CorrelationId::generate(),
            causation_id: None,
        }
    }

    /// Indexes a claim by the seat it claims, alongside its stream.
    fn seat_tags() -> TagsOf {
        Arc::new(|event: &DomainEvent| {
            let mut tags = vec![crate::application::ports::events::stream_tag(
                &event.stream_id,
            )];
            if let Some(seat) = event.payload.get("seatId").and_then(|seat| seat.as_str()) {
                tags.push(format!("seat:{seat}"));
            }
            tags
        })
    }

    fn by_seat(seat: &str) -> TagQuery {
        TagQuery {
            filters: vec![TagFilter {
                tags: vec![format!("seat:{seat}")],
                types: vec![],
            }],
        }
    }

    #[tokio::test]
    #[ignore = "integration: needs Postgres at DATABASE_URL, migrated"]
    async fn holds_to_the_event_store_contract() {
        let url = url();
        event_store_contract::run(async |tags_of| {
            PostgresEventStore::connect_tagged(&url, tags_of)
                .await
                .expect("connect to postgres")
        })
        .await;
    }

    /// The assertion no infrastructure-free store can make: the in-memory adapter serialises every call and
    /// SQLite serialises writers, so neither could produce two winners and both would pass while proving
    /// nothing. Only a real store genuinely races.
    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    #[ignore = "integration: needs Postgres at DATABASE_URL, migrated"]
    async fn exactly_one_of_many_simultaneous_first_writes_wins() {
        let store = store(default_tags_of()).await;
        let stream = unique("race");
        let gate = Arc::new(Barrier::new(8));
        let attempts: Vec<_> = (0..8)
            .map(|index| {
                let (store, stream, gate) = (store.clone(), stream.clone(), gate.clone());
                tokio::spawn(async move {
                    // Every task waits on the same gate, so they contend rather than queue.
                    gate.wait().await;
                    store
                        .append(
                            &stream,
                            NO_STREAM,
                            vec![event(
                                &stream,
                                &format!("Attempt{index}"),
                                serde_json::Value::Null,
                            )],
                        )
                        .await
                })
            })
            .collect();
        let mut winners = 0;
        for attempt in attempts {
            match attempt.await.expect("task").expect("append") {
                AppendResult::Appended { .. } => winners += 1,
                AppendResult::VersionConflict { actual_version } => assert_eq!(actual_version, 0),
            }
        }
        assert_eq!(winners, 1);
        assert_eq!(store.read(&stream).await.expect("read").len(), 1);
    }

    /// The guarantee the tag boundary rests on: SERIALIZABLE refuses every simultaneous claim but one.
    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    #[ignore = "integration: needs Postgres at DATABASE_URL, migrated"]
    async fn exactly_one_of_many_simultaneous_conditional_appends_wins() {
        let store = store(seat_tags()).await;
        let seat = unique("seat");
        let decided = store
            .read_tagged(&by_seat(&seat), 0, 0)
            .await
            .expect("read the boundary")
            .head;
        let gate = Arc::new(Barrier::new(8));
        let attempts: Vec<_> = (0..8)
            .map(|index| {
                let (store, seat, gate) = (store.clone(), seat.clone(), gate.clone());
                tokio::spawn(async move {
                    gate.wait().await;
                    let claim = format!("claim-{seat}-{index}");
                    let condition = Condition {
                        query: by_seat(&seat),
                        after: decided,
                    };
                    store
                        .append_if(
                            &condition,
                            vec![event(
                                &claim,
                                "SeatClaimed",
                                serde_json::json!({"seatId": seat}),
                            )],
                        )
                        .await
                })
            })
            .collect();
        let mut winners = 0;
        for attempt in attempts {
            if let ConditionalAppendResult::Recorded { .. } =
                attempt.await.expect("task").expect("conditional append")
            {
                winners += 1;
            }
        }
        assert_eq!(winners, 1);
        assert_eq!(
            store
                .read_tagged(&by_seat(&seat), 0, 0)
                .await
                .expect("read")
                .events
                .len(),
            1
        );
    }

    /// The guarantee a projection's single-number checkpoint rests on, and the one bug in this design that
    /// leaves no trace: two appends take 5 and 6, 6 commits first, and a replay that hands out 6 while 5 is in
    /// flight makes a projection record "next is 7" — a view missing a row forever. So a replay waits for the
    /// appends in flight: it finishes only after the slow one commits, holding both events in order.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "integration: needs Postgres at DATABASE_URL, migrated"]
    async fn a_replay_stops_short_of_a_position_an_earlier_event_could_still_arrive_behind() {
        // Three stores on three pools: the slow append holds its transaction open while the others work.
        let (slow, fast, reader) = (
            store(default_tags_of()).await,
            store(default_tags_of()).await,
            store(default_tags_of()).await,
        );
        let (first, second) = (unique("slow"), unique("fast"));
        let start = reader.head().await.expect("head") + 1;
        let (finished_tx, mut finished) = tokio::sync::oneshot::channel();
        slow.in_unit_of_work(|| async {
            slow.append(
                &first,
                NO_STREAM,
                vec![event(&first, "Slow", serde_json::Value::Null)],
            )
            .await?;
            // Its position is taken; its commit is not. This one takes the next position and commits.
            fast.append(
                &second,
                NO_STREAM,
                vec![event(&second, "Fast", serde_json::Value::Null)],
            )
            .await?;
            let replaying = reader.clone();
            tokio::spawn(async move {
                let mut replayed = Vec::new();
                let outcome = replaying
                    .read_all(start, &mut |committed| {
                        replayed.push(committed.event.event_type);
                        Ok(())
                    })
                    .await;
                let _ = finished_tx.send(outcome.map(|()| replayed));
            });
            let early = tokio::time::timeout(Duration::from_secs(1), &mut finished).await;
            assert!(
                early.is_err(),
                "a replay finished while an append was in flight: {early:?}"
            );
            Ok(())
        })
        .await
        .expect("the slow append");
        let replayed = tokio::time::timeout(Duration::from_secs(10), finished)
            .await
            .expect("a replay never finished after the append committed")
            .expect("the replay task")
            .expect("replay");
        let ours: Vec<_> = replayed
            .into_iter()
            .filter(|kind| kind == "Slow" || kind == "Fast")
            .collect();
        assert_eq!(ours, ["Slow", "Fast"]);
    }

    /// The same gap on the write path, where it breaks the constraint instead of a view: a boundary is only
    /// ever a settled position, so a decision drawn while a claim is in flight waits for it, sees it, and a
    /// caller that decided anyway is refused by the condition rather than by luck.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "integration: needs Postgres at DATABASE_URL, migrated"]
    async fn refuses_a_conditional_append_against_an_event_in_flight_when_the_boundary_was_read() {
        let (slow, fast, decider) = (
            store(seat_tags()).await,
            store(seat_tags()).await,
            store(seat_tags()).await,
        );
        let seat = unique("seat");
        let (drawn_tx, mut drawn) = tokio::sync::oneshot::channel();
        slow.in_unit_of_work(|| async {
            let claim = format!("claim-{seat}-1");
            slow.append(
                &claim,
                NO_STREAM,
                vec![event(
                    &claim,
                    "SeatClaimed",
                    serde_json::json!({"seatId": seat}),
                )],
            )
            .await?;
            let other = unique("other");
            fast.append(
                &other,
                NO_STREAM,
                vec![event(&other, "Unrelated", serde_json::Value::Null)],
            )
            .await?;
            let deciding = decider.clone();
            let query = by_seat(&seat);
            tokio::spawn(async move {
                let decision = async {
                    let boundary = deciding.head().await?;
                    let found = deciding.read_tagged(&query, 0, boundary).await?;
                    Ok::<_, StoreError>(
                        found
                            .events
                            .into_iter()
                            .map(|e| e.event.event_type)
                            .collect::<Vec<_>>(),
                    )
                };
                let _ = drawn_tx.send(decision.await);
            });
            let early = tokio::time::timeout(Duration::from_secs(1), &mut drawn).await;
            assert!(
                early.is_err(),
                "a boundary was drawn while an append was in flight: {early:?}"
            );
            Ok(())
        })
        .await
        .expect("the slow append");
        let seen = tokio::time::timeout(Duration::from_secs(10), drawn)
            .await
            .expect("a boundary was never drawn after the append committed")
            .expect("the decision task")
            .expect("decision");
        assert_eq!(seen, ["SeatClaimed"], "the decision should see the claim");
        let claim = format!("claim-{seat}-2");
        let refused = decider
            .append_if(
                &Condition {
                    query: by_seat(&seat),
                    after: 0,
                },
                vec![event(
                    &claim,
                    "SeatClaimed",
                    serde_json::json!({"seatId": seat}),
                )],
            )
            .await
            .expect("conditional append");
        assert!(
            matches!(refused, ConditionalAppendResult::ConditionConflict { .. }),
            "{refused:?}"
        );
    }

    #[tokio::test]
    #[ignore = "integration: needs Postgres at DATABASE_URL, migrated"]
    async fn refuses_to_let_a_committed_event_be_rewritten_or_removed() {
        let store = store(default_tags_of()).await;
        let stream = unique("append-only");
        store
            .append(
                &stream,
                NO_STREAM,
                vec![event(&stream, "Recorded", serde_json::Value::Null)],
            )
            .await
            .expect("append");
        for statement in [
            "UPDATE events SET event_type = 'Rewritten' WHERE stream_id = $1",
            "DELETE FROM events WHERE stream_id = $1",
        ] {
            let refused = sqlx::query(statement)
                .bind(&stream)
                .execute(&store.pool)
                .await;
            let error = refused.expect_err(statement);
            assert!(
                error.to_string().contains("append-only"),
                "{statement} failed for the wrong reason: {error}"
            );
        }
        let recorded = store.read(&stream).await.expect("read");
        assert_eq!(
            recorded
                .iter()
                .map(|e| e.event.event_type.as_str())
                .collect::<Vec<_>>(),
            ["Recorded"]
        );
    }
}
