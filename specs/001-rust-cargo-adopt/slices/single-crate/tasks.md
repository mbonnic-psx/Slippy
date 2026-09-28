---

description: "Tasks for slice single-crate: adopt proposes a one-crate Cargo repository"
---

# Tasks: single-crate — adopt proposes a one-crate Cargo repository

**Input**: `specs/001-rust-cargo-adopt/slices/single-crate/plan.md` (Design, Source Code, Pin, Not working yet),
`research.md`, `specs/001-rust-cargo-adopt/spec.md` (US1 scenarios 1, 2, 5 and the no-answer halves of 3 and 4;
Edge Cases; SG1–SG6). There is no `data-model.md` or `contracts/`: the slice adds one row to an in-process table
and no new interface.

**Cycle**: `.specify/drive.json` says `delegate=story`, `cycle=rule`. One delegate takes the whole of US1 and each
rule below is its own RED-GREEN-REFACTOR, one per commit. Every test enters at the survey's boundary (`survey` /
`buildable` over a fixture tree; `adopt` end to end in the harness) and uses no mock (`AGENTS.md`).

**Format**: `[ID] [P?] [Story] Description` — `[P]` only where the task's files are disjoint from its siblings'.

## Layers this slice covers

The survey table (T001), the adopted record through the end-to-end harness (T002), the changelog (T003) and the
gate (T004). No screen, no service, no store: the slice is a CLI's survey.

## Phase 1: Setup

None. The pin already exists (`plan.md`, *Pin*): `tests/test_survey.py` and `make test-adoption` for the nine
existing ecosystems. T001 begins by running both green, so a later red is the slice's and not the tree's.

## Phase 2: User Story 1 — A single-crate repository is proposed with its commands (P1)

**Goal**: `slipwai adopt` on a repository holding one crate offers one Rust / Cargo candidate, evidenced by
`Cargo.toml`, with an answer for all eight targets.

**Independent test**: survey a fixture tree holding one `Cargo.toml`; adopt the committed Rust fixture end to end.

- [x] T001 [US1] **Rule 1 — a directory holding a `Cargo.toml` is proposed as Rust built by Cargo, with its
  commands** (scenarios 1, 2, 5; the no-answer halves of 3 and 4; FR-001, FR-002, FR-004, FR-008; SG1, SG3, SG4,
  SG5; every edge case of this slice).
  Files: `tests/test_survey.py`, `src/slipwai/ecosystems.py`.
  - Pin first: run `python3 -m pytest tests/test_survey.py` and confirm green.
  - RED: one test, the pin's observation inverted — `buildable`/`survey` over a tree whose root holds one
    `[package]` `Cargo.toml` returns one candidate (today `()`): path `.`, language `rust`, ecosystem `cargo`,
    evidence `Cargo.toml`, install `cargo fetch --locked`, typecheck `cargo check --all-targets`, lint
    `cargo clippy --all-targets -- -D warnings && cargo fmt --check`, test `cargo test`, and integration,
    adversarial, audit and mutation absent (a written no), toolchain `{"kind": "rust", "version": ""}` even where
    the tree carries a `rust-toolchain.toml` (SG1). Observe it fail on the missing candidate, not on a build error.
  - GREEN: add `cargo(root, directory)` to `src/slipwai/ecosystems.py` exactly as the plan's Design table has it
    (found by file name alone; every command through `in_dir`; `complete(...)` for the rest) and append it to
    `ECOSYSTEMS` after `ruby`.
  - Guards written in this increment, each observed to have teeth by the sanctioned route (break the row, run,
    `git checkout -- src/slipwai/ecosystems.py`) rather than as a test that is born green: a crate in
    `crates/ledger` has every command prefixed once, lint as
    `cd crates/ledger && cargo clippy --all-targets -- -D warnings && cargo fmt --check` (scenario 5, SG3); a
    malformed `Cargo.toml` is still Cargo (edge case); no `Cargo.lock` leaves install `cargo fetch --locked`
    (edge case); a `Cargo.toml` under `target/`, `vendor/` or a skipped fixture directory is not proposed; a
    directory with `package.json` beside `Cargo.toml` is reported once, as Node, with `package.json` as the
    evidence (SC-004, Assumptions); a `fuzz/Cargo.toml` under a root crate is a candidate of its own (SG5); a
    `Dockerfile` beside the crate makes its role `service` and nothing beside it leaves the role unrecorded (SG4).
    Extend the precedent `test_a_repository_with_several_builds_reports_each_once_and_none_inside_an_owner` or add
    beside it; do not weaken it.
  - REFACTOR: add `rust` to the list of kinds in the `Detected.toolchain` docstring; no behaviour change, suite green.

- [x] T002 [US1] **Rule 2 — an adopted Rust crate is recorded, left alone by a second adopt, passes its own gate and
  survives a newer factory's `migrate`** (scenario 1 through the command's boundary; FR-008; SC-001, SC-003, SC-004;
  SG6). Depends on T001.
  Files: `tests/fixtures/adopt/rust-crate/Cargo.toml`, `tests/fixtures/adopt/rust-crate/Cargo.lock`,
  `tests/fixtures/adopt/rust-crate/src/lib.rs`, `tests/fixtures/adopt/rust-crate/README.md`,
  `scripts/test-adoption.py`.
  - RED: add `"rust-crate": ([], "cargo")` to `ADOPTIONS` (with a comment in the style of its siblings) before the
    fixture exists; `python3 scripts/test-adoption.py --only rust-crate` stops on "no fixture under …". Then add
    the fixture files one at a time until the run reaches the recorded-candidate assertion, so each failure is the
    harness's own.
  - GREEN: a `[package]` crate with a committed `Cargo.lock` (generated by `cargo generate-lockfile`, no
    dependencies), one function and one `#[test]` in `src/lib.rs`, clippy-, fmt- and test-clean, plus a
    `README.md` as `go-module` has. `adopt` records ecosystem `cargo`, toolchain kind `rust` with an empty
    version; `adopt --refresh` changes nothing; `make -f delivery/Makefile verify` is green where `cargo` is on
    the machine and skipped with the harness's reason where it is not; `migrate` under a newer factory holds.
    Confirms research's two *assumed* rows (`-D warnings` after `--`, `cargo fmt --check`) by running them.
  - REFACTOR: none expected; the comment on the `ADOPTIONS` row says why the fixture exists.
  - If the harness needs anything in `src/slipwai/` beyond T001 to pass (for example a page that raises on an
    unknown `kind`), stop and report: it is a plan contradiction (SG2 says unknown kinds are skipped), not an
    addition to this task.

- [x] T003 [P] [US1] **Changelog fragment** (FR-009; `changelog.d/README.md`).
  Files: `changelog.d/rust-cargo-adopt.md`.
  Not a RED-GREEN increment: a fragment is a document, and `tests/test_changelog.py` is its check. First line
  `MINOR`; then one bold sentence saying `slipwai adopt` now recognises a Cargo repository (one crate: install,
  typecheck, lint, test proposed; audit, mutation, integration and adversarial written as no answer); say the
  adoption path is experimental, per `AGENTS.md`; and say what stays out and comes later — toolchain pin, Rust
  setup in the adopted CI, workspaces. A **Catch-up.** paragraph: none is needed, an already-adopted repository
  has no Cargo candidate recorded and `slipwai adopt --refresh` proposes one; leave `VERSION` as `1.4.0.dev0`,
  which already meets a MINOR over `1.3.0`. Run `python3 -m pytest tests/test_changelog.py`.

## Design review

No screen in this slice.

## Model mockups

No white box in this slice: no event model, no screen states to write back; `check-model` has nothing to refuse.

## Phase 3: Polish

- [x] T004 [US1] **Run the gate.** Depends on T001–T003. No files written.
  Run `make verify`, then `make test-adoption`; both green (`cargo` is at `~/.cargo/bin/cargo` here, so the
  fixture's `verify` runs and is not skipped). Report each command's outcome. Do not commit red; do not touch any
  file not named above to make it pass — hand the failure back.

## Dependencies & execution order

- T001 first: it owns `src/slipwai/ecosystems.py` and the survey's tests.
- T002 needs T001's row, or its RED is the wrong one (the harness would find no candidate for a reason that is not
  the missing fixture).
- T003 needs nothing from the others.
- T004 last.
- Commit one task per commit; T003's fragment rides with the change in the same pull request.

## Parallel opportunities

- **May run alongside:** T003 with T001 or T002. Its one file, `changelog.d/rust-cargo-adopt.md`, appears in no
  other task, and it reads nothing they write.
- **May not:** T001 and T002 are sequential — different files, but T002's RED depends on T001's behaviour, so
  `[P]` would be unjustified. T004 waits for all. With `delegate=story` one delegate takes T001, T002 in order;
  a second agent for T003 is optional and not worth the coordination — the fragment is a paragraph.
- No two tasks write the same file, so nothing here needs a lock.

## Not working yet (owned by later slices)

Audit and mutation are always a written no (`optional-tools`); the toolchain version is always empty
(`toolchain-pin`); the adopted CI sets up no Rust (`ci-toolchain`); a workspace's members are proposed as
candidates of their own (`workspace`). No task here fixes them.

## Convergence

_To be written by the convergence pass._
