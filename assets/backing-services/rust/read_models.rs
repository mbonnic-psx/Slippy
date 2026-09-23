//! The read-side port: where a projection has got to, and who is allowed to advance it.
//!
//! The write side of this project arrives finished — a port, its adapters, one contract suite. This is the
//! other half, and it exists because the half that was missing is the half that gets invented badly: a read
//! model folded in the request because that was the only read path already built.
//!
//! Two ports, and they are deliberately separate:
//!
//! - [`CheckpointStore`] — how far a named projection has consumed the log, and an exclusive lease so
//!   exactly one worker advances it. This is infrastructure, and it ships with adapters.
//! - [`Projection`] — what a project's own view *is*: a fold with somewhere to put the result. This ships as
//!   a trait only, because the rows are the project's.
//!
//! A checkpoint is only a checkpoint if it commits with the rows it describes. Everything here is shaped by
//! that one sentence: `record` never commits on its own, and the runner in `crate::projections` calls it
//! inside `EventStore::in_unit_of_work`, so the position and the view move together or neither moves.
//! Record it separately and you have chosen, without noticing, between losing updates and applying them
//! twice.
//!
//! `docs/event-model/README.md` says which of the three lifecycles — live, inline, async — a slice's read
//! model uses, and `make check-model` makes a slice say so before it can be planned. This module is what
//! async is built from, and the unit of work is what inline is built from.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use time::OffsetDateTime;

use super::events::{CommittedEvent, StoreError};

/// Where a projection that has never run starts. The position is the *next* global position to apply, so
/// zero means "the whole log", and a rebuild is a `record` of zero followed by a catch-up.
pub const FROM_THE_BEGINNING: i64 = 0;

/// The right to advance one projection, held by one worker until it expires.
///
/// An expiry rather than a lock, because the failure to survive is a worker that dies holding it: a lock
/// nothing releases is a projection that never advances again, and the first anybody hears of it is a stale
/// view. A lease that lapses is a projection that resumes on its own.
///
/// The lease is an efficiency measure, not a correctness one. Two workers holding it at once would each
/// apply events inside a unit of work, and the checkpoint moving in the same transaction is what makes the
/// loser's work a no-op rather than a duplicate. The lease is what stops that being the normal case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lease {
    pub projection: String,
    pub owner: String,
    pub expires_at: OffsetDateTime,
}

/// Where each projection has got to, and who currently owns advancing it.
///
/// Every adapter is built from the event store it follows, deliberately: `record` has to land in the same
/// transaction as the view write, and two adapters on two connections cannot do that however carefully
/// they are called.
pub trait CheckpointStore: Send + Sync {
    /// The next global position this projection has not yet applied.
    ///
    /// [`FROM_THE_BEGINNING`] for a projection nothing has recorded — an unknown projection is one that has
    /// consumed nothing, not an error, because a rebuild has to be expressible without a special case.
    fn position_of(&self, projection: &str)
    -> impl Future<Output = Result<i64, StoreError>> + Send;

    /// Moves the checkpoint. It does not commit — the caller's unit of work does.
    ///
    /// Call it inside the same unit of work as the writes it accounts for, always. It is safe to call
    /// outside one, and then it commits on its own, which is exactly the mistake this comment exists to
    /// name.
    fn record(
        &self,
        projection: &str,
        position: i64,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;

    /// Takes or renews the lease, and answers `None` if somebody else holds an unexpired one.
    ///
    /// `now` is passed in rather than read from a clock, for the reason every instant in this project is: a
    /// test that cannot choose the time cannot test expiry without sleeping. The owner already holding the
    /// lease always succeeds, which is how a long catch-up renews.
    fn claim(
        &self,
        projection: &str,
        owner: &str,
        now: OffsetDateTime,
        ttl: Duration,
    ) -> impl Future<Output = Result<Option<Lease>, StoreError>> + Send;

    /// Gives the lease up early. A lease held by somebody else is left alone. Releasing is politeness, not
    /// correctness: not releasing costs the next worker one ttl.
    fn release(&self, lease: &Lease) -> impl Future<Output = Result<(), StoreError>> + Send;
}

/// A read model that is maintained rather than folded per query — inline or async.
///
/// Three requirements, and the third is the one usually missed:
///
/// - `name` identifies its checkpoint. Stable for the life of the projection: renaming it silently starts a
///   second projection at position zero.
/// - `apply` folds a batch into whatever the view is kept in. Under async it is called inside the store's
///   unit of work, so it must write through it and must not commit; under inline the append's transaction
///   is the one it joins.
/// - `reset` empties the view. Without it the projection is not disposable, and a projection that cannot be
///   rebuilt is a second source of truth — which is the whole thing an event log exists to avoid.
///
/// Every row a projection writes is derived: it comes from the events, and nothing else. A column the
/// projection alone knows — a `done_at` the worker stamps when it processes the row — cannot survive
/// `reset`, and finding that out during an incident is how a rebuild becomes data loss. Where a value like
/// that is needed, it belongs in an event.
///
/// Its futures are boxed, unlike the rest of the port, so that one runner can hold every projection a service
/// has in one list — `Vec<Arc<dyn Projection>>` — whatever each one's type. Implement it with
/// `Box::pin(async move { ... })`.
pub trait Projection: Send + Sync {
    fn name(&self) -> &str;

    fn apply<'a>(&'a self, batch: &'a [CommittedEvent]) -> Pending<'a>;

    fn reset(&self) -> Pending<'_>;
}

/// What a [`Projection`] hands back: the work, to be awaited, and whether it held.
pub type Pending<'a> = Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

/// A shared checkpoint store is the checkpoint store, for the reason a shared event store is.
impl<C: CheckpointStore> CheckpointStore for std::sync::Arc<C> {
    fn position_of(
        &self,
        projection: &str,
    ) -> impl Future<Output = Result<i64, StoreError>> + Send {
        (**self).position_of(projection)
    }

    fn record(
        &self,
        projection: &str,
        position: i64,
    ) -> impl Future<Output = Result<(), StoreError>> + Send {
        (**self).record(projection, position)
    }

    fn claim(
        &self,
        projection: &str,
        owner: &str,
        now: OffsetDateTime,
        ttl: Duration,
    ) -> impl Future<Output = Result<Option<Lease>, StoreError>> + Send {
        (**self).claim(projection, owner, now, ttl)
    }

    fn release(&self, lease: &Lease) -> impl Future<Output = Result<(), StoreError>> + Send {
        (**self).release(lease)
    }
}
