MINOR

**Rust answers the event-store axis: `memory`, `sqlite` and `postgres`.** A Rust service under the event
profile now gets the whole write and read side the other backends have. That means the event-store port with
tags, tag queries and conditional append (DCB), and the checkpoint store with its leases. It also gets the
projection runner and ticker, and one contract suite each for the event store and the checkpoint store, run
against every adapter. The in-memory adapters are the default. SQLite and Postgres use sqlx 0.9. Postgres runs
conditional appends at SERIALIZABLE, keeps `ReadAll` behind the settled position with an advisory lock, and
compiles its migrations into `src/bin/migrate.rs`, which `make migrate` runs. A unit of work is carried in a
task-local and nests as savepoints. Each store's `sqlx` line sits in its own marked region of `Cargo.toml`, so
`./init --event-store memory` removes the crate along with the adapter that used it and re-locks the
workspace. The committed `Cargo.lock` is chosen per union of stores across the workspace's services, so
`--locked` builds offline. Coverage skips code that only integration tests reach (the Postgres adapters and
`src/bin`), and a committed `.cargo/mutants.toml` keeps cargo-mutants off that code and off the contract
suites. Rust still answers no transport and no identity provider; axum comes next.

**Catch-up.** Nothing for an existing project: this only adds what a new Rust service is generated with.
