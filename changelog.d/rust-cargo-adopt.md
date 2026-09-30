MINOR

**`slipwai adopt` now recognises a Cargo repository.** A directory holding a `Cargo.toml` is proposed as Rust
built by Cargo: install (`cargo fetch --locked`), typecheck (`cargo check --all-targets`), lint (`cargo clippy
--all-targets --message-format=short -- -D warnings && cargo fmt --check`, one line per finding so the ratchet
holds each warning as its own) and test (`cargo test`) are offered, and audit, mutation,
integration and adversarial are written as no answer. A Cargo workspace root is one candidate, not one per
member: its typecheck, lint and test carry `--workspace` (`cargo check --workspace --all-targets`, `cargo clippy
--workspace --all-targets --message-format=short -- -D warnings && cargo fmt --check`, `cargo test --workspace`),
with no `--all-features`, which needs system libraries some crates do not have on every machine; add it when
confirming. No member is proposed, a workspace below a workspace root (a cargo-fuzz `fuzz/`) is a candidate of its
own, and no "no lockfile" finding is reported for a member, since Cargo writes one `Cargo.lock` at the root. Where a
workspace root shares its directory with a `package.json`, a `pyproject.toml` or any other manifest the survey
tries first (a napi-rs or maturin crate), that directory stays one candidate, as the language of the manifest
tried first, and its Cargo workspace still owns the members below it, so no Cargo candidate is proposed for them
and the Rust is gated only when you override the root's language and commands when confirming (`--language
<name>=rust`, `--command <name>:<target>=<command>`). The adoption path is experimental (see `AGENTS.md`), so what
it offers may still change in a MINOR.

What stays out, and comes later: a toolchain pin read from `rust-toolchain.toml` and Rust set up in the adopted CI.
Nor are `members`, `exclude` and `default-members` read, or an inline `workspace = { … }` table: a workspace is
the `[workspace]` table header, and every `Cargo.toml` below one is its member, so a crate the root excludes that
declares no workspace of its own is owned and never proposed, however the root lists its members.

The ratchet also reads a missing clippy or rustfmt component as the tool not being there, so it refuses and
records nothing instead of baselining it. That covers cargo's `error: no such command` (exit 101, which otherwise
looks like a clippy failure) and rustup's `error: 'cargo-clippy' is not installed for …` (exit 1). It holds with
colour on, past a leading `cd <dir> &&`, `NAME=value` or `env`, and for every cargo subcommand a repository records.
A quarantined `cargo test` is keyed by each failing test's name on libtest's `test … ... FAILED` line, not by the
panic line, which names the OS thread and so differs every run. A test binary killed by a signal is a finding of
its own, `crash: SIGABRT`, so a crash fails a quarantined suite instead of passing on the exit code alone.

**Catch-up.** No release needs one. An already-adopted repository has no Cargo candidate recorded, and `slipwai
adopt --refresh` does not propose one: it reports the directory as `not wrapped: ... builds (cargo, rust ...) and
has no record`. To adopt it, add a `"generated": false` record for it under `deployables` in `project.json` and run
`slipwai adopt --refresh` again so the Makefile follows the record. A repository whose adopted `delivery/scripts/`
predates this release gets the ratchet change from `slipwai migrate`. A repository adopted with a snapshot before
this change has its root's commands refreshed to the workspace ones by `slipwai adopt --refresh`; a member it
recorded as a deployable of its own is reported as no longer recognised and left as written, so a snapshot
adoption asks you to remove that record from `project.json` if you want it gone.
