MINOR

**`slipwai adopt` now recognises a Cargo repository.** A directory holding a `Cargo.toml` is proposed as Rust
built by Cargo: install (`cargo fetch --locked`), typecheck (`cargo check --all-targets`), lint (`cargo clippy
--all-targets --message-format=short -- -D warnings && cargo fmt --check`, one line per finding so the ratchet
holds each warning as its own) and test (`cargo test`) are offered, and audit, mutation,
integration and adversarial are written as no answer. This covers one crate; the adoption path is experimental
(see `AGENTS.md`), so what it offers may still change in a MINOR.

What stays out, and comes later: a toolchain pin read from `rust-toolchain.toml`, Rust set up in the adopted CI,
and Cargo workspaces.

**Catch-up.** None is needed. An already-adopted repository has no Cargo candidate recorded; `slipwai adopt
--refresh` proposes one.
