```rust
/// One aggregate instance's event stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamId(pub String);

/// What an append came to. A version conflict — another writer got there first — is an outcome the caller
/// matches on (retry, surface a 409), not an error it could lump in with a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendResult {
    Appended { version: u64 },
    VersionConflict { actual: u64 },
}

/// The port the application layer owns and the command handler consumes, typed to one aggregate's event
/// family like a repository. One physical store holds many kinds of stream; the adapter parses stored JSON
/// into `AccountEvent` on read, so this view only ever exposes that family.
pub trait EventStore {
    /// Every event recorded for the stream, in order, with the stream's current version.
    async fn read(&self, stream_id: &StreamId) -> Result<(Vec<AccountEvent>, u64), StoreError>;

    /// Writes the events if, and only if, the stream is still at `expected_version`.
    async fn append(
        &self,
        stream_id: &StreamId,
        events: &[AccountEvent],
        expected_version: u64,
    ) -> Result<AppendResult, StoreError>;
}
```
