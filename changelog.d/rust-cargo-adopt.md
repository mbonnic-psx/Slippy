MINOR

**`slipwai adopt` now recognises a Cargo repository.** A directory holding a `Cargo.toml` is proposed as Rust
built by Cargo: install (`cargo fetch --locked`), typecheck (`cargo check --all-targets`), lint (`cargo clippy
--all-targets --message-format=short -- -D warnings && cargo fmt --check`, one line per finding so the ratchet
holds each warning as its own) and test (`cargo test`) are offered, and audit, mutation,
integration and adversarial are written as no answer. This covers one crate; the adoption path is experimental
(see `AGENTS.md`), so what it offers may still change in a MINOR.

What stays out, and comes later: a toolchain pin read from `rust-toolchain.toml`, Rust set up in the adopted CI,
and Cargo workspaces.

The ratchet also reads cargo's `error: no such command` — a machine without the clippy or rustfmt component —
as the tool not being there (exit 101 otherwise looks like a clippy failure), so it refuses and records nothing
instead of baselining the missing component. This holds for every cargo subcommand a repository records.

**Catch-up.** None is needed. An already-adopted repository has no Cargo candidate recorded, and `slipwai adopt
--refresh` does not propose one: it reports the directory as `not wrapped: ... builds (cargo, rust ...) and has no
record`. To adopt it, add a `"generated": false` record for it under `deployables` in `project.json` and run
`slipwai adopt --refresh` again so the Makefile follows the record. A repository whose adopted `delivery/scripts/`
predates this release gets the ratchet change from `slipwai migrate`.
