```rust
// gifting/src/testing/fakes/pledge_persistence.rs

use std::collections::HashMap;
use std::sync::Mutex;

/// An in-memory `PledgePersistence` for use-case tests. It implements the real trait and exposes what it saw
/// for assertions. It lives outside the production hexagon, under `testing/fakes/`, never among the production
/// adapters. The mutex makes it safe under genuinely concurrent tasks, which is what lets the
/// optimistic-concurrency test exercise a real race.
#[derive(Default)]
pub struct FakePledgePersistence {
    state: Mutex<FakeState>,
}

#[derive(Default)]
struct FakeState {
    occasions: HashMap<OccasionId, StoredOccasion>,
    saved: Vec<Occasion>,
    outbox: Vec<PledgeRecorded>,
}

impl FakePledgePersistence {
    pub fn with(occasions: impl IntoIterator<Item = Occasion>) -> Self {
        let occasions = occasions.into_iter().map(|o| (o.id.clone(), StoredOccasion { value: o, version: 0 })).collect();
        Self { state: Mutex::new(FakeState { occasions, ..FakeState::default() }) }
    }

    pub fn saved(&self) -> Vec<Occasion> {
        self.state.lock().unwrap().saved.clone()
    }

    pub fn outbox(&self) -> Vec<PledgeRecorded> {
        self.state.lock().unwrap().outbox.clone()
    }
}

impl PledgePersistence for FakePledgePersistence {
    async fn find_occasion_by_id(&self, id: &OccasionId) -> Result<Option<StoredOccasion>, PersistenceError> {
        Ok(self.state.lock().unwrap().occasions.get(id).cloned())
    }

    async fn save_with_outbox(
        &self,
        occasion: &Occasion,
        events: &[PledgeRecorded],
        expected_version: i64,
    ) -> Result<SaveOutcome, PersistenceError> {
        let mut state = self.state.lock().unwrap();
        match state.occasions.get(&occasion.id) {
            Some(current) if current.version == expected_version => {}
            _ => return Ok(SaveOutcome::Conflict),
        }
        state.occasions.insert(occasion.id.clone(), StoredOccasion { value: occasion.clone(), version: expected_version + 1 });
        state.saved.push(occasion.clone());
        state.outbox.extend_from_slice(events);
        Ok(SaveOutcome::Saved)
    }
}

// gifting/src/testing/fakes/pledge_projection.rs

/// An in-memory `PledgeProjection`, idempotent by event id, that exposes `records()` for assertions.
#[derive(Default)]
pub struct FakePledgeProjection {
    records: Mutex<Vec<PledgeRecorded>>,
}

impl FakePledgeProjection {
    pub fn records(&self) -> Vec<PledgeRecorded> {
        self.records.lock().unwrap().clone()
    }
}

impl PledgeProjection for FakePledgeProjection {
    async fn record_from(&self, event: &PledgeRecorded) -> Result<(), PersistenceError> {
        let mut records = self.records.lock().unwrap();
        if !records.iter().any(|recorded| recorded.id == event.id) {
            records.push(event.clone());
        }
        Ok(())
    }
}
```
