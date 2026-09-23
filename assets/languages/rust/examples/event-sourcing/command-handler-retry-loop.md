```rust
/// The storage port a Decider-based handler reads from and appends to, generic over that Decider's events.
pub trait EventStore<Event> {
    async fn read(&self, stream_id: &StreamId) -> Result<(Vec<Event>, u64), StoreError>;
    async fn append(&self, stream_id: &StreamId, events: &[Event], expected_version: u64)
        -> Result<AppendResult, StoreError>;
}

#[derive(Debug, PartialEq)]
pub enum CommandResult<Event, Reason> {
    Succeeded { events: Vec<Event> },
    Rejected(Reason),
    ConcurrentModification,
}

/// Handles a command with the given Decider and store. On a version conflict — the stream moved under us —
/// it reloads and re-decides against the fresh state, up to `max_attempts` times, rather than failing on the
/// first race.
pub async fn handle_with_retry<S: Clone, C, E: Clone, R, Err>(
    decider: &Decider<S, C, E, R, Err>,
    store: &impl EventStore<E>,
    stream_id: &StreamId,
    command: &C,
    max_attempts: usize,
) -> Result<CommandResult<E, R>, HandleError>
where
    HandleError: From<Err>,
{
    for _ in 0..max_attempts.max(1) {
        let (events, version) = store.read(stream_id).await?; // load
        let state = rehydrate(decider, &events)?; // rehydrate (pure)
        let new_events = match (decider.decide)(command, &state) {
            Ok(events) => events, // decide (pure)
            Err(reason) => return Ok(CommandResult::Rejected(reason)),
        };
        match store.append(stream_id, &new_events, version).await? {
            AppendResult::Appended { .. } => return Ok(CommandResult::Succeeded { events: new_events }),
            // The stream moved under us: loop to reload and re-decide against fresh state.
            AppendResult::VersionConflict { .. } => continue,
        }
    }
    Ok(CommandResult::ConcurrentModification)
}
```
