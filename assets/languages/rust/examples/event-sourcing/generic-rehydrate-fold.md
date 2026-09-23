```rust
/// Rebuilds any Decider's current state by folding `evolve` over past events from its initial state. It
/// stops at the first event that is not a legal transition and hands that error back: corrupt history is
/// reported, never folded past.
pub fn rehydrate<S: Clone, C, E, R, Corrupt>(
    decider: &Decider<S, C, E, R, Corrupt>,
    events: &[E],
) -> Result<S, Corrupt> {
    events.iter().try_fold(decider.initial_state.clone(), decider.evolve)
}
```
