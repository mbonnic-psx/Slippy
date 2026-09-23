//! The event-store port and the envelope it carries.
//!
//! Five capabilities, and a store that cannot offer all five must not be adopted:
//!
//! 1. append with an expected version (optimistic concurrency, one stream at a time)
//! 2. ordered reads of a single stream
//! 3. replay from position zero, across all streams
//! 4. reads by tag query, which return the store head as well as the events
//! 5. append conditional on a tag query — the same guarantee as (1) over a boundary that is not one
//!    stream
//!
//! The last two are the Dynamic Consistency Boundary, and they are additive: a stream and its expected
//! version remain the default boundary, and every slice generated so far uses nothing else. A tag is the
//! more general of the two — stream-per-aggregate is the case where every event carries exactly one tag,
//! `stream:<stream id>`, which is what [`default_tags_of`] returns.
//!
//! Nothing here names Postgres, SQL, or a vendor. Every adapter under `adapters::driven` implements it,
//! and the same contract suite runs against all of them.

use std::fmt;
use std::future::Future;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

/// The expected version for a stream that must not exist yet, which is how first-write races are
/// detected.
pub const NO_STREAM: i64 = -1;

/// Who or what caused the event. Recorded on every event so the log answers "who did this".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub kind: String,
    pub id: String,
}

/// A correlation or causation id that was not a UUID.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("a {what} must be a UUID, and {raw:?} is not")]
pub struct NotAUuid {
    what: &'static str,
    raw: String,
}

/// The whole business transaction an event belongs to — a **UUID**, in a type of its own.
///
/// Written by one service and read by another, often years later by a tool nobody has written yet, so the
/// one thing it must be is unambiguous: a UUID is unique without a registry, parses the same everywhere,
/// and cannot quietly become a request path, a customer reference or an empty string nothing rejects. Two
/// types rather than one used twice, because correlation and causation sit side by side in every event
/// and are both UUIDs underneath: were they one type, swapping them would compile, and it destroys the one
/// thing they exist for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorrelationId(Uuid);

/// The message that directly caused an event. The rule, from Greg Young: responding to a message, copy
/// its correlation id as your own and take its id as your causation id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CausationId(Uuid);

fn parse_uuid(raw: &str, what: &'static str) -> Result<Uuid, NotAUuid> {
    // Hyphenated form only, so one id has one spelling on the way in as well as the way out.
    if raw.len() != 36 {
        return Err(NotAUuid {
            what,
            raw: raw.to_owned(),
        });
    }
    Uuid::parse_str(raw).map_err(|_| NotAUuid {
        what,
        raw: raw.to_owned(),
    })
}

impl CorrelationId {
    /// Parses an incoming correlation id. Parsing at the edge is what keeps the type honest: a value that
    /// arrived as text — a header, a queue message, a stored row — is checked once, here.
    pub fn parse(raw: &str) -> Result<Self, NotAUuid> {
        parse_uuid(raw, "correlation id").map(Self)
    }

    /// A fresh correlation id, to start a new business transaction.
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for CorrelationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl CausationId {
    /// Parses the id of the message that directly caused an event.
    pub fn parse(raw: &str) -> Result<Self, NotAUuid> {
        parse_uuid(raw, "causation id").map(Self)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for CausationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

// Lower-case and hyphenated, so one id has one spelling: Postgres hands back the canonical form whatever
// went in, and a store that kept a caller's casing would compare unequal to the same id read back.
impl fmt::Display for CorrelationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hyphenated().fmt(formatter)
    }
}

impl fmt::Display for CausationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hyphenated().fmt(formatter)
    }
}

/// A fact the domain decided on, before any store has seen it.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainEvent {
    pub event_type: String,
    pub schema_version: i32,
    pub stream_id: String,
    pub payload: Map<String, Value>,
    /// Domain time, as an ISO-8601 instant. Taken from a clock at the edge and passed in as a typed input —
    /// the domain never reads the clock, which is what makes a decision testable without freezing time.
    /// Distinct from `recorded_at`, and conflating the two is how clock problems become unreconcilable.
    pub occurred_at: String,
    pub actor: Actor,
    /// The whole business transaction this event belongs to.
    pub correlation_id: CorrelationId,
    /// The message that directly caused this event, and `None` when nothing did — the first event of a
    /// transaction has no cause to point at, and inventing one would be a lie about the shape of the tree.
    pub causation_id: Option<CausationId>,
}

/// What the store gives back: the event plus the facts only the store knows.
#[derive(Debug, Clone, PartialEq)]
pub struct CommittedEvent {
    pub event: DomainEvent,
    pub version: i64,
    pub global_position: i64,
    pub recorded_at: String,
}

/// The outcome of an append.
///
/// A version conflict is a **value, not an error**. Contention is expected under load, not exceptional:
/// the caller re-reads and re-decides. Returning it as an error would put ordinary concurrency on the error
/// path, where the next reader cannot tell it from a broken connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendResult {
    /// The events were written; the stream's head is now `version`.
    Appended { version: i64 },
    /// The stream was not where the caller thought it was; it is at `actual_version`.
    VersionConflict { actual_version: i64 },
}

// ── The other consistency boundary ─────────────────────────────────────────────────────────────────
//
// Everything from here to `EventStore` is the Dynamic Consistency Boundary. Read it only when a decision
// cannot be guarded by one stream's version — a constraint spanning two entities, where either can change
// under you. Until then `append` is the whole port.

/// The tag every event carries by default: its own stream, as a tag.
///
/// A function rather than an inlined string because it is the one place the correspondence between the two
/// boundaries is spelled: a stream id *is* a tag, so a log that indexes tags indexes the streams it had.
pub fn stream_tag(stream_id: &str) -> String {
    format!("stream:{stream_id}")
}

/// How an adapter learns what to index an event by. A plain function, so a project's tagging rule is
/// testable without a database.
pub type TagsOf = Arc<dyn Fn(&DomainEvent) -> Vec<String> + Send + Sync>;

/// What an event is indexed by, unless this project says otherwise.
///
/// Tags are derived, never modelled: nothing in `docs/event-model/model.yaml` names one, because a tag is a
/// technical index over the log and not a fact about the business. A project's own tagging function is
/// where it decides what its events are findable by — usually the payload's identifying attributes,
/// alongside the stream. Pass it to the adapter's constructor. Changing it later is safe and cheap — the
/// index is derived, so `retag` rebuilds it from the log.
pub fn default_tags_of() -> TagsOf {
    Arc::new(|event: &DomainEvent| vec![stream_tag(&event.stream_id)])
}

/// One conjunction: an event matches when it carries every tag named here and, where `types` names any,
/// when its type is one of them. A filter naming neither matches nothing — see [`TagQuery`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagFilter {
    pub tags: Vec<String>,
    pub types: Vec<String>,
}

impl TagFilter {
    /// Whether one event matches this filter, given the tags it was indexed by.
    ///
    /// In the port rather than in each adapter because it is the definition, and an adapter that computed
    /// it differently in SQL would be a second definition nothing compares. The in-memory adapter uses this
    /// directly; the SQL adapters translate it, and the contract suite holds the translation to it.
    pub fn matches(&self, event: &CommittedEvent, tags: &[String]) -> bool {
        if self.tags.is_empty() && self.types.is_empty() {
            return false;
        }
        if !self.types.is_empty() && !self.types.contains(&event.event.event_type) {
            return false;
        }
        self.tags.iter().all(|tag| tags.contains(tag))
    }
}

/// Which events a decision loads, and — as part of a [`Condition`] — which events invalidate it.
///
/// A disjunction of filters, because the constraint that motivates any of this spans entities: deciding
/// whether a student may subscribe to a course needs the course's events *and* that student's, which is
/// two filters and cannot be one. An empty query matches nothing, never everything: a condition that
/// matched everything would refuse every concurrent append in the system, and a read that matched
/// everything would quietly become a full replay.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagQuery {
    pub filters: Vec<TagFilter>,
}

impl TagQuery {
    /// Whether any of the query's filters matches the event.
    pub fn matches(&self, event: &CommittedEvent, tags: &[String]) -> bool {
        self.filters
            .iter()
            .any(|filter| filter.matches(event, tags))
    }
}

/// What a decision was made from, and the position it was made at.
///
/// `head` is the store's last global position at the moment of the read — zero for an empty log. It is the
/// anchor a [`Condition`] is built from, so "these are the facts I decided on" and "nothing else has
/// happened since" are one round trip rather than two that can disagree.
#[derive(Debug, Clone, PartialEq)]
pub struct TaggedRead {
    pub events: Vec<CommittedEvent>,
    pub head: i64,
}

/// The guard on a conditional append: `query` must have matched nothing after `after`.
///
/// `after` is the head of the [`TaggedRead`] the decision was made from. The pair is the boundary —
/// dynamic because the caller draws it per decision, out of tags, rather than inheriting it from how
/// streams were laid out months earlier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
    pub query: TagQuery,
    pub after: i64,
}

/// The outcome of a conditional append. `head` is the store head as found, so a caller that retries reads
/// from there. A conflict is a value here for the same reason a version conflict is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalAppendResult {
    Recorded { head: i64 },
    ConditionConflict { head: i64 },
}

/// A genuine failure — a lost connection, a corrupt row — and never a version conflict, which is a value.
///
/// It carries its cause without naming it, so nothing that consumes the port learns which database is
/// behind it.
#[derive(Debug, thiserror::Error)]
#[error("{context}: {source}")]
pub struct StoreError {
    context: String,
    #[source]
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl StoreError {
    /// A failure with the operation it happened in, and what caused it.
    pub fn new(
        context: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self {
            context: context.into(),
            source: source.into(),
        }
    }
}

/// What `read_all` hands each event to, in order; an `Err` stops the replay and is returned by it.
pub type Visit<'a> = &'a mut (dyn FnMut(CommittedEvent) -> Result<(), StoreError> + Send);

/// The port. An `Err` is a genuine failure — a lost connection, a corrupt row — never a version conflict.
pub trait EventStore: Send + Sync {
    fn read(
        &self,
        stream_id: &str,
    ) -> impl Future<Output = Result<Vec<CommittedEvent>, StoreError>> + Send;

    fn append(
        &self,
        stream_id: &str,
        expected_version: i64,
        events: Vec<DomainEvent>,
    ) -> impl Future<Output = Result<AppendResult, StoreError>> + Send;

    /// Replays across all streams from a global position, handing each event to `visit` in order and
    /// stopping if it returns an `Err`.
    ///
    /// This exists so read models can be rebuilt from zero — without it they are not disposable, and a
    /// projection bug becomes unfixable. A visitor rather than a returned `Vec`, so a rebuild over a long
    /// log does not materialise the whole thing in memory.
    fn read_all(
        &self,
        from_position: i64,
        visit: Visit<'_>,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;

    /// Runs `work` in one transaction, which an append and somebody else's write share.
    ///
    /// Inside `work`, `append` and `append_if` do not commit: an `Ok` commits everything once, and an `Err`
    /// rolls all of it back. Outside it they commit themselves, exactly as they always have. The
    /// transaction travels with the task that runs `work`, so every call `work` makes to this store — or to
    /// a checkpoint store built from it — is part of it. Two things need this, and they are the same need.
    /// A read model materialised inline is written here, so a query can never see an event whose view row
    /// is missing. A projection maintained asynchronously records its checkpoint here, in the same
    /// transaction as the rows it derived, which is what makes it exactly-once rather than
    /// approximately-once. A checkpoint committed separately from the view it describes is not a
    /// checkpoint; it is a race with a number in it.
    ///
    /// Nesting is allowed: the outermost call owns the commit.
    fn in_unit_of_work<T, F, Fut>(
        &self,
        work: F,
    ) -> impl Future<Output = Result<T, StoreError>> + Send
    where
        T: Send,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, StoreError>> + Send;

    /// The last global position in the log, or zero when it is empty.
    ///
    /// The boundary a decision is made against, taken once and then handed to every read that decision
    /// needs as `until`, so two reads see the same log and a [`Condition`] built from it covers exactly the
    /// facts it was decided on.
    fn head(&self) -> impl Future<Output = Result<i64, StoreError>> + Send;

    /// The events matching `query` in `(after, until]`, and the position they are as of.
    ///
    /// This is the DCB read: what a decision loads. `after` is for resuming a long read, not for the guard.
    /// `until` is the ceiling — zero for none — and the head handed back is `until` when it is given and the
    /// store's head when it is not.
    fn read_tagged(
        &self,
        query: &TagQuery,
        after: i64,
        until: i64,
    ) -> impl Future<Output = Result<TaggedRead, StoreError>> + Send;

    /// Appends only if nothing matching `condition.query` was recorded after `condition.after`.
    ///
    /// The events still name their streams and still land at gapless per-stream versions, so `read`, folds
    /// and every slice written against `append` keep working unchanged. What differs is only what the write
    /// is guarded by.
    fn append_if(
        &self,
        condition: &Condition,
        events: Vec<DomainEvent>,
    ) -> impl Future<Output = Result<ConditionalAppendResult, StoreError>> + Send;

    /// Indexes events this store's tagging function has not indexed yet, and reports how many were made
    /// findable by tag. Idempotent, so a run that died halfway is resumed by running it again.
    fn reindex_tags(
        &self,
        from_position: i64,
    ) -> impl Future<Output = Result<usize, StoreError>> + Send;

    /// Adopts a new tagging function and rebuilds the whole index under it, reporting how many events were
    /// indexed. Both halves happen together: an index rebuilt under one function while appends carry on
    /// under another is an index that disagrees with itself.
    fn retag(&self, tags_of: TagsOf) -> impl Future<Output = Result<usize, StoreError>> + Send;
}

/// The expected version to pass when appending to a stream just read.
pub fn current_version(events: &[CommittedEvent]) -> i64 {
    events.last().map_or(NO_STREAM, |event| event.version)
}

/// A shared store is the store: the composition root hands one store to everything that needs it, and the
/// checkpoint store built from it holds it too.
impl<S: EventStore> EventStore for Arc<S> {
    fn read(
        &self,
        stream_id: &str,
    ) -> impl Future<Output = Result<Vec<CommittedEvent>, StoreError>> + Send {
        (**self).read(stream_id)
    }

    fn append(
        &self,
        stream_id: &str,
        expected_version: i64,
        events: Vec<DomainEvent>,
    ) -> impl Future<Output = Result<AppendResult, StoreError>> + Send {
        (**self).append(stream_id, expected_version, events)
    }

    fn read_all(
        &self,
        from_position: i64,
        visit: Visit<'_>,
    ) -> impl Future<Output = Result<(), StoreError>> + Send {
        (**self).read_all(from_position, visit)
    }

    fn in_unit_of_work<T, F, Fut>(
        &self,
        work: F,
    ) -> impl Future<Output = Result<T, StoreError>> + Send
    where
        T: Send,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, StoreError>> + Send,
    {
        (**self).in_unit_of_work(work)
    }

    fn head(&self) -> impl Future<Output = Result<i64, StoreError>> + Send {
        (**self).head()
    }

    fn read_tagged(
        &self,
        query: &TagQuery,
        after: i64,
        until: i64,
    ) -> impl Future<Output = Result<TaggedRead, StoreError>> + Send {
        (**self).read_tagged(query, after, until)
    }

    fn append_if(
        &self,
        condition: &Condition,
        events: Vec<DomainEvent>,
    ) -> impl Future<Output = Result<ConditionalAppendResult, StoreError>> + Send {
        (**self).append_if(condition, events)
    }

    fn reindex_tags(
        &self,
        from_position: i64,
    ) -> impl Future<Output = Result<usize, StoreError>> + Send {
        (**self).reindex_tags(from_position)
    }

    fn retag(&self, tags_of: TagsOf) -> impl Future<Output = Result<usize, StoreError>> + Send {
        (**self).retag(tags_of)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_correlation_id_is_a_uuid_in_one_spelling() {
        let raw = "018F3A2B-6C41-7C9D-9F0E-2A5B7C1D4E84";
        let id = CorrelationId::parse(raw).expect("a UUID parses");
        assert_eq!(
            id.to_string(),
            raw.to_lowercase(),
            "one id has one spelling"
        );
        assert_eq!(id.as_uuid().to_string(), raw.to_lowercase());
        assert_eq!(CorrelationId::from(id.as_uuid()), id);
    }

    #[test]
    fn a_causation_id_is_a_uuid_in_one_spelling() {
        let raw = "018f3a2b-6c41-7c9d-9f0e-2a5b7c1d4e84";
        let id = CausationId::parse(raw).expect("a UUID parses");
        assert_eq!(id.to_string(), raw);
        assert_eq!(id.as_uuid().to_string(), raw);
        assert_eq!(CausationId::from(id.as_uuid()), id);
    }

    #[test]
    fn anything_but_a_hyphenated_uuid_is_refused() {
        for raw in [
            "",
            "not-a-uuid",
            "018f3a2b6c417c9d9f0e2a5b7c1d4e84",
            "{018f3a2b-6c41-7c9d-9f0e-2a5b7c1d4e84}",
        ] {
            let refusal = CorrelationId::parse(raw).expect_err("not a UUID");
            assert!(
                refusal
                    .to_string()
                    .contains("correlation id must be a UUID"),
                "{refusal}"
            );
            assert!(CausationId::parse(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn a_stream_is_the_tag_its_events_carry() {
        assert_eq!(stream_tag("account-1"), "stream:account-1");
        let event = DomainEvent {
            event_type: "Opened".to_owned(),
            schema_version: 1,
            stream_id: "account-1".to_owned(),
            payload: Map::new(),
            occurred_at: "2024-01-01T00:00:00Z".to_owned(),
            actor: Actor {
                kind: "test".to_owned(),
                id: "tags".to_owned(),
            },
            correlation_id: CorrelationId::generate(),
            causation_id: None,
        };
        assert_eq!(default_tags_of()(&event), ["stream:account-1"]);
    }
}
