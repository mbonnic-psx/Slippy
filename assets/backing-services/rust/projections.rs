//! The catch-up runner: the loop that keeps an async read model current, and the ticker that drives it.
//!
//! One pass claims the projection's lease, reads from its checkpoint a batch at a time, and applies each
//! batch and moves the checkpoint in one unit of work — so the view and the position move together or
//! neither does. That is exactly-once with no idempotency key: a crash before a commit leaves both untouched,
//! and the next pass reads the same batch again.
//!
//! No framework owns a loop here, so [`Ticker::run`] is the loop, ended by the shutdown signal it is given;
//! the entry point starts it beside the server. It imports no transport: a project can have an event store and
//! no HTTP adapter at all.

use std::sync::Arc;
use std::time::Duration;

use time::OffsetDateTime;

use crate::application::ports::events::{CommittedEvent, EventStore, StoreError};
use crate::application::ports::read_models::{
    CheckpointStore, FROM_THE_BEGINNING, Lease, Projection,
};

/// How long a pass holds its lease when the runner names no term.
pub const DEFAULT_LEASE: Duration = Duration::from_secs(30);

/// How many events one batch — one transaction — applies when the runner names no size. The size trades how
/// much work a crash repeats against how long one transaction holds its locks.
pub const DEFAULT_BATCH_SIZE: usize = 500;

/// Where "now" comes from: passed in, so a test can choose the time rather than sleep through it.
pub type Clock = Arc<dyn Fn() -> OffsetDateTime + Send + Sync>;

/// The system clock.
pub fn system_clock() -> Clock {
    Arc::new(OffsetDateTime::now_utc)
}

/// Who is running passes, by what clock, holding the lease how long, in batches how big.
#[derive(Clone)]
pub struct Runner {
    pub owner: String,
    pub clock: Clock,
    /// `None` is [`DEFAULT_LEASE`].
    pub ttl: Option<Duration>,
    /// `None` is [`DEFAULT_BATCH_SIZE`].
    pub batch_size: Option<usize>,
}

impl Runner {
    fn ttl(&self) -> Duration {
        self.ttl.unwrap_or(DEFAULT_LEASE)
    }

    fn batch_size(&self) -> usize {
        self.batch_size
            .filter(|size| *size > 0)
            .unwrap_or(DEFAULT_BATCH_SIZE)
    }
}

/// What a pass did: how many events it applied, and whether it held the lease at all. A pass that could not
/// take the lease is the ordinary case for every replica but one, and not a failure — which is why it is
/// reported apart from `applied`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Advance {
    pub applied: usize,
    pub leased: bool,
}

/// Brings one projection up to date, if this runner can take its lease.
pub async fn catch_up(
    store: &impl EventStore,
    checkpoints: &impl CheckpointStore,
    projection: &dyn Projection,
    runner: &Runner,
) -> Result<Advance, StoreError> {
    let Some(lease) = checkpoints
        .claim(
            projection.name(),
            &runner.owner,
            (runner.clock)(),
            runner.ttl(),
        )
        .await?
    else {
        return Ok(Advance::default());
    };
    let applied = apply_from_the_checkpoint(store, checkpoints, projection, &lease, runner).await;
    // Released whether or not the pass held: otherwise one bad batch costs a whole lease before anything tries
    // again, every time. Releasing is politeness, so its own failure changes nothing.
    let _ = checkpoints.release(&lease).await;
    Ok(Advance {
        applied: applied?,
        leased: true,
    })
}

/// Brings every projection up to date, one after another, so one that fails does not stop the rest. The
/// failures come back together, each named for its projection.
pub async fn catch_up_each(
    store: &impl EventStore,
    checkpoints: &impl CheckpointStore,
    projections: &[Arc<dyn Projection>],
    runner: &Runner,
) -> (Vec<(String, Advance)>, Vec<(String, StoreError)>) {
    let mut advances = Vec::new();
    let mut failures = Vec::new();
    for projection in projections {
        match catch_up(store, checkpoints, projection.as_ref(), runner).await {
            Ok(advance) => advances.push((projection.name().to_owned(), advance)),
            Err(error) => failures.push((projection.name().to_owned(), error)),
        }
    }
    (advances, failures)
}

/// Empties the view and folds the whole log into it again — what makes a read model disposable, and a
/// projection bug fixable by changing the fold rather than patching rows. Refuses, applying nothing, when
/// somebody else holds the lease: a rebuild is destructive, so "somebody else is advancing this" has to be
/// distinguishable from "there was nothing to apply".
pub async fn rebuild(
    store: &impl EventStore,
    checkpoints: &impl CheckpointStore,
    projection: &dyn Projection,
    runner: &Runner,
) -> Result<Advance, StoreError> {
    let Some(lease) = checkpoints
        .claim(
            projection.name(),
            &runner.owner,
            (runner.clock)(),
            runner.ttl(),
        )
        .await?
    else {
        return Ok(Advance::default());
    };
    let rebuilt = async {
        store
            .in_unit_of_work(|| async {
                projection.reset().await?;
                checkpoints
                    .record(projection.name(), FROM_THE_BEGINNING)
                    .await
            })
            .await?;
        apply_from_the_checkpoint(store, checkpoints, projection, &lease, runner).await
    }
    .await;
    let _ = checkpoints.release(&lease).await;
    Ok(Advance {
        applied: rebuilt?,
        leased: true,
    })
}

async fn apply_from_the_checkpoint(
    store: &impl EventStore,
    checkpoints: &impl CheckpointStore,
    projection: &dyn Projection,
    lease: &Lease,
    runner: &Runner,
) -> Result<usize, StoreError> {
    let mut applied = 0;
    // Where the last batch left the checkpoint. A store that reads back less than that has lost the write —
    // and a runner that carried on would read the same batch forever, so it stops and says so instead.
    let mut recorded: Option<i64> = None;
    loop {
        let position = checkpoints.position_of(projection.name()).await?;
        if let Some(expected) = recorded
            && position < expected
        {
            return Err(StoreError::new(
                format!("advance {}", projection.name()),
                CheckpointLost {
                    expected,
                    found: position,
                },
            ));
        }
        let batch = take(store, position, runner.batch_size()).await?;
        let Some(last) = batch.last().map(|event| event.global_position) else {
            return Ok(applied);
        };
        store
            .in_unit_of_work(|| async {
                projection.apply(&batch).await?;
                checkpoints.record(projection.name(), last + 1).await
            })
            .await?;
        applied += batch.len();
        recorded = Some(last + 1);
        // Renewed between batches, so a long catch-up keeps its lease; lost, the pass stops and whoever holds
        // it now carries on from the checkpoint this pass left.
        let renewed = checkpoints
            .claim(
                &lease.projection,
                &lease.owner,
                (runner.clock)(),
                runner.ttl(),
            )
            .await?;
        if renewed.is_none() {
            return Ok(applied);
        }
    }
}

/// A checkpoint that read back behind where the pass had just recorded it.
#[derive(Debug, thiserror::Error)]
#[error("the checkpoint read back at {found} after being recorded at {expected}")]
struct CheckpointLost {
    expected: i64,
    found: i64,
}

/// Stops a replay once a batch is full.
#[derive(Debug, thiserror::Error)]
#[error("enough")]
struct Enough;

async fn take(
    store: &impl EventStore,
    from_position: i64,
    count: usize,
) -> Result<Vec<CommittedEvent>, StoreError> {
    let mut batch = Vec::with_capacity(count);
    let replayed = store
        .read_all(from_position, &mut |event| {
            batch.push(event);
            if batch.len() == count {
                Err(StoreError::new("take a batch", Enough))
            } else {
                Ok(())
            }
        })
        .await;
    match replayed {
        Err(error) if !stopped_early(&error) => Err(error),
        _ => Ok(batch),
    }
}

fn stopped_early(error: &StoreError) -> bool {
    std::error::Error::source(error).is_some_and(|source| source.is::<Enough>())
}

/// How long a ticker waits between passes when nothing says otherwise.
pub const DEFAULT_EVERY: Duration = Duration::from_secs(1);

/// What a ticker does with a pass that failed. The pass is not retried early: the next tick retries it.
pub type OnFailure = Arc<dyn Fn(&str, &StoreError) + Send + Sync>;

/// The timer that drives async projections: a pass over every projection, then a pause, until shutdown.
#[derive(Clone, Default)]
pub struct Ticker {
    /// `None` reads `PROJECTIONS_EVERY_MS`, then falls back to [`DEFAULT_EVERY`].
    pub every: Option<Duration>,
    /// `None` names an owner unique to this process and moment.
    pub owner: Option<String>,
    /// `None` is the system clock.
    pub clock: Option<Clock>,
    /// Told about each projection whose pass failed; `None` drops the failure, and the next tick retries.
    pub on_failure: Option<OnFailure>,
}

impl Ticker {
    /// Runs passes until `shutdown` completes. Returns at once when there is nothing to maintain.
    pub async fn run(
        &self,
        store: &impl EventStore,
        checkpoints: &impl CheckpointStore,
        views: &[Arc<dyn Projection>],
        shutdown: impl Future<Output = ()>,
    ) {
        if views.is_empty() {
            return;
        }
        let runner = Runner {
            owner: self.owner(),
            clock: self.clock(),
            ttl: None,
            batch_size: None,
        };
        let pause = self.every();
        tokio::pin!(shutdown);
        loop {
            let (_, failures) = catch_up_each(store, checkpoints, views, &runner).await;
            if let Some(on_failure) = &self.on_failure {
                for (name, error) in &failures {
                    on_failure(name, error);
                }
            }
            tokio::select! {
                () = &mut shutdown => return,
                () = tokio::time::sleep(pause) => {}
            }
        }
    }

    fn owner(&self) -> String {
        self.owner.clone().unwrap_or_else(|| {
            let nanos = OffsetDateTime::now_utc().unix_timestamp_nanos();
            format!("worker-{}-{nanos}", std::process::id())
        })
    }

    fn clock(&self) -> Clock {
        self.clock.clone().unwrap_or_else(system_clock)
    }

    fn every(&self) -> Duration {
        self.every
            .filter(|every| !every.is_zero())
            .or_else(|| every_from(std::env::var("PROJECTIONS_EVERY_MS").ok().as_deref()))
            .unwrap_or(DEFAULT_EVERY)
    }
}

/// The pause `PROJECTIONS_EVERY_MS` names, where it names a usable one.
fn every_from(configured: Option<&str>) -> Option<Duration> {
    configured?
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|millis| *millis > 0)
        .map(Duration::from_millis)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    use time::macros::datetime;

    use super::*;
    use crate::adapters::driven::checkpoint_store_memory::InMemoryCheckpointStore;
    use crate::adapters::driven::event_store_memory::InMemoryEventStore;
    use crate::application::ports::events::{Actor, CorrelationId, DomainEvent, NO_STREAM};
    use crate::application::ports::read_models::Pending;

    const NOW: OffsetDateTime = datetime!(2026-09-10 12:00 UTC);

    fn runner(owner: &str, at: OffsetDateTime) -> Runner {
        Runner {
            owner: owner.to_owned(),
            clock: Arc::new(move || at),
            ttl: None,
            batch_size: None,
        }
    }

    /// A read model with somewhere to put the result, which is all a projection is. Its rows are derived:
    /// every one comes from an event, so `reset` plus a replay reproduces it exactly.
    #[derive(Default)]
    struct Titles {
        name: String,
        rows: Mutex<Vec<String>>,
        batches: Mutex<usize>,
        fail: bool,
    }

    impl Titles {
        fn named(name: &str) -> Arc<Self> {
            Arc::new(Self {
                name: name.to_owned(),
                ..Self::default()
            })
        }

        fn rows(&self) -> Vec<String> {
            self.rows.lock().unwrap().clone()
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("the view write failed")]
    struct ViewFailed;

    impl Projection for Titles {
        fn name(&self) -> &str {
            &self.name
        }

        fn apply<'a>(&'a self, batch: &'a [CommittedEvent]) -> Pending<'a> {
            Box::pin(async move {
                *self.batches.lock().unwrap() += 1;
                self.rows
                    .lock()
                    .unwrap()
                    .extend(batch.iter().map(|event| event.event.event_type.clone()));
                if self.fail {
                    Err(StoreError::new("apply", ViewFailed))
                } else {
                    Ok(())
                }
            })
        }

        fn reset(&self) -> Pending<'_> {
            Box::pin(async move {
                self.rows.lock().unwrap().clear();
                Ok(())
            })
        }
    }

    fn pair() -> (InMemoryEventStore, InMemoryCheckpointStore) {
        let store = InMemoryEventStore::new();
        let checkpoints = InMemoryCheckpointStore::new(&store);
        (store, checkpoints)
    }

    async fn given_a_log_of(store: &InMemoryEventStore, types: &[&str]) {
        let stream = format!("projection-{}", uuid::Uuid::new_v4().simple());
        let correlation_id = CorrelationId::generate();
        let batch = types
            .iter()
            .map(|event_type| DomainEvent {
                event_type: (*event_type).to_owned(),
                schema_version: 1,
                stream_id: stream.clone(),
                payload: serde_json::Map::new(),
                occurred_at: "2024-01-01T00:00:00.000000+00:00".to_owned(),
                actor: Actor {
                    kind: "test".to_owned(),
                    id: "projections".to_owned(),
                },
                correlation_id,
                causation_id: None,
            })
            .collect();
        store
            .append(&stream, NO_STREAM, batch)
            .await
            .expect("append");
    }

    #[tokio::test]
    async fn applies_the_whole_log_and_leaves_the_checkpoint_past_it() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed", "Paid"]).await;
        let view = Titles::named("titles");

        let advance = catch_up(
            &store,
            &checkpoints,
            view.as_ref(),
            &runner("worker-1", NOW),
        )
        .await
        .unwrap();

        assert_eq!(
            advance,
            Advance {
                applied: 2,
                leased: true
            }
        );
        assert_eq!(view.rows(), ["Placed", "Paid"]);
        assert_eq!(checkpoints.position_of("titles").await.unwrap(), 3);
    }

    #[tokio::test]
    async fn resumes_from_the_checkpoint_and_applies_nothing_twice() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed"]).await;
        let view = Titles::named("titles");
        catch_up(
            &store,
            &checkpoints,
            view.as_ref(),
            &runner("worker-1", NOW),
        )
        .await
        .unwrap();

        given_a_log_of(&store, &["Paid"]).await;
        let again = catch_up(
            &store,
            &checkpoints,
            view.as_ref(),
            &runner("worker-1", NOW),
        )
        .await
        .unwrap();

        assert_eq!(again.applied, 1);
        assert_eq!(view.rows().len(), 2);
    }

    #[tokio::test]
    async fn says_a_pass_had_nothing_to_do_without_touching_the_view() {
        let (store, checkpoints) = pair();
        let view = Titles::named("titles");

        let advance = catch_up(
            &store,
            &checkpoints,
            view.as_ref(),
            &runner("worker-1", NOW),
        )
        .await
        .unwrap();

        assert_eq!(
            advance,
            Advance {
                applied: 0,
                leased: true
            }
        );
        assert_eq!(*view.batches.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn applies_a_long_log_in_batches_of_the_size_it_was_given() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["One", "Two", "Three", "Four", "Five"]).await;
        let view = Titles::named("titles");
        let pass = Runner {
            batch_size: Some(2),
            ..runner("worker-1", NOW)
        };

        let advance = catch_up(&store, &checkpoints, view.as_ref(), &pass)
            .await
            .unwrap();

        assert_eq!(advance.applied, 5);
        assert_eq!(*view.batches.lock().unwrap(), 3);
    }

    /// A size of zero is no size: one past the default is two batches, not one unbounded read.
    #[tokio::test]
    async fn applies_in_batches_of_the_default_size_when_given_a_size_of_none() {
        let (store, checkpoints) = pair();
        let log: Vec<String> = (0..=DEFAULT_BATCH_SIZE)
            .map(|index| format!("Event{index}"))
            .collect();
        given_a_log_of(&store, &log.iter().map(String::as_str).collect::<Vec<_>>()).await;
        let view = Titles::named("titles");
        let pass = Runner {
            batch_size: Some(0),
            ..runner("worker-1", NOW)
        };

        let advance = catch_up(&store, &checkpoints, view.as_ref(), &pass)
            .await
            .unwrap();

        assert_eq!(advance.applied, DEFAULT_BATCH_SIZE + 1);
        assert_eq!(*view.batches.lock().unwrap(), 2);
    }

    /// A store whose replay fails: every other call goes to the in-memory store it wraps.
    struct FailingReplay(InMemoryEventStore);

    impl EventStore for FailingReplay {
        async fn read(&self, stream_id: &str) -> Result<Vec<CommittedEvent>, StoreError> {
            self.0.read(stream_id).await
        }
        async fn append(
            &self,
            stream_id: &str,
            expected_version: i64,
            events: Vec<DomainEvent>,
        ) -> Result<crate::application::ports::events::AppendResult, StoreError> {
            self.0.append(stream_id, expected_version, events).await
        }
        async fn read_all(
            &self,
            _: i64,
            _: crate::application::ports::events::Visit<'_>,
        ) -> Result<(), StoreError> {
            Err(StoreError::new("replay", "the connection dropped"))
        }
        async fn in_unit_of_work<T, F, Fut>(&self, work: F) -> Result<T, StoreError>
        where
            T: Send,
            F: FnOnce() -> Fut + Send,
            Fut: Future<Output = Result<T, StoreError>> + Send,
        {
            self.0.in_unit_of_work(work).await
        }
        async fn head(&self) -> Result<i64, StoreError> {
            self.0.head().await
        }
        async fn read_tagged(
            &self,
            query: &crate::application::ports::events::TagQuery,
            after: i64,
            until: i64,
        ) -> Result<crate::application::ports::events::TaggedRead, StoreError> {
            self.0.read_tagged(query, after, until).await
        }
        async fn append_if(
            &self,
            condition: &crate::application::ports::events::Condition,
            events: Vec<DomainEvent>,
        ) -> Result<crate::application::ports::events::ConditionalAppendResult, StoreError>
        {
            self.0.append_if(condition, events).await
        }
        async fn reindex_tags(&self, from_position: i64) -> Result<usize, StoreError> {
            self.0.reindex_tags(from_position).await
        }
        async fn retag(
            &self,
            tags_of: crate::application::ports::events::TagsOf,
        ) -> Result<usize, StoreError> {
            self.0.retag(tags_of).await
        }
    }

    #[tokio::test]
    async fn reports_a_replay_that_failed_rather_than_an_empty_one() {
        let inner = InMemoryEventStore::new();
        let checkpoints = InMemoryCheckpointStore::new(&inner);
        given_a_log_of(&inner, &["Placed"]).await;
        let store = FailingReplay(inner);

        let outcome = catch_up(
            &store,
            &checkpoints,
            Titles::named("titles").as_ref(),
            &runner("worker-1", NOW),
        )
        .await;

        let failure = outcome.expect_err("a failed replay is a failed pass");
        assert!(
            failure.to_string().contains("the connection dropped"),
            "{failure}"
        );
    }

    /// A checkpoint store that loses every position it is asked to record — and gives up after a few reads,
    /// so a runner that failed to notice fails this test rather than spinning in it forever.
    struct Forgetful {
        inner: InMemoryCheckpointStore,
        reads: std::sync::atomic::AtomicUsize,
    }

    impl CheckpointStore for Forgetful {
        async fn position_of(&self, projection: &str) -> Result<i64, StoreError> {
            if self.reads.fetch_add(1, Ordering::Relaxed) > 10 {
                return Err(StoreError::new(
                    "forgetful",
                    "asked for a position too many times",
                ));
            }
            self.inner.position_of(projection).await
        }
        async fn record(&self, _: &str, _: i64) -> Result<(), StoreError> {
            Ok(())
        }
        async fn claim(
            &self,
            projection: &str,
            owner: &str,
            now: OffsetDateTime,
            ttl: Duration,
        ) -> Result<Option<Lease>, StoreError> {
            self.inner.claim(projection, owner, now, ttl).await
        }
        async fn release(&self, lease: &Lease) -> Result<(), StoreError> {
            self.inner.release(lease).await
        }
    }

    #[tokio::test]
    async fn stops_rather_than_spins_when_the_checkpoint_does_not_move() {
        let store = InMemoryEventStore::new();
        let checkpoints = Forgetful {
            inner: InMemoryCheckpointStore::new(&store),
            reads: std::sync::atomic::AtomicUsize::new(0),
        };
        given_a_log_of(&store, &["Placed", "Paid"]).await;
        let pass = Runner {
            batch_size: Some(1),
            ..runner("worker-1", NOW)
        };

        let outcome = catch_up(
            &store,
            &checkpoints,
            Titles::named("titles").as_ref(),
            &pass,
        )
        .await;

        let failure = outcome.expect_err("a lost checkpoint is a failed pass");
        assert!(
            failure.to_string().contains("after being recorded at 2"),
            "{failure}"
        );
    }

    #[tokio::test]
    async fn does_nothing_in_a_second_worker_while_the_first_holds_the_lease() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed"]).await;
        checkpoints
            .claim("titles", "worker-1", NOW, Duration::from_secs(30))
            .await
            .unwrap();
        let view = Titles::named("titles");

        let second = runner("worker-2", NOW + Duration::from_secs(1));
        let advance = catch_up(&store, &checkpoints, view.as_ref(), &second)
            .await
            .unwrap();

        assert_eq!(advance, Advance::default());
        assert!(view.rows().is_empty());
    }

    #[tokio::test]
    async fn takes_over_in_another_worker_once_the_lease_has_lapsed() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed"]).await;
        checkpoints
            .claim("titles", "worker-1", NOW, Duration::from_secs(30))
            .await
            .unwrap();
        let view = Titles::named("titles");

        let later = runner("worker-2", NOW + Duration::from_secs(300));
        let advance = catch_up(&store, &checkpoints, view.as_ref(), &later)
            .await
            .unwrap();

        assert_eq!(
            advance,
            Advance {
                applied: 1,
                leased: true
            }
        );
    }

    #[tokio::test]
    async fn releases_the_lease_when_the_pass_is_done() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed"]).await;
        catch_up(
            &store,
            &checkpoints,
            Titles::named("titles").as_ref(),
            &runner("worker-1", NOW),
        )
        .await
        .unwrap();

        let taken = checkpoints
            .claim("titles", "worker-2", NOW, Duration::from_secs(30))
            .await
            .unwrap();

        assert!(taken.is_some(), "the lease was not released");
    }

    // Exactly-once, with no idempotency key: the checkpoint moves in the same transaction as the rows.
    #[tokio::test]
    async fn moves_the_checkpoint_not_at_all_when_a_batch_fails() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed"]).await;
        let broken = Arc::new(Titles {
            name: "broken".to_owned(),
            fail: true,
            ..Titles::default()
        });

        let outcome = catch_up(
            &store,
            &checkpoints,
            broken.as_ref(),
            &runner("worker-1", NOW),
        )
        .await;

        assert!(
            outcome.is_err(),
            "the view's failure should reach the caller"
        );
        assert_eq!(
            checkpoints.position_of("broken").await.unwrap(),
            FROM_THE_BEGINNING
        );
    }

    #[tokio::test]
    async fn releases_the_lease_even_when_the_pass_fails() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed"]).await;
        let broken = Arc::new(Titles {
            name: "broken".to_owned(),
            fail: true,
            ..Titles::default()
        });
        assert!(
            catch_up(
                &store,
                &checkpoints,
                broken.as_ref(),
                &runner("worker-1", NOW)
            )
            .await
            .is_err()
        );

        let taken = checkpoints
            .claim("broken", "worker-2", NOW, Duration::from_secs(30))
            .await
            .unwrap();

        assert!(taken.is_some(), "a failed pass kept its lease");
    }

    #[tokio::test]
    async fn a_rebuild_empties_the_view_and_folds_the_whole_log_into_it_again() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed", "Paid"]).await;
        let view = Titles::named("titles");
        catch_up(
            &store,
            &checkpoints,
            view.as_ref(),
            &runner("worker-1", NOW),
        )
        .await
        .unwrap();
        view.rows
            .lock()
            .unwrap()
            .push("something nothing derived".to_owned());

        let advance = rebuild(
            &store,
            &checkpoints,
            view.as_ref(),
            &runner("worker-1", NOW),
        )
        .await
        .unwrap();

        assert_eq!(
            advance,
            Advance {
                applied: 2,
                leased: true
            }
        );
        assert_eq!(view.rows(), ["Placed", "Paid"]);
    }

    #[tokio::test]
    async fn a_rebuild_that_cannot_take_the_lease_leaves_the_view_alone() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed"]).await;
        let view = Titles::named("titles");
        catch_up(
            &store,
            &checkpoints,
            view.as_ref(),
            &runner("worker-1", NOW),
        )
        .await
        .unwrap();
        checkpoints
            .claim("titles", "worker-1", NOW, Duration::from_secs(30))
            .await
            .unwrap();

        let rival = runner("worker-2", NOW + Duration::from_secs(1));
        let advance = rebuild(&store, &checkpoints, view.as_ref(), &rival)
            .await
            .unwrap();

        assert_eq!(advance, Advance::default());
        assert_eq!(view.rows().len(), 1);
    }

    /// A view that, while it applies, has a rival try to take its lease — the only moment the lease can be
    /// observed, because the view is the only code the runner calls while holding it.
    struct Contended {
        titles: Arc<Titles>,
        checkpoints: Arc<InMemoryCheckpointStore>,
        rival_at: OffsetDateTime,
        stolen: AtomicBool,
    }

    impl Projection for Contended {
        fn name(&self) -> &str {
            self.titles.name()
        }

        fn apply<'a>(&'a self, batch: &'a [CommittedEvent]) -> Pending<'a> {
            Box::pin(async move {
                let rival = self
                    .checkpoints
                    .claim(
                        self.name(),
                        "worker-2",
                        self.rival_at,
                        Duration::from_secs(30),
                    )
                    .await?;
                self.stolen.fetch_or(rival.is_some(), Ordering::Relaxed);
                self.titles.apply(batch).await
            })
        }

        fn reset(&self) -> Pending<'_> {
            self.titles.reset()
        }
    }

    async fn contended(rival_after: Duration, ttl: Option<Duration>) -> (Advance, bool) {
        let store = InMemoryEventStore::new();
        let checkpoints = Arc::new(InMemoryCheckpointStore::new(&store));
        given_a_log_of(&store, &["Placed"]).await;
        let view = Contended {
            titles: Titles::named("titles"),
            checkpoints: checkpoints.clone(),
            rival_at: NOW + rival_after,
            stolen: AtomicBool::new(false),
        };
        let pass = Runner {
            ttl,
            ..runner("worker-1", NOW)
        };
        let advance = catch_up(&store, &checkpoints, &view, &pass).await.unwrap();
        (advance, view.stolen.load(Ordering::Relaxed))
    }

    #[tokio::test]
    async fn keeps_its_lease_through_a_pass_when_it_was_given_no_term() {
        let (advance, stolen) = contended(Duration::from_secs(1), None).await;
        assert!(
            !stolen,
            "a second worker took the lease from a pass still applying a batch"
        );
        assert_eq!(
            advance,
            Advance {
                applied: 1,
                leased: true
            }
        );
    }

    #[tokio::test]
    async fn keeps_its_lease_for_the_term_it_was_given() {
        // Longer than DEFAULT_LEASE, so a runner that ignored the term it was given would lose the lease.
        let (advance, stolen) =
            contended(Duration::from_secs(120), Some(Duration::from_secs(300))).await;
        assert!(
            !stolen,
            "a second worker took a lease the runner asked to hold for five minutes"
        );
        assert_eq!(
            advance,
            Advance {
                applied: 1,
                leased: true
            }
        );
    }

    #[tokio::test]
    async fn a_ticker_catches_a_projection_up_without_anybody_calling_it() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed"]).await;
        let view = Titles::named("titles");
        let views: Vec<Arc<dyn Projection>> = vec![view.clone()];
        let ticker = Ticker {
            every: Some(Duration::from_millis(5)),
            ..Ticker::default()
        };

        let applied = async {
            while view.rows().is_empty() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        };
        tokio::time::timeout(
            Duration::from_secs(5),
            ticker.run(&store, &checkpoints, &views, applied),
        )
        .await
        .expect("the ticker never applied the log");

        assert_eq!(view.rows(), ["Placed"]);
    }

    #[tokio::test]
    async fn a_ticker_keeps_going_and_reports_when_a_projection_fails() {
        let (store, checkpoints) = pair();
        given_a_log_of(&store, &["Placed"]).await;
        let broken = Arc::new(Titles {
            name: "broken".to_owned(),
            fail: true,
            ..Titles::default()
        });
        let healthy = Titles::named("healthy");
        let views: Vec<Arc<dyn Projection>> = vec![broken, healthy.clone()];
        let reported = Arc::new(Mutex::new(Vec::new()));
        let on_failure: OnFailure = {
            let reported = reported.clone();
            Arc::new(move |name: &str, _: &StoreError| {
                reported.lock().unwrap().push(name.to_owned())
            })
        };
        let ticker = Ticker {
            every: Some(Duration::from_millis(5)),
            on_failure: Some(on_failure),
            ..Ticker::default()
        };

        let done = async {
            while healthy.rows().is_empty() || reported.lock().unwrap().is_empty() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        };
        tokio::time::timeout(
            Duration::from_secs(5),
            ticker.run(&store, &checkpoints, &views, done),
        )
        .await
        .expect("stuck");

        assert!(reported.lock().unwrap().iter().all(|name| name == "broken"));
        assert_eq!(healthy.rows(), ["Placed"]);
    }

    #[test]
    fn a_ticker_pauses_for_the_interval_the_environment_names_and_falls_back_otherwise() {
        assert_eq!(every_from(Some("250")), Some(Duration::from_millis(250)));
        assert_eq!(every_from(Some("600000")), Some(Duration::from_secs(600)));
        for unusable in [None, Some(""), Some("0"), Some("-5"), Some("soon")] {
            assert_eq!(every_from(unusable), None, "{unusable:?}");
        }
        let explicit = Ticker {
            every: Some(Duration::from_millis(7)),
            ..Ticker::default()
        };
        assert_eq!(explicit.every(), Duration::from_millis(7));
    }

    #[test]
    fn a_ticker_names_an_owner_of_its_own_when_given_none() {
        let named = Ticker {
            owner: Some("worker-9".to_owned()),
            ..Ticker::default()
        };
        assert_eq!(named.owner(), "worker-9");
        assert!(
            Ticker::default()
                .owner()
                .starts_with(&format!("worker-{}-", std::process::id()))
        );
    }
}
