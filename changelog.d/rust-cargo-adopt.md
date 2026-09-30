MINOR

**`slipwai adopt` now recognises a Cargo repository.** A directory holding a `Cargo.toml` is proposed as Rust
built by Cargo: install (`cargo fetch --locked`), typecheck (`cargo check --all-targets`), lint (`cargo clippy
--all-targets --message-format=short -- -D warnings && cargo fmt --check`, one line per finding so the ratchet
holds each warning as its own) and test (`cargo test`) are offered, and audit, mutation,
integration and adversarial are written as no answer. This covers one crate; the adoption path is experimental
(see `AGENTS.md`), so what it offers may still change in a MINOR.

What stays out, and comes later: a toolchain pin read from `rust-toolchain.toml`, Rust set up in the adopted CI,
and Cargo workspaces.

The ratchet also reads a missing clippy or rustfmt component as the tool not being there, so it refuses and
records nothing instead of baselining it. That covers cargo's `error: no such command` (exit 101, which otherwise
looks like a clippy failure) and rustup's `error: 'cargo-clippy' is not installed for …` (exit 1). It holds with
colour on, past a leading `cd <dir> &&`, `NAME=value` or `env`, and for every cargo subcommand a repository records.
A quarantined `cargo test` is keyed by each failing test's name on libtest's `test … ... FAILED` line, not by the
panic line, which names the OS thread and so differs every run. A test binary killed by a signal is a finding of
its own, `crash: SIGABRT`, so a crash fails a quarantined suite instead of passing on the exit code alone.

**Catch-up.** None is needed. An already-adopted repository has no Cargo candidate recorded, and `slipwai adopt
--refresh` does not propose one: it reports the directory as `not wrapped: ... builds (cargo, rust ...) and has no
record`. To adopt it, add a `"generated": false` record for it under `deployables` in `project.json` and run
`slipwai adopt --refresh` again so the Makefile follows the record. A repository whose adopted `delivery/scripts/`
predates this release gets the ratchet change from `slipwai migrate`.
