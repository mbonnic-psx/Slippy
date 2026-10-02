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

## D4 — Gaps: how is a Rust `axum` service's OpenAPI document produced and held?
- **Stage:** slice gaps · **Slice:** http-axum · **When:** 2026-10-02T02:47:00Z · **Iteration:** 4
- **Question:** The http obligations include an OpenAPI document and its test. Fastify and FastAPI export theirs from the framework (`EXPORTERS`, `make check-openapi`); Go's `net-http` commits a hand-written `openapi.yaml` that a test holds to the routes (`openapi.py`, `http_openapi_test.go`).
- **Options:** (a) hand-written `openapi.yaml`, a Rust test holding it to the router's routes, as Go (recommended by the gaps pass) · (b) generated with utoipa, an `EXPORTERS` row and `check-openapi`
- **Decision:** (a). The document is committed and hand-written; a test in the service holds its paths to the routes the router registers; no exporter, no `check-openapi` recipe, no YAML crate added for the test's sake.
- **Why:** The parity bar is the backends that own no startup, and axum, like Go's mux, cannot list its own routes without a macro layer the other such backends do not carry; the person generating the service gets the same document and the same guarantee Go's user gets.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** the owner wants every backend's document generated from code, which would change Go's as well — a feature of its own
- **Written to:** specs/003-rust-every-axis/spec.md, specs/003-rust-every-axis/decisions.md
- **Status:** standing

## D5 — Gaps: two Rust `axum` services in one workspace both build a `serve` binary; name them apart?
- **Stage:** slice gaps · **Slice:** http-axum · **When:** 2026-10-02T02:47:30Z · **Iteration:** 4
- **Question:** Rust services share one Cargo workspace and one `target/`, so two services' `serve` binaries collide in name (as their `migrate` binaries already do); running two `make dev` at once on the host can trip over it.
- **Options:** accept it and say so in the generated docs (recommended by the gaps pass) · name each binary after its crate
- **Decision:** Accept it: every service keeps a `serve` binary, `dev_command` stays `cargo run --locked --bin serve` run from the service's own package, and the generated Rust page says two services' dev servers are run one at a time on the host or through Compose, where each has its own container.
- **Why:** `dev_command` is already recorded as `--bin serve` and the `migrate` binary already lives with the same shape; renaming one binary per crate changes a recorded command for a case `make demo` (one container per service) does not have.
- **Decided by:** host (stage recommendation)
- **Confidence:** medium · **Would reverse if:** a generated two-service Rust project's `make dev` fails rather than warns — then the binaries are named per crate
- **Written to:** specs/003-rust-every-axis/decisions.md
- **Status:** standing

## D6 — Gaps: the unreleased Rust fragments say "no transport"; amend them or supersede them?
- **Stage:** slice gaps · **Slice:** http-axum · **When:** 2026-10-02T02:47:45Z · **Iteration:** 4
- **Question:** `changelog.d/rust-backend.md` says Rust has no transport and `rust-event-store.md` says axum comes next; assembled into 1.4.0 beside this slice's fragment they contradict it.
- **Options:** amend the unreleased fragments (recommended by the gaps pass) · leave them and let the new fragment supersede
- **Decision:** Amend them: the sentences that will be false at release are corrected in place, and this slice's own fragment says what `axum` adds.
- **Why:** None of them has been released, so no reader has read them as a promise (Principle I); a release entry that contradicts itself is what the person upgrading would read. Feature 001's D20 amended its unreleased fragment the same way.
- **Decided by:** host (standing decision 001/D20)
- **Confidence:** high · **Would reverse if:** 1.4.0 is released before this slice merges — then the old lines are history and a new fragment supersedes them
- **Written to:** specs/003-rust-every-axis/decisions.md
- **Status:** standing

## D7 — Release constraint: how does `http-axum` reach users, and what gates it?
- **Stage:** release constraint · **Slice:** http-axum · **When:** 2026-10-02T02:48:00Z · **Iteration:** 4
- **Question:** `release: flagged` asks every slice to land dark; the factory has no flag mechanism of its own.
- **Options:** releasable on merge · held behind the `.dev` pre-release only `make release` turns into a release (recommended; 001's D7) · a coordinated deploy
- **Decision:** Held behind the pre-release, exactly as feature 001's D7: a merge to `main` publishes a `1.4.0.dev<N>` snapshot installers pass over unless asked; a person merges the PR and a person runs `make release`. No flag file.
- **Why:** That is this repository's dark launch; someone who generates with a released slipwai never meets a half-finished Rust transport, and the four later slices land behind the same gate.
- **Decided by:** host (standing decision 001/D7)
- **Confidence:** high · **Would reverse if:** the owner wants snapshots treated as releases
- **Written to:** specs/003-rust-every-axis/slices/http-axum/plan.md, specs/003-rust-every-axis/decisions.md
- **Status:** standing

## D8 — Convergence: scenario 9 says `--http none` is byte-identical but for `project.json`; three more files differ
- **Stage:** convergence · **Slice:** http-axum · **When:** 2026-10-02T09:43:00Z · **Iteration:** 4
- **Question:** At `--http none` the generated README's selection line, the shipped `scripts/backing-services.py` (which gains the `axum` rows in every backend's copy) and, on event-modelling, the re-resolved `Cargo.lock` also differ from the base (T019).
- **Options:** (a) amend scenario 9 to name the three (recommended by the delegate) · (b) keep the wording and suppress the README line for Rust's `none`, the other two being unavoidable
- **Decision:** (a). Scenario 9 names the recorded answer (both `project.json` and the README line), the shared pruner script and the re-resolved lock as the differences; every other byte is held by `tests/test_rust_http_none.py`. The fragment's sentence is corrected to match.
- **Why:** The README line restates the selection the record carries, as it does for every backend; the pruner cannot stay unchanged while it learns an option; a lock refresh moves transitive crates inside a MINOR. (b) would make Rust's README read unlike every other backend's for nothing the person generating gains.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** a file outside those three is found to differ at `--http none`
- **Written to:** specs/003-rust-every-axis/spec.md, changelog.d/rust-http-axum.md, specs/003-rust-every-axis/decisions.md
- **Status:** standing

## D9 — Convergence: keep `serde_path_to_error` in the axum region?
- **Stage:** convergence · **Slice:** http-axum · **When:** 2026-10-02T09:43:30Z · **Iteration:** 4
- **Question:** The implementer added `serde_path_to_error` so a 400 names the field of a wrong-type value, as Go's does (T021); it is a dependency the plan did not list.
- **Options:** (a) keep it, recorded in `research.md` (recommended by the delegate) · (b) drop it and answer `field: "(root)"` for a wrong type
- **Decision:** (a). It stays in the `axum` region and is recorded in the slice's crate table.
- **Why:** Parity with Go's 400 is the bar (the spec's parity rule), and it costs no package in any lock — axum's `json` feature already pulls it in; dropping it would tell the caller less than Go's service tells them.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** a lock is found to gain a package from it
- **Written to:** specs/003-rust-every-axis/slices/http-axum/research.md, specs/003-rust-every-axis/decisions.md
- **Status:** standing

## D10 — Adversary: two pruner flaws the pass found predate the slice; fix them here?
- **Stage:** adversary · **Slice:** http-axum · **When:** 2026-10-02T10:23:00Z · **Iteration:** 4
- **Question:** F2 (`backing-services.py --http <x>` rewrites every service of a mixed project) and F3 (re-locking offline needs a populated cargo cache) reproduce at the base for other backends and other axes; are they this slice's work?
- **Options:** fix both here · leave both open in the log, named in the PR, for a fix of their own that every backend shares (recommended) · fix F2 for Rust only
- **Decision:** Leave both open, not fixed in this slice. They are named in the PR and the slice's tasks as the factory's own, to be taken as their own change; the four findings this slice introduced or widened (F1, R1, R2, R3) are fixed here through failing tests.
- **Why:** The parity bar says Rust gets what every backend gets, and no more; a per-service `--http` in the shared pruner changes every backend's generated script and is a change of its own, which fixing for Rust alone would make Rust's pruner differ from the rest. Neither is CRITICAL, so neither pre-empts the next slice.
- **Decided by:** host (stage recommendation)
- **Confidence:** medium · **Would reverse if:** the owner wants mixed-project pruning fixed before Rust's transport ships
- **Written to:** specs/003-rust-every-axis/adversary-log.md, specs/003-rust-every-axis/decisions.md
- **Status:** standing
