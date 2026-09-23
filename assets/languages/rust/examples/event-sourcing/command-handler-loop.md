```rust
/// What handling a command against a stream came to.
#[derive(Debug, PartialEq)]
pub enum CommandResult {
    Succeeded { events: Vec<AccountEvent> },
    Rejected(Rejection),
    ConcurrentModification,
}

/// The bare use-case shape: load the stream, rehydrate by folding, decide, then append with optimistic
/// concurrency. It does not retry on a conflict — the retry-loop example does.
pub async fn handle_command(
    store: &impl EventStore,
    stream_id: &StreamId,
    command: &AccountCommand,
) -> Result<CommandResult, HandleError> {
    // 1. LOAD the stream's events and the version they were read at.
    let (events, version) = store.read(stream_id).await?;

    // 2. REHYDRATE current state by folding — pure.
    let state = events.iter().try_fold(INITIAL_STATE, evolve)?;

    // 3. DECIDE — pure business logic.
    let new_events = match decide(command, state) {
        Ok(events) => events,
        Err(rejection) => return Ok(CommandResult::Rejected(rejection)),
    };

    // 4. APPEND, asserting the stream has not moved since it was read. A conflict is a value to inspect.
    match store.append(stream_id, &new_events, version).await? {
        AppendResult::Appended { .. } => Ok(CommandResult::Succeeded { events: new_events }),
        AppendResult::VersionConflict { .. } => Ok(CommandResult::ConcurrentModification),
    }
}
```
