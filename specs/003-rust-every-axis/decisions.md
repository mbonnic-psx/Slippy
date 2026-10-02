# Decisions — Rust answers every axis and target

## D1 — Ground: the map rows this feature touches are still the survey's reading; proceed on them?
- **Stage:** ground · **Slice:** feature · **When:** 2026-10-01T20:20:00Z · **Iteration:** 3
- **Question:** Safety net, Structure, Path to production and Strategy are `detected`, not a person's answer, and this feature changes `catalog.json`, `assets/` and `src/slipwai/`, touching all four. Place them, or proceed?
- **Options:** ask a person and park · proceed on the detected values as stated assumptions, rows left as they are (stop table row 13, recommended)
- **Decision:** Proceed on the detected values, exactly as feature 001 does: the suite `make verify` is the safety net, the one application is the `slipwai` tool at the root, a change reaches users through the publish pipeline on a `v*` tag, and slices land in the existing package under `leave-it` (ADR 0002, `Proposed`). No row is moved or marked `confirmed`.
- **Why:** A fact about the world is not a decision; the work is additive inside the tool the map already names, the same ground feature 001 stood on (its D1, standing).
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** the owner places a row differently with `/ground`
- **Written to:** specs/003-rust-every-axis/decisions.md
- **Status:** standing

## D2 — What is Rust's `default.http` once Rust is listed under the http axis?
- **Stage:** product specification (pre-planning gaps) · **Slice:** none (feature-level; holds the http slice) · **When:** 2026-10-01T20:30:00Z · **Iteration:** 3
- **Question:** With Rust under `none` and a new `axum`, what does `slipwai generate --language rust` (or the interactive default) produce when nobody answers `--http`?
- **Options:** (a) `axum` — every other backend defaults to its own transport, and `make starters` regenerates from defaults · (b) `none` — a defaulted Rust project keeps generating exactly what it does today. No recommendation from the stage.
- **Decision:** (a). `catalog.json` `default.http` gains `"rust": "axum"`; `--http none` stays an answer anyone can give. `docs/axes.md`'s `--http` row lists `axum` among the per-backend defaults, and the http slice's fragment says a new Rust project now gets an HTTP service by default and a project generated before keeps what it recorded.
- **Why:** A Rust project already generated records no `http` key, and `Selection.option` answers an axis the record never asked with its `absent` (`none`), never the catalog default — so no answer it gave changes meaning, and this is MINOR, not MAJOR. Rust is unreleased (its fragment is still in `changelog.d/`). `axes.py` calls a default that silently resolves to nothing "a recommendation that silently stopped being made", and every later slice of this feature needs `http`.
- **Decided by:** drive-skipper (claude-opus-5-5[1m])
- **Confidence:** high · **Would reverse if:** an already-generated project's missing `http` answer is read back through the catalog default instead of the axis's `absent` — in `migrate` or `add-service` replaying a record — which would silently add HTTP to existing Rust projects
- **Written to:** specs/003-rust-every-axis/spec.md, specs/003-rust-every-axis/decisions.md
- **Status:** standing

## D3 — Split: which slices, in what order?
- **Stage:** split · **Slice:** feature · **When:** 2026-10-01T20:40:00Z · **Iteration:** 3
- **Question:** The issue orders http → auth → users → aws → azure, one PR each; is that the split, and can any run side by side?
- **Options:** the issue's order as a chain, one slice per axis or target (recommended) · `auth` and `users` concurrently after `http-axum` · per identity provider
- **Decision:** A chain of five: `http-axum`, `auth`, `users`, `aws`, `azure`, each its own PR. `auth` and `users` run in sequence because they write the same Rust rows of `prune.py`, the same marked regions of `Cargo.toml` and the same committed locks.
- **Why:** The issue's own order, `requires: http` on everything after the first, and a regenerated `Cargo.lock` on two branches is a conflict in a generated file; the four identity providers share one adapter.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** the lock is restructured so the identity crates live in a region of their own that two branches can add without conflict
- **Written to:** specs/003-rust-every-axis/story-split.md, specs/003-rust-every-axis/decisions.md
- **Status:** standing
