```rust
/// Everything needed to decide and evolve one aggregate: its seed state, its pure decision function, its pure
/// fold, and when the stream is done. Function pointers rather than a trait so a Decider is a value that can
/// be built, passed and tested like any other.
pub struct Decider<State, Command, Event, Reason, Corrupt> {
    pub initial_state: State,
    pub decide: fn(&Command, &State) -> Result<Vec<Event>, Reason>,
    /// An `Err` rather than a bare state when a known event cannot legally follow the current one: corrupt
    /// history is an operational occurrence the caller handles, not a value to paper over.
    pub evolve: fn(State, &Event) -> Result<State, Corrupt>,
    /// Whether the state accepts no further commands. `None` means the stream never terminates.
    pub is_terminal: Option<fn(&State) -> bool>,
}
```
