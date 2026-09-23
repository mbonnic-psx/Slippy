```rust
use std::collections::HashMap;
use std::sync::Mutex;

/// A real implementation of `EventStore` backed by a map — not a mock. It satisfies the same trait a
/// production adapter does, so tests built on it drive the real command-handling path with no special-cased
/// double anywhere. The mutex lets concurrent tests share one instance.
#[derive(Default)]
pub struct InMemoryEventStore {
    streams: Mutex<HashMap<StreamId, Vec<AccountEvent>>>,
}

impl EventStore<AccountEvent> for InMemoryEventStore {
    async fn read(&self, stream_id: &StreamId) -> Result<(Vec<AccountEvent>, u64), StoreError> {
        let streams = self.streams.lock().expect("store lock");
        let events = streams.get(stream_id).cloned().unwrap_or_default();
        let version = events.len() as u64;
        Ok((events, version))
    }

    async fn append(
        &self,
        stream_id: &StreamId,
        events: &[AccountEvent],
        expected_version: u64,
    ) -> Result<AppendResult, StoreError> {
        let mut streams = self.streams.lock().expect("store lock");
        let stream = streams.entry(stream_id.clone()).or_default();
        let actual = stream.len() as u64;
        if actual != expected_version {
            return Ok(AppendResult::VersionConflict { actual });
        }
        stream.extend_from_slice(events);
        Ok(AppendResult::Appended { version: stream.len() as u64 })
    }
}

/// The concrete `decide`/`evolve` pair as the generic Decider the retrying handler takes.
const ACCOUNT_DECIDER: Decider<AccountState, AccountCommand, AccountEvent, Rejection, CorruptHistory> = Decider {
    initial_state: INITIAL_STATE,
    decide: |command, state| decide(command, *state),
    evolve,
    is_terminal: None,
};

#[tokio::test]
async fn a_deposit_persists_so_a_later_withdrawal_sees_the_funds() {
    let store = InMemoryEventStore::default();
    let account = StreamId("acc-1".into());
    let handle = |command| async move { handle_with_retry(&ACCOUNT_DECIDER, &store, &account, &command, 3).await };

    handle(AccountCommand::OpenAccount { currency: Currency::Gbp }).await.unwrap();
    handle(AccountCommand::DepositMoney { amount: money(10_000) }).await.unwrap();
    let result = handle(AccountCommand::WithdrawMoney { amount: money(6_000) }).await.unwrap();

    assert_eq!(result, CommandResult::Succeeded { events: vec![AccountEvent::MoneyWithdrawn { amount: money(6_000) }] });
}

#[tokio::test]
async fn an_append_against_a_stale_version_is_a_conflict() {
    let store = InMemoryEventStore::default();
    let account = StreamId("acc-1".into());
    store.append(&account, &[opened()], 0).await.unwrap();

    let result = store.append(&account, &[deposited(10_000)], 0).await.unwrap(); // stale

    assert_eq!(result, AppendResult::VersionConflict { actual: 1 });
}
```
