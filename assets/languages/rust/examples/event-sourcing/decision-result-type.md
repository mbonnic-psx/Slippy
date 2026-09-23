```rust
/// What a decide function returns: the events to append, or why not. Rust's `Result` is exactly that shape,
/// so the alias only names it — a rejection is a value the caller matches on, never a panic.
pub type Outcome<Event, Reason> = Result<Vec<Event>, Reason>;

/// An accepted outcome carrying the given events.
pub fn accept<Event, Reason>(events: Vec<Event>) -> Outcome<Event, Reason> {
    Ok(events)
}

/// A rejected outcome carrying the given reason.
pub fn reject<Event, Reason>(reason: Reason) -> Outcome<Event, Reason> {
    Err(reason)
}
```
