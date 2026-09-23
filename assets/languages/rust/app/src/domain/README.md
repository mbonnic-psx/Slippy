# The domain layer

The model and nothing else: the types, the domain event shapes, and the decision functions that turn a
command and a fold of past events into new events or a refusal. This is the part of the service that
survives replacing the transport, the store and the framework, so it is the part that names none of them.

This directory ships with no module in it because the model is yours to write; the layer exists from day
one so that the first thing written into it lands in the right place. Add `src/domain/mod.rs` and
`pub mod domain;` to `src/lib.rs`, and the module is a module.

**What belongs here.** Plain structs, enums and functions, and their `#[cfg(test)]` modules. No I/O, no
`SystemTime::now`, no randomness: a decision that needs the time or an id takes it as an argument, which is
what lets its test be a table of inputs and expected events.

**What it may not use**, enforced by `make check-imports`:

- anything under `crate::adapters`, `crate::infrastructure` or `crate::delivery` — those use the domain,
  never the reverse.

Keeping drivers and frameworks out (`sqlx`, `axum`, `tokio`) is the house rule the review holds you to; the
import gate checks the layer direction, not the crate list, for Rust.

The port traits a decision is driven through live under `../application/ports/`.
