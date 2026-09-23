//! The event store's port, with nothing behind it yet.
//!
//! This service's event-store axis offers no adapter, so the port arrives with its shape and one test of
//! the rule every adapter will be held to: a version conflict is a value the caller inspects, never a panic
//! or an `Err` it can forget to handle differently from a failure.

use std::collections::BTreeMap;

/// One fact that happened, in the shape every adapter stores.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainEvent {
    pub event_type: String,
    pub schema_version: u32,
    pub stream_id: String,
    pub payload: BTreeMap<String, String>,
}

/// What an append came to. A conflict is an outcome, not an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendResult {
    /// The events were written; the stream now stands at `version`.
    Appended { version: u64 },
    /// Somebody else wrote first; the stream stands at `actual`, not the version the caller expected.
    VersionConflict { actual: u64 },
}

/// The port every write goes through.
pub trait EventStore {
    fn read(&self, stream_id: &str) -> Vec<DomainEvent>;
    fn append(
        &mut self,
        stream_id: &str,
        expected_version: u64,
        events: Vec<DomainEvent>,
    ) -> AppendResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_version_conflict_is_a_value() {
        let result = AppendResult::VersionConflict { actual: 2 };
        assert!(matches!(
            result,
            AppendResult::VersionConflict { actual: 2 }
        ));
    }
}
