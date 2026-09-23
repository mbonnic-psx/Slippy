```rust
use time::OffsetDateTime;
use uuid::Uuid;

/// Which whole business transaction a message belongs to, and what directly caused it. Two types rather than
/// one used twice: they sit side by side below and are both UUIDs underneath, so swapping them would compile
/// and destroy the one thing they exist for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorrelationId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CausationId(pub Uuid);

/// A stored domain event with the metadata the store and its consumers need and the payload does not carry.
#[derive(Debug, Clone, PartialEq)]
pub struct EventEnvelope<Data> {
    /// Uniquely identifies this event — for idempotency, and as the causation of events it later causes.
    pub id: Uuid,
    /// The event's type name, a string so a reader that does not yet know a newer type can still read it.
    pub event_type: String,
    /// The aggregate instance this event belongs to.
    pub stream_id: StreamId,
    /// This event's position within its own stream, for optimistic concurrency.
    pub version: u64,
    /// This event's position in the store-wide order, for resuming a catch-up read.
    pub global_position: u64,
    /// Assigned by the store, in UTC.
    pub timestamp: OffsetDateTime,
    /// The domain payload.
    pub data: Data,
    /// Cross-cutting, non-domain context.
    pub metadata: EventMetadata,
}

/// Tracing context for an event. A real system commonly adds a user, a tenant or a schema version.
#[derive(Debug, Clone, PartialEq)]
pub struct EventMetadata {
    /// Ties every message in one business transaction together.
    pub correlation_id: CorrelationId,
    /// The message that directly caused this event; `None` when nothing did.
    pub causation_id: Option<CausationId>,
}
```
