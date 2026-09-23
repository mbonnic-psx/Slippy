```rust
// The model says which attributes identify something; this is that table, transcribed once.
//
//   - id: S7
//     frames:
//       - type: evt
//         name: SeatClaimed
//         attributes:
//           - { name: seatId,   identifies: seat }
//           - { name: fromHold, identifies: hold }
//           - { name: toHold,   identifies: hold }
//           - { name: claimedAt, type: instant }
//
// Two attributes identifying the same kind is the case worth noticing: a transfer carries two `hold:` tags,
// and a query for either finds the event. A design that mapped one kind to one attribute could not express
// it, and asking such an index "which hold?" has no answer.

/// What each event's payload identifies, keyed by event type: the attribute, and the kind it is a tag for.
///
/// Transcribed from docs/event-model/model.yaml — the model is the source, this is the copy the store can
/// execute, and `make check-model` is what keeps an event's name honest between them.
const IDENTIFIES: &[(&str, &[(&str, &str)])] = &[
    ("SeatClaimed", &[("seatId", "seat"), ("fromHold", "hold"), ("toHold", "hold")]),
    ("SeatReleased", &[("seatId", "seat")]),
];

/// Every tag an event is findable by: its own stream, plus what it identifies.
///
/// The stream tag stays, always. It is what makes the index a superset of what the log already had, so a
/// tagged read for `stream_tag(id)` is the same question as `read(id)`.
///
/// A missing attribute is skipped rather than an error: history is not rewritten, so an event appended
/// before an attribute existed has to keep loading. That is also why this reads the payload by name rather
/// than through a typed shape — it runs over every version of an event that was ever written.
pub fn tags_of(event: &DomainEvent) -> Vec<String> {
    let mut tags = vec![stream_tag(&event.stream_id)];
    let identities = IDENTIFIES.iter().find(|(name, _)| *name == event.event_type).map_or(&[][..], |(_, ids)| ids);
    for (attribute, kind) in identities {
        if let Some(value) = event.payload.get(*attribute).and_then(|v| v.as_str()).filter(|v| !v.is_empty()) {
            tags.push(format!("{kind}:{value}"));
        }
    }
    tags
}
```

Wire it in where the store is built — `PostgresEventStore::tagged(pool, tags_of)` — and run the reindex once
against a log that predates it, which is the same rebuild any read model gets. The tags themselves are never
in the model: they are an index over the log, and the model records only which attributes identify what.
