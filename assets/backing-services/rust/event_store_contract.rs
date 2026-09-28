//! One contract, run against every event-store adapter.
//!
//! The infrastructure-free adapters run it in `make test`; Postgres runs it in `make test-integration`. Two
//! adapters that pass different tests are two different ports wearing one name, and the day they diverge
//! is the day a slice that worked on the fake stops working in production.
//!
//! Nothing here truncates or deletes: the log is append-only, and a real store enforces that with a
//! trigger, so a shared table cannot be cleaned between tests. Every case therefore works in freshly named
//! streams and asserts only about those. That is not a workaround — it is what testing against a real
//! append-only log looks like.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::application::ports::events::{
    Actor, AppendResult, CausationId, CommittedEvent, Condition, ConditionalAppendResult,
    CorrelationId, DomainEvent, EventStore, NO_STREAM, StoreError, TagFilter, TagQuery, TaggedRead,
    TagsOf, current_version, default_tags_of, stream_tag,
};

/// A tagging function of the shape a real project writes: the stream, plus the payload's identifying
/// attributes. Tags are derived from the event and nothing else, which is what makes the index rebuildable
/// — so this runs on the way in *and* during a reindex, and the contract proves the two agree.
pub fn payload_tags() -> TagsOf {
    Arc::new(|event: &DomainEvent| {
        let mut tags = vec![stream_tag(&event.stream_id)];
        for (key, kind) in [("courseId", "course"), ("studentId", "student")] {
            if let Some(Value::String(value)) = event.payload.get(key) {
                tags.push(format!("{kind}:{value}"));
            }
        }
        tags
    })
}

/// What a project that has not adopted tags has: a log with no index over it.
pub fn no_tags() -> TagsOf {
    Arc::new(|_: &DomainEvent| Vec::new())
}

struct Case {
    run: String,
    correlation_id: CorrelationId,
    counter: AtomicUsize,
}

impl Case {
    fn new() -> Self {
        Self {
            run: Uuid::new_v4().simple().to_string()[..16].to_owned(),
            correlation_id: CorrelationId::generate(),
            counter: AtomicUsize::new(0),
        }
    }

    fn stream(&self) -> String {
        format!(
            "contract-{}-{}",
            self.run,
            self.counter.fetch_add(1, Ordering::Relaxed) + 1
        )
    }

    /// The value a case's tags are built from. Unique per case, not per run, because Postgres runs this
    /// suite against a database it shares with every other case and run — and a tag is not scoped to a
    /// stream, so two cases tagging by the same id would find each other's events.
    fn subject(&self) -> String {
        format!(
            "{}-{}",
            self.run,
            self.counter.fetch_add(1, Ordering::Relaxed) + 1
        )
    }

    fn event(&self, stream_id: &str, event_type: &str, payload: Value) -> DomainEvent {
        let payload: Map<String, Value> = match payload {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        DomainEvent {
            event_type: event_type.to_owned(),
            schema_version: 1,
            stream_id: stream_id.to_owned(),
            payload,
            occurred_at: "2024-01-01T00:00:00.000000+00:00".to_owned(),
            actor: Actor {
                kind: "test".to_owned(),
                id: self.run.clone(),
            },
            correlation_id: self.correlation_id,
            causation_id: None,
        }
    }

    fn plain(&self, stream_id: &str, event_type: &str) -> DomainEvent {
        self.event(stream_id, event_type, Value::Null)
    }
}

fn types_of(events: &[CommittedEvent]) -> Vec<&str> {
    events
        .iter()
        .map(|event| event.event.event_type.as_str())
        .collect()
}

fn tagged(tags: &[&str]) -> TagQuery {
    TagQuery {
        filters: vec![TagFilter {
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            types: vec![],
        }],
    }
}

async fn must_append(
    store: &impl EventStore,
    stream: &str,
    expected: i64,
    events: Vec<DomainEvent>,
) -> AppendResult {
    let result = store
        .append(stream, expected, events)
        .await
        .expect("append");
    assert!(
        matches!(result, AppendResult::Appended { .. }),
        "append to {stream} was refused: {result:?}"
    );
    result
}

async fn read_tagged(store: &impl EventStore, query: &TagQuery, until: i64) -> TaggedRead {
    store
        .read_tagged(query, 0, until)
        .await
        .expect("read by tag query")
}

async fn replay(
    store: &impl EventStore,
    from: i64,
    keep: impl Fn(&CommittedEvent) -> bool + Send + Sync,
) -> Vec<CommittedEvent> {
    let mut replayed = Vec::new();
    store
        .read_all(from, &mut |event| {
            if keep(&event) {
                replayed.push(event);
            }
            Ok(())
        })
        .await
        .expect("replay");
    replayed
}

#[derive(Debug, thiserror::Error)]
#[error("the view write failed")]
struct ViewFailed;

/// Holds an adapter to the port's contract. `new_store` builds the adapter under test, indexing each event
/// by what the given tagging function returns; it is called once per case, so each starts clean where the
/// adapter can be clean and works in fresh streams where it cannot.
pub async fn run<S: EventStore>(new_store: impl AsyncFn(TagsOf) -> S) {
    let case = Case::new();
    let plain = async || new_store(default_tags_of()).await;
    let with_tags = async || new_store(payload_tags()).await;

    // reads an unknown stream as empty rather than failing
    {
        let store = plain().await;
        assert!(store.read(&case.stream()).await.expect("read").is_empty());
    }

    // appends to a stream that does not exist yet
    {
        let store = plain().await;
        let stream = case.stream();
        let result = store
            .append(
                &stream,
                NO_STREAM,
                vec![
                    case.plain(&stream, "Started"),
                    case.plain(&stream, "Continued"),
                ],
            )
            .await
            .expect("append");
        assert_eq!(result, AppendResult::Appended { version: 1 });
    }

    // returns the stream in version order with what only the store knows
    {
        let store = plain().await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.event(&stream, "Started", json!({"step": 1}))],
        )
        .await;
        must_append(
            &store,
            &stream,
            0,
            vec![case.event(&stream, "Continued", json!({"step": 2}))],
        )
        .await;
        let recorded = store.read(&stream).await.expect("read");
        assert_eq!(types_of(&recorded), ["Started", "Continued"]);
        assert_eq!((recorded[0].version, recorded[1].version), (0, 1));
        assert!(
            recorded[1].global_position > recorded[0].global_position,
            "global position did not advance"
        );
        assert_eq!(
            recorded[0].event.payload.get("step"),
            Some(&json!(1)),
            "payload did not survive"
        );
        assert_eq!(
            recorded[0].event.actor,
            Actor {
                kind: "test".to_owned(),
                id: case.run.clone()
            }
        );
        assert!(
            !recorded[0].recorded_at.is_empty(),
            "the store recorded no storage time"
        );
    }

    // round-trips the correlation and causation ids as UUIDs
    {
        let store = plain().await;
        let stream = case.stream();
        let cause = CausationId::from(Uuid::new_v4());
        let mut caused = case.plain(&stream, "Caused");
        caused.causation_id = Some(cause);
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "Started"), caused],
        )
        .await;
        let recorded = store.read(&stream).await.expect("read");
        assert_eq!(recorded[0].event.correlation_id, case.correlation_id);
        // The first event of a transaction has no cause, and the store does not invent one.
        assert_eq!(recorded[0].event.causation_id, None);
        assert_eq!(recorded[1].event.causation_id, Some(cause));
    }

    // reports a stale expected version as a value, not an error
    {
        let store = plain().await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "Started")],
        )
        .await;
        let result = store
            .append(&stream, NO_STREAM, vec![case.plain(&stream, "Raced")])
            .await;
        assert_eq!(
            result.expect("a version conflict must not be an error"),
            AppendResult::VersionConflict { actual_version: 0 }
        );
    }

    // writes nothing when the expected version is stale
    {
        let store = plain().await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "Started")],
        )
        .await;
        store
            .append(
                &stream,
                NO_STREAM,
                vec![
                    case.plain(&stream, "Rejected"),
                    case.plain(&stream, "AlsoRejected"),
                ],
            )
            .await
            .expect("append");
        assert_eq!(
            types_of(&store.read(&stream).await.expect("read")),
            ["Started"]
        );
    }

    // continues a stream from the version it reports
    {
        let store = plain().await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "Started")],
        )
        .await;
        let history = store.read(&stream).await.expect("read");
        assert_eq!(current_version(&history), 0);
        let result = store
            .append(
                &stream,
                current_version(&history),
                vec![case.plain(&stream, "Continued")],
            )
            .await;
        assert_eq!(
            result.expect("append"),
            AppendResult::Appended { version: 1 }
        );
    }

    // treats an empty append as a no-op at the expected version
    {
        let store = plain().await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "Started")],
        )
        .await;
        assert_eq!(
            store.append(&stream, 0, vec![]).await.expect("append"),
            AppendResult::Appended { version: 0 }
        );
        assert_eq!(
            store.read(&stream).await.expect("read").len(),
            1,
            "an empty append wrote something"
        );
    }

    // keeps streams apart
    {
        let store = plain().await;
        let (one, other) = (case.stream(), case.stream());
        must_append(&store, &one, NO_STREAM, vec![case.plain(&one, "Mine")]).await;
        must_append(
            &store,
            &other,
            NO_STREAM,
            vec![case.plain(&other, "Theirs")],
        )
        .await;
        assert_eq!(types_of(&store.read(&one).await.expect("read")), ["Mine"]);
        assert_eq!(
            types_of(&store.read(&other).await.expect("read")),
            ["Theirs"]
        );
    }

    // replays across all streams in global-position order
    {
        let store = plain().await;
        let (one, other) = (case.stream(), case.stream());
        must_append(&store, &one, NO_STREAM, vec![case.plain(&one, "First")]).await;
        must_append(
            &store,
            &other,
            NO_STREAM,
            vec![case.plain(&other, "Second")],
        )
        .await;
        must_append(&store, &one, 0, vec![case.plain(&one, "Third")]).await;
        let replayed = replay(&store, 0, |e| {
            e.event.stream_id == one || e.event.stream_id == other
        })
        .await;
        assert_eq!(types_of(&replayed), ["First", "Second", "Third"]);
        assert!(
            replayed
                .windows(2)
                .all(|pair| pair[1].global_position > pair[0].global_position)
        );
    }

    // replays only from the requested position onward
    {
        let store = plain().await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "First"), case.plain(&stream, "Second")],
        )
        .await;
        let recorded = store.read(&stream).await.expect("read");
        let replayed = replay(&store, recorded[1].global_position, |e| {
            e.event.stream_id == stream
        })
        .await;
        assert_eq!(types_of(&replayed), ["Second"]);
    }

    // ── The tag index, and the boundary drawn out of it ───────────────────────────────────────────────
    //
    // A fake that answers a tag query differently from Postgres proves nothing about production. The SQL
    // adapters translate `TagFilter::matches` into SQL, and these cases hold the translation to it.

    // indexes every event by its own stream without being asked
    {
        let store = plain().await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "Started")],
        )
        .await;
        let found = read_tagged(&store, &tagged(&[&stream_tag(&stream)]), 0).await;
        assert_eq!(types_of(&found.events), ["Started"]);
    }

    // reports the store head with what it read
    {
        let store = with_tags().await;
        let stream = case.stream();
        let query = tagged(&[&stream_tag(&stream)]);
        let empty = read_tagged(&store, &query, 0).await;
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "Started")],
        )
        .await;
        let after = read_tagged(&store, &query, 0).await;
        assert!(
            after.head > empty.head,
            "the head did not move past {}",
            empty.head
        );
        assert_eq!(
            after.events.last().expect("an event").global_position,
            after.head
        );
    }

    // names the last position as the head, which a decision's boundary is drawn at
    {
        let store = plain().await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![
                case.plain(&stream, "Started"),
                case.plain(&stream, "Continued"),
            ],
        )
        .await;
        let last = store.read(&stream).await.expect("read")[1].global_position;
        assert!(
            store.head().await.expect("head") >= last,
            "the head is behind an appended event"
        );
    }

    // matches an event only when it carries every tag in a filter
    {
        let store = with_tags().await;
        let (stream, subject) = (case.stream(), case.subject());
        let payload = json!({"courseId": subject, "studentId": subject});
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.event(&stream, "Subscribed", payload)],
        )
        .await;
        let course = format!("course:{subject}");
        let both = read_tagged(
            &store,
            &tagged(&[&course, &format!("student:{subject}")]),
            0,
        )
        .await;
        let one_wrong = read_tagged(
            &store,
            &tagged(&[&course, &format!("student:{subject}-nobody")]),
            0,
        )
        .await;
        assert_eq!(both.events.len(), 1);
        assert!(
            one_wrong.events.is_empty(),
            "a filter matched with one tag wrong"
        );
    }

    // matches any filter in the query
    {
        let store = with_tags().await;
        let (course_stream, student_stream, subject) =
            (case.stream(), case.stream(), case.subject());
        must_append(
            &store,
            &course_stream,
            NO_STREAM,
            vec![case.event(
                &course_stream,
                "CourseCapacitySet",
                json!({"courseId": subject}),
            )],
        )
        .await;
        must_append(
            &store,
            &student_stream,
            NO_STREAM,
            vec![case.event(
                &student_stream,
                "StudentSubscribed",
                json!({"studentId": subject}),
            )],
        )
        .await;
        let query = TagQuery {
            filters: vec![
                TagFilter {
                    tags: vec![format!("course:{subject}")],
                    types: vec![],
                },
                TagFilter {
                    tags: vec![format!("student:{subject}")],
                    types: vec![],
                },
            ],
        };
        assert_eq!(
            types_of(&read_tagged(&store, &query, 0).await.events),
            ["CourseCapacitySet", "StudentSubscribed"]
        );
    }

    // narrows a filter by event type
    {
        let store = with_tags().await;
        let (stream, subject) = (case.stream(), case.subject());
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![
                case.event(&stream, "Subscribed", json!({"courseId": subject})),
                case.event(&stream, "Unsubscribed", json!({"courseId": subject})),
            ],
        )
        .await;
        let query = TagQuery {
            filters: vec![TagFilter {
                tags: vec![format!("course:{subject}")],
                types: vec!["Unsubscribed".to_owned()],
            }],
        };
        assert_eq!(
            types_of(&read_tagged(&store, &query, 0).await.events),
            ["Unsubscribed"]
        );
    }

    // matches nothing for an empty query rather than everything
    {
        let store = with_tags().await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "Started")],
        )
        .await;
        for (name, query) in [
            ("no filters", TagQuery::default()),
            (
                "empty filter",
                TagQuery {
                    filters: vec![TagFilter::default()],
                },
            ),
        ] {
            assert!(
                read_tagged(&store, &query, 0).await.events.is_empty(),
                "{name} matched something"
            );
        }
    }

    // appends conditionally when nothing matching arrived since the read
    {
        let store = with_tags().await;
        let (stream, subject) = (case.stream(), case.subject());
        let query = tagged(&[&format!("course:{subject}")]);
        let decided_at = read_tagged(&store, &query, 0).await.head;
        let result = store
            .append_if(
                &Condition {
                    query,
                    after: decided_at,
                },
                vec![case.event(&stream, "Subscribed", json!({"courseId": subject}))],
            )
            .await
            .expect("conditional append");
        assert!(
            matches!(result, ConditionalAppendResult::Recorded { .. }),
            "{result:?}"
        );
        assert_eq!(
            types_of(&store.read(&stream).await.expect("read")),
            ["Subscribed"]
        );
    }

    // refuses a conditional append when a matching event arrived since
    {
        let store = with_tags().await;
        let (stream, other, subject) = (case.stream(), case.stream(), case.subject());
        let query = tagged(&[&format!("course:{subject}")]);
        let decided_at = read_tagged(&store, &query, 0).await.head;
        must_append(
            &store,
            &other,
            NO_STREAM,
            vec![case.event(&other, "Subscribed", json!({"courseId": subject}))],
        )
        .await;
        let result = store
            .append_if(
                &Condition {
                    query,
                    after: decided_at,
                },
                vec![case.event(&stream, "Subscribed", json!({"courseId": subject}))],
            )
            .await
            .expect("conditional append");
        match result {
            ConditionalAppendResult::ConditionConflict { head } => assert!(head >= decided_at),
            other => panic!("expected a condition conflict, got {other:?}"),
        }
        assert!(
            store.read(&stream).await.expect("read").is_empty(),
            "a refused conditional append wrote"
        );
    }

    // leaves a conditional append readable as part of its stream
    {
        let store = with_tags().await;
        let (stream, subject) = (case.stream(), case.subject());
        let query = tagged(&[&format!("course:{subject}")]);
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.event(&stream, "Opened", json!({"courseId": subject}))],
        )
        .await;
        let head = read_tagged(&store, &query, 0).await.head;
        store
            .append_if(
                &Condition { query, after: head },
                vec![case.event(&stream, "Subscribed", json!({"courseId": subject}))],
            )
            .await
            .expect("conditional append");
        let stored = store.read(&stream).await.expect("read");
        assert_eq!(
            stored.iter().map(|e| e.version).collect::<Vec<_>>(),
            [0, 1],
            "versions are not gapless"
        );
        let continued = store
            .append(&stream, current_version(&stored), vec![])
            .await
            .expect("append");
        assert_eq!(continued, AppendResult::Appended { version: 1 });
    }

    // behaves exactly as it did before when it indexes nothing
    {
        let store = new_store(no_tags()).await;
        let stream = case.stream();
        assert_eq!(
            must_append(
                &store,
                &stream,
                NO_STREAM,
                vec![case.plain(&stream, "Started")]
            )
            .await,
            AppendResult::Appended { version: 0 }
        );
        assert_eq!(store.read(&stream).await.expect("read").len(), 1);
        assert!(
            read_tagged(&store, &tagged(&[&stream_tag(&stream)]), 0)
                .await
                .events
                .is_empty(),
            "indexed with no tagging"
        );
    }

    // indexes a log it did not index when the events were written
    {
        let store = new_store(no_tags()).await;
        let (stream, subject) = (case.stream(), case.subject());
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![
                case.event(&stream, "Enrolled", json!({"courseId": subject})),
                case.event(&stream, "Graduated", json!({"courseId": subject})),
            ],
        )
        .await;
        let query = tagged(&[&format!("course:{subject}")]);
        assert!(
            read_tagged(&store, &query, 0).await.events.is_empty(),
            "indexed before a reindex"
        );
        let indexed = store.retag(payload_tags()).await.expect("retag");
        assert!(
            indexed >= 2,
            "expected at least the two events indexed, got {indexed}"
        );
        assert_eq!(read_tagged(&store, &query, 0).await.events.len(), 2);
        // Idempotent: a second run indexes nothing, so one that died halfway is finished by running again.
        assert_eq!(store.reindex_tags(0).await.expect("reindex"), 0);
    }

    // counts only the events a reindex made findable by tag
    {
        let store = new_store(no_tags()).await;
        let stream = case.stream();
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![
                case.event(&stream, "Enrolled", json!({"courseId": case.subject()})),
                case.event(&stream, "Graduated", json!({"courseId": case.subject()})),
            ],
        )
        .await;
        assert_eq!(
            store.reindex_tags(0).await.expect("reindex"),
            0,
            "counted events a reindex tagged nothing for"
        );
    }

    // reads as of a boundary and nothing after it
    {
        let store = with_tags().await;
        let (stream, subject) = (case.stream(), case.subject());
        let query = tagged(&[&format!("course:{subject}")]);
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.event(&stream, "Enrolled", json!({"courseId": subject}))],
        )
        .await;
        let boundary = store.head().await.expect("head");
        // Somebody else's event, after the boundary this decision was drawn at.
        must_append(
            &store,
            &stream,
            0,
            vec![case.event(&stream, "Graduated", json!({"courseId": subject}))],
        )
        .await;
        let as_of = read_tagged(&store, &query, boundary).await;
        assert_eq!(types_of(&as_of.events), ["Enrolled"]);
        assert_eq!(as_of.head, boundary);
        // Unbounded, the same query sees both — the ceiling is the caller's decision.
        let current = read_tagged(&store, &query, 0).await;
        assert_eq!(current.events.len(), 2);
        assert!(current.head >= boundary);
    }

    // makes an append and its tags arrive together or not at all
    {
        let store = with_tags().await;
        let (stream, subject) = (case.stream(), case.subject());
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.event(&stream, "Subscribed", json!({"courseId": subject}))],
        )
        .await;
        let found = read_tagged(&store, &tagged(&[&format!("course:{subject}")]), 0).await;
        assert_eq!(
            store.read(&stream).await.expect("read").len(),
            found.events.len(),
            "the log and its index disagree"
        );
    }

    // leaves the log as it was when a unit of work fails
    {
        let store = plain().await;
        let stream = case.stream();
        let outcome: Result<(), StoreError> = store
            .in_unit_of_work(|| async {
                store
                    .append(&stream, NO_STREAM, vec![case.plain(&stream, "Started")])
                    .await?;
                Err(StoreError::new("the view", ViewFailed))
            })
            .await;
        let failure = outcome.expect_err("the view's failure reaches the caller");
        assert!(
            failure.to_string().contains("the view write failed"),
            "{failure}"
        );
        assert!(
            store.read(&stream).await.expect("read").is_empty(),
            "a failed unit of work committed"
        );
    }

    // commits everything in a unit of work that completes, once
    {
        let store = plain().await;
        let (one, other) = (case.stream(), case.stream());
        store
            .in_unit_of_work(|| async {
                store
                    .append(&one, NO_STREAM, vec![case.plain(&one, "Mine")])
                    .await?;
                store
                    .append(&other, NO_STREAM, vec![case.plain(&other, "Theirs")])
                    .await?;
                Ok(())
            })
            .await
            .expect("unit of work");
        assert_eq!(types_of(&store.read(&one).await.expect("read")), ["Mine"]);
        assert_eq!(
            types_of(&store.read(&other).await.expect("read")),
            ["Theirs"]
        );
    }

    // writes nothing and ends nothing when a nested append is refused: a conflict is a value, so the
    // caller's transaction survives it and goes on to commit what else it was doing
    {
        let store = plain().await;
        let (stream, other) = (case.stream(), case.stream());
        must_append(
            &store,
            &stream,
            NO_STREAM,
            vec![case.plain(&stream, "Started")],
        )
        .await;
        let refused = store
            .in_unit_of_work(|| async {
                let refused = store
                    .append(
                        &stream,
                        NO_STREAM,
                        vec![
                            case.plain(&stream, "Rejected"),
                            case.plain(&stream, "AlsoRejected"),
                        ],
                    )
                    .await?;
                store
                    .append(&other, NO_STREAM, vec![case.plain(&other, "Unaffected")])
                    .await?;
                Ok(refused)
            })
            .await
            .expect("unit of work");
        assert_eq!(refused, AppendResult::VersionConflict { actual_version: 0 });
        assert_eq!(
            types_of(&store.read(&stream).await.expect("read")),
            ["Started"]
        );
        assert_eq!(
            types_of(&store.read(&other).await.expect("read")),
            ["Unaffected"]
        );
    }
}
