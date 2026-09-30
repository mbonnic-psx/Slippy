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

## Phase 4: Owed after convergence pass 1

None of these re-opens the loop (no CRITICAL or HIGH); each is what the slice may ship without, written so it is
not forgotten. `[P]` where the files are disjoint.

- [x] T005 [P] [US1] **MEDIUM — the `target/` guard has no teeth.** `tests/test_survey_cargo.py:65` puts its
  `Cargo.toml` at `target/debug/build/x/`, four levels down, past the walk's `DEPTH = 3`
  (`src/slipwai/survey.py:44`), so it passes with `"target"` removed from `SKIPPED` (`survey.py:40`) — observed
  in pass 1 by that mutation, restored; with the same mutation a `target/debug/Cargo.toml` *is* proposed. T001
  claimed every guard was observed to have teeth.
  Files: `tests/test_survey_cargo.py`.
  GREEN (the sweep): every skipped-directory case in that test — `target/`, `vendor/`, the skipped fixture
  directory — places its `Cargo.toml` within `DEPTH`, and each is observed red by removing that one name from
  `SKIPPED` (or the path from the `skipped` argument) and restored with `git checkout -- src/slipwai/survey.py`.

- [x] T006 [US1] **MEDIUM — nothing asserts what `adopt` records for a Rust candidate.** The harness's second
  element in `ADOPTIONS` (`scripts/test-adoption.py:51`) is the tool the gate needs, not the ecosystem; no test
  reads `project.json` for `rust-crate`. Pass 1 read it by hand (`--keep`): `language: rust`, `toolchain:
  {kind: rust, version: "", ecosystem: cargo}`, the four commands and four `null`s, provenance `detected` — correct
  today, pinned nowhere, though T002 claims it.
  Files: `scripts/test-adoption.py` (or `tests/test_adopt.py` — the implementer names one and says why).
  GREEN (the sweep): every `ADOPTIONS` row states the `toolchain.kind` and `toolchain.ecosystem` its adoption
  records, and the harness fails a fixture whose `project.json` deployable disagrees — `rust-crate`'s also with an
  empty `version` (SG1) and its eight commands exactly as `plan.md`'s Design table. Observed red by changing one
  row's expectation, then restored.

- [x] T007 [P] [US1] **MEDIUM — the factory's own docs still describe the table before this slice.**
  `docs/adopting.md:26` links `../src/slipwai/ecosystems.py`, which no longer exists, and its list (line 27) names
  nine ecosystems without Cargo; `docs/maintaining.md:104` draws `ecosystems.py` as one file. Not user-visible
  under `AGENTS.md` (docs/), so no fragment and no version change.
  Files: `docs/adopting.md`, `docs/maintaining.md`.
  GREEN (the sweep): `grep -rn "ecosystems\.py" docs src scripts README.md` returns nothing; every list of
  recognised ecosystems under `docs/` names Cargo, tried last; the tree in `maintaining.md` shows the package's
  four modules.

- [x] T008 [P] [US1] **MEDIUM — "overridable" (spec, Assumptions) is claimed, not proved.** A directory with
  `package.json` beside `Cargo.toml` is surveyed as Node (`test_survey_cargo.py:73`); the spec says the maintainer
  may override that. With the cargo row in `ECOSYSTEMS`, `survey.toolchain_as` (`survey.py:305`) now returns the
  Rust toolchain for `--language <app>=rust`, and `confirm.py:80` records it — by construction, untested.
  Files: `tests/test_adopt.py` (the precedent is `test_flags_override_the_survey_and_the_record_says_so`).
  GREEN (the sweep): for every language the survey can recognise in a mixed Node-and-other directory — here the
  new one, Rust — overriding `--language` records that ecosystem's toolchain (`kind: rust`, `ecosystem: cargo`)
  with provenance `toolchain: overridden`, and `adopt --refresh` afterwards is a no-op.

- [x] T009 [P] [US1] **LOW — a dead helper.** `tests/test_survey.py:33` adds `only`, which no test in that file
  calls (the Cargo suite defines its own at `test_survey_cargo.py:17`).
  Files: `tests/test_survey.py`, `tests/test_survey_cargo.py`.
  GREEN (the sweep): no helper in `tests/test_survey*.py` is defined without a caller; one `only` if both suites
  use it (imported, as `write` is), none in `test_survey.py` otherwise.

- [x] T010 [US1] **LOW — SG2's deliberate hole is not pinned.** Pass 1 adopted a copy of the fixture with a
  `.github/workflows/ci.yml`: `verify-delivery.yml` is written with checkout and `make -f delivery/Makefile verify`
  and no Rust step (`adopted_ci.py:57` skips the unknown kind), as SG2 says. No test holds it, so `ci-toolchain`
  has no pin to invert. May be taken as `ci-toolchain`'s own Pin stage instead; say which.
  Files: `tests/test_adopt.py`.
  GREEN (the sweep): for every toolchain kind an `ECOSYSTEMS` row can record and `SETUP` (`adopted_ci.py:18-26`)
  has no row for — today only `rust` — the GitHub gate workflow is written without a setup step and without error,
  and the GitLab job (`adopted_ci.py:115-130`; no `GITLAB_IMAGES` entry) names the kind in its "needs" line; the
  test derives the kinds from the two tables, so a later row without a setup is caught too.

## Convergence

**Pass 1 (2026-09-28): converged.** No CRITICAL or HIGH owed; T005–T010 are MEDIUM and LOW and do not re-open the
loop. Reviewed diff `80362b6..HEAD`. `make test TESTS="test_survey test_survey_cargo test_adopt"`: 20 tests, OK.
`scripts/test-adoption.py --only rust-crate` (cargo on PATH): adopted, re-survey a no-op, verify green on day one
and after `migrate` to 99.0.0. The full gate was not re-run (green on `2ae4f22`; HEAD changed only this file).

Per level:

- **Survey table** (`src/slipwai/ecosystems/`) — the cargo row (`cargo.py:10-22`) matches every row of
  `plan.md`'s Design table; it is last in `ECOSYSTEMS` (`__init__.py:24-26`). The nine rows and `aggregates` were
  compared function by function (AST source segments) against `80362b6:src/slipwai/ecosystems.py`: all ten
  identical. The helpers and `Detected` moved to `common.py` unchanged save `rust` in the toolchain docstring.
  Every importer (`confirm`, `platform`, `programme`, `survey`, `cli_adopt`, `delivery_facts`, `wrappers`,
  `structure`, `quick_wins`, two tests) imports only names `__init__.py` re-exports. Gap: T005.
- **Use case** (`survey.survey` / `buildable`, `adopt`) — detection, commands, subdirectory prefix, malformed
  manifest, no lockfile, mixed directory, fuzz crate, role are proved at the survey boundary
  (`test_survey_cargo.py:22-95`). What `adopt` records was observed, not asserted: T006. The override path the
  Assumptions promise: T008.
- **Delivery adapter** (`cli_adopt.py`) — the refusal names `Cargo.toml` (`cli_adopt.py:38`), asserted with a
  non-zero exit and stderr at `test_adopt.py:341`. The report for a Cargo candidate prints `. (rust; what it is
  for is not recorded), 4 of 8 targets have a command` — the shared template, as for every ecosystem; the survey
  page carries `cargo, rust, from Cargo.toml`.
- **Screen** — none; a CLI.
- **Published contract** — the adopted `project.json`, `delivery/Makefile` (the three ratchet lines and
  `cargo fetch --locked`) and `verify-delivery.yml` with no Rust setup (SG2, deliberate) were read from real
  adoptions. Unpinned: T006, T010.

Constitution:

- **I** — additive: the nine answers unchanged (above), Cargo tried last so SC-004 holds
  (`test_survey_cargo.py:73`); fragment `changelog.d/rust-cargo-adopt.md:1` claims `MINOR` and says experimental
  (line 7); `VERSION:1` reads `1.4.0.dev0`, already the MINOR over `1.3.0` (`tests/test_changelog.py` in the
  green gate); adopted migration is one clean merge, own files untouched (`test-adoption`, rust-crate).
- **II** — `adopt --refresh` on the adopted Rust fixture is a no-op, proved by `scripts/test-adoption.py:220-223`
  over the row at `:49-51`; the slice adds no writing command.
- **III** — the package split is justified against the structure budget at `plan.md:127-131`
  (`MODULE_BUDGET = 350`, `scripts/check-structure.py:68`; `ecosystems.py` was 350, 365 with the row); the tier
  entry `"ecosystems"` (`check-structure.py:44`) covers every submodule by prefix, so no gate was changed; no new
  abstraction — four plain modules, and it stays in `src/slipwai/` (leave-it, line 107).
- **V (as it holds today, lines 149-152)** — every new test enters at the survey boundary over a fixture tree
  (`buildable` / `survey`) or through `slipwai adopt` in the harness; no mock anywhere in the diff.
- **VII** — the refusal still goes to stderr with a non-zero exit (`test_adopt.py:338-341`).
- **VIII** — `rust` / `cargo` are new values, and readers skip an unknown kind (`adopted_ci.py:57`), observed.
- IV, VI, IX–XI — not touched.

`make -f delivery/Makefile check-convergence`: 1 of 9 axes at target, unchanged. No row this slice reached should
move: it adds a survey row and tests; nothing measures mutation (safety-net stays `tests-exist`) or changes the
structure, platform or pipeline evidence.

Handed back, not decided: `survey.DEPENDENCY_MANIFESTS` (`survey.py:80-83`) does not read `Cargo.toml`, so a
database driver a crate names (`sqlx`, `diesel`, `tokio-postgres`) is not reported. SG's out-of-scope list names
the manifest-keyed pages (#11) but not this one — the owner decides whether it is #11's or a task here.

## Re-opened by the post-converge gaps pass (2026-09-29)

- [x] T011 [US1] **HIGH — each clippy finding is its own ratchet key** (D10; SG3 as amended). Files:
  `src/slipwai/ecosystems/cargo.py`, `tests/test_survey_cargo.py`, a ratchet test beside the existing ones for
  `assets/adoption/scripts/ratchet.py` (the one that already feeds other tools' output to `findings_in`),
  `changelog.d/rust-cargo-adopt.md`. RED: the survey tests expect `--message-format=short` in every lint proposal
  (root and subdirectory) and a ratchet test feeds `findings_in` clippy's short output with two findings in one file
  and expects two keys — observe both fail for their own reason. GREEN: the row proposes the amended command; the
  fragment quotes it. Sweep: every place the lint string is asserted or quoted.
- [x] T012 [US1] **MEDIUM (Phase 4) — a machine without the clippy or rustfmt component is not refused.** A missing
  cargo subcommand exits 101, as a clippy `-D warnings` failure does, so the ratchet records `{exit: 101,
  findings: []}` and passes on it after. Close it for every cargo subcommand the row proposes, in the ratchet
  (`assets/adoption/scripts/ratchet.py`, cargo's `no such command` treated as not runnable), with its test.
- [x] T013 [US1] **LOW (Phase 4) — the fragment's Catch-up says refresh proposes a Cargo candidate; it reports it
  `not wrapped`.** Reword `changelog.d/rust-cargo-adopt.md` to say what a maintainer does (`resurvey.py:174-178`,
  `:343-347`), and add a refresh test for a Cargo root in an adopted repository after `tests/test_pin.py:117`.

**Pass 2 (2026-09-29): converged — confirming pass; the loop stopped at its bound (2 of 2).** No CRITICAL or HIGH
owed; nothing appended. Reviewed diff `cd2d3f3..HEAD` against D10 and T011. `make test TESTS="test_ratchet_cargo
test_survey_cargo test_survey test_adopt"`: 21 tests, OK; `make lint` and `make test TESTS=test_changelog` green.
`scripts/test-adoption.py --only rust-crate` (cargo on PATH): adopted, re-survey a no-op, verify green on day one
and after `migrate` to 99.0.0 — with the amended lint. The full gate was not run (it runs after demo acceptance).

- **Every lint proposal is the D10 command.** There is one proposal site, `src/slipwai/ecosystems/cargo.py:18-20`,
  and `in_dir` prefixes it for root and subdirectory alike; both are asserted verbatim at
  `tests/test_survey_cargo.py:35-38` (root) and `:56-60` (`crates/ledger`). Every other `clippy` in `src/` is the
  generated project's own gate (`project/languages/rust.py:35,120`, `project/docs.py:38`), untouched, as D10 says.
- **The ratchet keys each finding separately.** Real clippy 0.1.98 on a two-finding crate with the proposed
  flags, piped through `ratchet.findings_in` (`assets/adoption/scripts/ratchet.py:116`), gave two keys
  (`src/lib.rs: error: unneeded \`return\` statement`, `src/lib.rs: error: length comparison to zero: …`); the
  `could not compile` / `build failed` lines produced none. `tests/test_ratchet_cargo.py:52-63` holds the same
  with a fixture output. Real output matches the fixture's shape; its column numbers differ, which the key drops.
- **The test has teeth, though its `findings_in` half was never red.** Removing ` --message-format=short` from
  `cargo.py` (restored with `git checkout -- src/slipwai/ecosystems/cargo.py`) fails the new test at its first
  assertion (`test_ratchet_cargo.py:57`) and both survey tests — three failures, each for the flag. The
  `findings_in` assertions are a characterisation of the ratchet, not a RED: they would catch a ratchet change
  that merged same-file short lines, and the `DEFAULT` assertion (`:63`) records why the flag exists. Adequate
  for what D10 asked.
- **No regression elsewhere.** The diff touches only `cargo.py`, the two tests, the fragment and `tasks.md`;
  pass 1's per-level account stands unchanged, and T005–T010, T012, T013 are still owed as Phase 4.

Constitution this pass touched:

- **I** — `changelog.d/rust-cargo-adopt.md:4-6` quotes the D10 command and says why; line 1 still claims `MINOR`,
  `VERSION` unchanged at `1.4.0.dev0` (`test_changelog` green). The lint string changed from pass 1's, but no
  release carries it — the fragment is unreleased — so no project was given the old answer.
- **II** — the re-survey of the adopted `rust-crate` is still a no-op with the new command
  (`scripts/test-adoption.py:220-223`); the change adds no writing command.
- **V (as it holds today, `.specify/memory/constitution.md:149-152`)** — the new test enters at the survey
  boundary over a fixture tree and calls the ratchet's own function loaded from the shipped script; no mock.
- III, IV, VI–XI — not touched by this diff.

## Mutation (2026-09-29)

**Not run: no mutation tool is configured here.** `project.json` records `commands.mutation: null` for `slipwai`,
and neither `mutmut` nor `cosmic-ray` is installed (`.python-tools/` carries ruff and mypy only). The setup
decision a person owes: choose a Python mutation tool, record it as `commands.mutation`, and scope it to changed
paths. Until then the evidence gate stands on what the teeth checks recorded above show — each Cargo test observed
failing against a deliberate break of `cargo.py`, `survey.py` or `ratchet.py` and restored (T001, T005, T008,
T011 pass 2, T012) — which is recorded reachability, not a mutation score.

## Owed after the adversary pass (2026-09-29)

From `specs/001-rust-cargo-adopt/adversary-log.md`, rows R1–R5. Each task is RED → GREEN through a failing test,
under a new `implement` entry. S1/S2, the directory name that reaches the shell or the Makefile, predates this
slice and is fixed off `main` on its own branch (`fix-unsafe-candidate-paths`, GHSA-3fpx-wg55-c4qj). Merge that
first; S3 falls with it, because a name with a space is no longer proposed. R6–R8 and S4 stay open in the log.

- [x] T014 [US1] **HIGH (R1) — the tool is read past `cd <dir> &&`, `VAR=… ` and `env …`.** `ratchet.py`
  takes the first word after `cd` as the tool, so the survey's own `cd ledger && cargo clippy …`, for any crate
  one level down, is never matched as `cargo`, and a missing clippy is baselined. Files:
  `assets/adoption/scripts/ratchet.py`, `tests/test_ratchet_cargo.py`. RED: a stub `cargo` on PATH printing
  ``error: no such command: `clippy` `` with exit 101, run through the ratchet as `cd sub && cargo clippy`,
  `RUSTFLAGS=-Dwarnings cargo clippy` and `env RUSTFLAGS=x cargo clippy`; each is refused with nothing recorded,
  exactly as the bare `cargo clippy` already is. Reproduced by the host on 2026-09-29.
- [x] T015 [US1] **HIGH (R2) — rustup's "component not installed" is not runnable either.** Confirmed by the host
  on 2026-09-29 against a linked toolchain holding only `cargo` and `rustc`, with rustup 1.29.0: `cargo clippy`
  and `cargo fmt --check` print ``error: 'cargo-clippy' is not installed for the … toolchain …`` (and
  `'cargo-fmt'`) and exit **1**. Files: `ratchet.py`, `tests/test_ratchet_cargo.py`. RED: a stub printing that
  line with exit 1 is refused, naming `cargo clippy` / `cargo fmt`; the fragment and the docstring say both
  forms.
- [x] T016 [US1] **HIGH (R3) — a Rust test failure is keyed without the thread id.** libtest prints
  `thread 'tests::known_red' (520735) panicked at src/lib.rs:7:22:`, and the id changes every run. Key it by
  the test name, which is stable: a `TEST_FAILURES` pattern for libtest's `---- <name> stdout ----` or
  `test <name> ... FAILED` (this also closes R7). Files: `ratchet.py`, `tests/test_ratchet_cargo.py`. RED: two
  runs of the same failing output that differ only in the thread id give the same key, and a quarantined suite
  passes its second run.
- [x] T017 [US1] **HIGH (R4) — a crash is a new finding against a quarantined suite.** A test binary killed by a
  signal prints ``process didn't exit successfully: `…` (signal: 6, SIGABRT…)`` with exit 101 and no location,
  and the exit-code comparison accepts it. Files: `ratchet.py`, `tests/test_ratchet_cargo.py`. RED: with a
  quarantined baseline that has findings, an output with no findings at all and that `signal:` line fails as new.
  Decide and write down: does "no finding and a baseline that had some" always fail, or only on a signal?
  **Decided (2026-09-30): only the signal.** `CRASH` makes cargo's ``process didn't exit successfully: … (signal:
  N, SIGX…)`` a finding of its own, `crash: SIGX`, compared like any other. A known crash stays known. The general
  rule is unchanged, because other ecosystems' baselines rest on it.
- [x] T018 [US1] **MEDIUM (R5) — ANSI colour is stripped before `no such command` is matched.** With
  `CARGO_TERM_COLOR=always`, the line starts with `\e[1m\e[91merror`. Files: `ratchet.py`,
  `tests/test_ratchet_cargo.py`. RED: the coloured bytes (as recorded in the adversary log) with exit 101 are
  refused; strip the escapes before matching, not only for this pattern.

Done 2026-09-30, RED observed for every one of the ten new cases (`tests/test_ratchet_cargo.py`: tool past `cd`,
an assignment and `env` ×4; rustup's form ×2; colour; the same failure on another thread; a non-panicking failure;
a crash). GREEN in `assets/adoption/scripts/ratchet.py`: `tool_of`, `CARGO_NO_SUCH_COMMAND` with the rustup form
at exit 1 or 101, `ANSI` stripped before any match, libtest's `... FAILED` line in `TEST_FAILURES`, its panic line
in `NOT_A_FINDING`, and `CRASH`. R7 closed with R3. Every test run under `systemd-run … MemoryMax=4G`.
