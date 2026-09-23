```rust
/// Rebuilds current state by folding `evolve` over past events. `evolve` returns an `Err` when a known event
/// cannot follow the current state (corrupt history), and `try_fold` stops there and hands it back rather
/// than returning a partially folded state.
pub fn rehydrate(events: &[AccountEvent]) -> Result<AccountState, CorruptHistory> {
    events.iter().try_fold(INITIAL_STATE, evolve)
}
```
