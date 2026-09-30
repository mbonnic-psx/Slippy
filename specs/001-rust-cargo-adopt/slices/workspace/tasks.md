---

description: "Tasks for slice workspace: a Cargo workspace is proposed once, at its root"
---

# Tasks: workspace — a Cargo workspace is proposed once, at its root

**Input**: `specs/001-rust-cargo-adopt/slices/workspace/plan.md` (Design, Pin, Source Code, Not working yet),
`research.md`, `specs/001-rust-cargo-adopt/spec.md` (US3 scenarios 1–3; FR-006; SC-002; the reworded nested-workspace
edge case; WG1–WG9), `decisions.md` D11, D12, D13. There is no `data-model.md` or `contracts/`: the slice changes
one row of an in-process table, one predicate and one lockfile rule, and adds no interface.

**Cycle**: `.specify/drive.json` says `delegate=story`, `cycle=rule`. One delegate takes the whole of US3 and each
rule below is its own RED-GREEN-REFACTOR, one per commit. Every test enters at the survey's boundary (`survey` /
`buildable` over a tree written on disk; `slipwai adopt` in a throwaway repository through `tests/test_adopt.py`'s
`repository` / `slipwai` helpers; `scripts/test-adoption.py` for the committed fixture) and uses no mocking library
(`AGENTS.md`): fakes written in the test tree only.

**Format**: `[ID] [P?] [Story] Description` — `[P]` only where the task's files are disjoint from its siblings' and no
RED depends on another task's behaviour.

## HARD SAFETY RULES (every implementer reads these; follow them for every command you run)

- Wrap EVERY test run, `make verify`, `slipwai` invocation, `cargo` build, and any adversary or mutation probe in:
  `systemd-run --user --scope -q -p MemoryMax=4G -p MemorySwapMax=0 env TMPDIR=$HOME/.cache/slippy-ws-tmp timeout <seconds> <command>`
  (quote PATH if you pass it; it contains spaces). `make verify` takes about 35 minutes, beyond a single tool call's limit. Run it instead as a detached unit:
  `systemd-run --user --unit=slippy-ws-verify -p MemoryMax=4G -p MemorySwapMax=0 --working-directory=/home/mbonnic/Slippy-worktrees/rust-workspace --setenv="PATH=$PATH" --setenv="HOME=$HOME" --setenv=TMPDIR=$HOME/.cache/slippy-ws-tmp /bin/bash -c "make verify > $HOME/.cache/slippy-ws-verify.log 2>&1"`
  and poll `systemctl --user show slippy-ws-verify -p ActiveState,Result`.
- `/tmp` is a 7.6 GB RAM-backed tmpfs. Never put probe repos or build dirs there; use `$HOME/.cache/slippy-ws-tmp/`.
- Known, not yours: `test_changelog.test_every_release_this_repository_has_ever_tagged_has_an_entry` fails locally only because this clone has upstream's `v1.4.0`, `v1.5.0` and `v1.5.1` tags, which the fork doesn't have. Treat that one failure as environmental. Any other failure is real.

Also (`delivery/docs/delegated-agent-safety.md`): the one sanctioned way to see a RED on code that exists, or to check
a guard has teeth, is to change the production file, run the test, and restore it with `git checkout -- <exact path>`.
Never `git stash`, never copy a tracked file aside. Do not commit from the tasks session; one task per commit in the
implementation session. `delivery/survey/pinned.md` is not this slice's to write (shared surface): the two rows the
plan's *Pin* names are handed back to the delegating session.

## Layers this slice covers

The survey table and its ownership rule (T002–T004), the quick-wins rule that reads the lockfile (T001, T005), the
adopted record end to end through the committed fixture and the harness (T006), the changelog and one docs clause
(T007, T008) and the gate (T009). No screen, no service, no store: the slice is a CLI's survey.

## The example trees

Three trees, each written on disk by a named test (helper `write` from `tests/test_survey.py`):

- **Tauri shape** — root `package.json`; `src-tauri/Cargo.toml` holding `[workspace] members = ["helper"]`,
  `[workspace.package]`, `[workspace.dependencies]` and `[package]` with `[lib]`, `[[bin]]` and an `app` feature;
  `src-tauri/helper/Cargo.toml` a member. Example of T003 (WG5).
- **Pure virtual workspace** — root `[workspace] members = ["crates/*"]`, `crates/ledger` and `crates/report`, no
  `[package]` at the root. Example of T002 (WG2) and T003 (WG4), and the committed fixture of T006 (WG8).
- **Nested workspace root** — a workspace root with a cargo-fuzz style `fuzz/Cargo.toml` holding `[workspace]
  members = ["."]` and its own `[package]`. Example of T004 (WG6).

## Phase 1: Setup — the Pin

- [ ] T001 **Pin — observe today's answers green before any production change** (plan *Pin* rows 1 and 2; SC-004).
  Files: `tests/test_survey_cargo_workspace.py` (new; holds this slice's survey tests), `tests/test_quick_wins.py`.
  No production file is touched. Not a RED-GREEN increment: a characterisation is green by design.
  - First run `make test TESTS="test_survey test_survey_cargo test_quick_wins"` and confirm green, so a later red is
    the slice's and not the tree's.
  - In the new suite, one characterisation: a tree with a root `package.json` holding `"workspaces": ["packages/*"]`
    and, at `packages/app/package.json`, a package that itself holds `"workspaces": [...]`, surveys (`buildable`) as
    one candidate, `.`; the nested npm "workspaces" package is owned (WG6 regression, SC-004).
  - In `tests/test_quick_wins.py`, characterise `missing_lockfiles` at its boundary: a `Cargo.toml` with no
    `Cargo.lock` beside it is reported when it is a plain crate with none, when it is a member below a workspace root
    whose own `Cargo.lock` is tracked (the case D13 changes on purpose; T005 inverts this assertion), and a
    `package.json` with no lock is reported.
  - Observe each green. Name each test for what it pins; T005 edits the member one.

## Phase 2: User Story 3 — A Cargo workspace is proposed once, at its root (P3)

**Goal**: `slipwai adopt` on a Cargo workspace offers one candidate at the workspace root, whose commands cover
every member, and none per member; a separate workspace below it is a candidate of its own.

**Independent test**: survey the three example trees; adopt the committed `rust-workspace` fixture end to end.

- [ ] T002 [US3] **Rule WG1 + WG2 (+ WG3) — a manifest with a workspace header is proposed the workspace commands,
  and any other manifest keeps its commands** (scenario 2; D11; FR-008; WG1, WG2, WG3). Depends on T001.
  Files: `tests/test_survey_cargo_workspace.py`, `src/slipwai/ecosystems/cargo.py`,
  `src/slipwai/ecosystems/__init__.py`.
  - RED: one test over the **pure virtual workspace** tree — `survey` returns candidate `.` whose typecheck is
    `cargo check --workspace --all-targets`, lint `cargo clippy --workspace --all-targets --message-format=short -- -D
    warnings && cargo fmt --check`, test `cargo test --workspace`, install `cargo fetch --locked`, and integration,
    adversarial, audit, mutation still absent. Observe it fail on the missing `--workspace`, not on an import or a
    build error.
  - GREEN: add `WORKSPACE` and `declares_workspace(manifest)` to `cargo.py` and the `flag` in `cargo(root,
    directory)` exactly as the plan's Design table has them; re-export `declares_workspace` from
    `ecosystems/__init__.py`.
  - Guards written in this increment, each observed to have teeth by the sanctioned route (break the row, run,
    `git checkout -- src/slipwai/ecosystems/cargo.py`) rather than as a test that is born green: a manifest that is
    `[workspace]` and `[package]` gets the same flagged commands; `[workspace.package]` or `[workspace.dependencies]`
    alone, and an indented `  [ workspace ]`, make a root (WG1); `workspace = true` inside a dependency,
    `package.workspace = "…"`, a commented `# [workspace]` and a `[[workspace.x]]` array table do not; a malformed
    manifest is still Cargo, and is flagged exactly when such a header line is in it, and an unreadable manifest
    is not flagged; a root crate with no header keeps `single-crate`'s commands word for word (WG3); a workspace in
    `crates/site` has every command prefixed once, e.g. `cd crates/site && cargo check --workspace --all-targets`,
    lint `cd crates/site && cargo clippy --workspace … && cargo fmt --check` (SG3).
  - REFACTOR: the module docstring of `cargo.py` says the row reads one fact from the manifest; no behaviour change,
    suite green.

- [ ] T003 [US3] **Rule WG4 (+ WG5, WG9) — a Cargo workspace root owns the crates below it, so no member is proposed**
  (scenarios 1 and 3; FR-006; SC-002; WG4, WG5, WG9). Depends on T002.
  Files: `tests/test_survey_cargo_workspace.py`, `src/slipwai/ecosystems/rows.py`,
  `src/slipwai/ecosystems/__init__.py` (only if a name is re-exported).
  - RED: one test over the **pure virtual workspace** tree — `buildable` returns exactly one candidate, `.`, where
    today it returns three (`.`, `crates/ledger`, `crates/report`). Observe it fail on the count.
  - GREEN: `aggregates` in `rows.py` gains `if found.ecosystem == "cargo": return declares_workspace(here)` and its
    docstring names Cargo workspaces beside npm, Maven, Gradle and .NET.
  - Guards written in this increment, each observed to have teeth by the sanctioned route (break the branch, run,
    `git checkout -- src/slipwai/ecosystems/rows.py`): a root that is both `[workspace]` and `[package]` is still one
    candidate (scenario 3); a member is owned wherever it sits below the root within the survey's depth, whatever
    `members` says, and `members` is not read (WG4); a plain root crate with a `fuzz/Cargo.toml` that declares no
    workspace keeps SG5 — two candidates (WG3); **the Tauri shape** — `buildable` returns exactly `.` (Node, evidence
    `package.json`) and `src-tauri` (Cargo, evidence `src-tauri/Cargo.toml`, every command `cd src-tauri && …`
    with `--workspace`), with `src-tauri/helper` not proposed, and `slipwai adopt --yes` in a throwaway repository
    holding that tree records exactly those two deployables in `project.json` (WG5); **WG9** — in a throwaway
    repository adopted as the tree stood when the member was a candidate (its `project.json` written with the member
    as a deployable of its own and the root's old commands), `slipwai adopt --refresh` refreshes the root's detected
    commands to the WG2 ones and reports the member as `nothing the survey recognises builds at … any more; its
    record stands as written`, and leaves its record in `project.json`.
  - If the WG9 test is red for a reason outside `rows.py` (for example `resurvey` does not refresh or does not
    report), stop and report: the plan names no change to `resurvey`, so it is a plan contradiction and not an
    addition to this task.
  - REFACTOR: none expected beyond the docstring; suite green.

- [ ] T004 [US3] **Rule WG6 — a Cargo workspace root is never owned by an outer owner; it is a candidate of its own**
  (D12; the reworded edge case; WG6; SC-004). Depends on T003 (same files, and its RED needs `aggregates` to own the
  members first).
  Files: `tests/test_survey_cargo_workspace.py`, `src/slipwai/ecosystems/rows.py`,
  `src/slipwai/ecosystems/__init__.py`, `src/slipwai/survey.py`.
  - RED: one test over the **nested workspace root** tree — a workspace root with `fuzz/Cargo.toml` holding
    `[workspace] members = ["."]` — `buildable` returns two candidates, `.` and `fuzz`, the second with the workspace
    commands prefixed `cd fuzz && …`; today only `.` (the outer root owns it). Observe it fail on the missing
    candidate.
  - GREEN: add `stands_alone(root, found)` to `rows.py` (`found.ecosystem == "cargo" and aggregates(root, found)`,
    `False` for every other ecosystem), re-export it, and in `survey.buildable` make `owned = not stands_alone(root,
    found) and any(…as today…)`. Update the docstrings of `buildable` and the module to name Cargo workspaces.
  - Guards, each observed to have teeth by the sanctioned route (`git checkout -- src/slipwai/survey.py
    src/slipwai/ecosystems/rows.py`): the nested root owns what is below it (a plain crate under `fuzz/` is not
    proposed); a plain member under the outer root with no header stays owned (FR-006, SC-002); T001's nested npm
    `"workspaces"` package stays owned, so the exception is keyed on Cargo alone (SC-004); the nine other
    ecosystems' existing suites stay green.
  - REFACTOR: none expected; suite green, and `python3 scripts/check-structure.py` holds the new suite under the
    350-line budget (split by example tree if not).

- [ ] T005 [P] [US3] **Rule WG7 — a member's lockfile is its workspace root's** (D13; WG7; SC-004). Depends on T001
  (the pin it inverts) and T002 (`declares_workspace`); it reads nothing T003 or T004 write.
  Files: `tests/test_quick_wins.py`, `src/slipwai/quick_wins.py`.
  - RED: invert T001's member assertion — a `Cargo.toml` that declares no workspace, below a tracked `Cargo.toml`
    that declares one, with no `Cargo.lock` beside it, is not reported; observe it fail because today it is
    reported. Add, in the same RED, the root without a lock: reported once, at the root, and not for each member.
  - GREEN: in `missing_lockfiles`, for `Cargo.toml` only, skip a manifest with no lock beside it when it declares no
    workspace itself and some ancestor directory's tracked `Cargo.toml` declares one (its lock is judged at that root's
    own path). Everything else as today.
  - Guards, each observed to have teeth by the sanctioned route (`git checkout -- src/slipwai/quick_wins.py`): a
    separate workspace below a root (WG6) with no lock of its own is still reported; a plain crate with none beside
    it is still reported; a `package.json` member of an npm workspace with no lock is still reported and so is a
    `Gemfile` or `composer.json` (other ecosystems unchanged).
  - REFACTOR: none expected; suite green.

- [ ] T006 [US3] **Rule WG8 — an adopted Cargo workspace is recorded once, left alone by a second adopt, passes its
  own gate and survives a newer factory's `migrate`** (US3 independent test; SC-002, SC-003). Depends on T004.
  Files: `tests/fixtures/adopt/rust-workspace/Cargo.toml`, `tests/fixtures/adopt/rust-workspace/Cargo.lock`,
  `tests/fixtures/adopt/rust-workspace/crates/ledger/Cargo.toml`,
  `tests/fixtures/adopt/rust-workspace/crates/ledger/src/lib.rs`,
  `tests/fixtures/adopt/rust-workspace/crates/report/Cargo.toml`,
  `tests/fixtures/adopt/rust-workspace/crates/report/src/lib.rs`,
  `tests/fixtures/adopt/rust-workspace/README.md`, `scripts/test-adoption.py`.
  - RED: add `"rust-workspace": ([], "cargo", ("rust", "cargo"))` to `ADOPTIONS` (with a comment in the style of
    its siblings) and `RUST_WORKSPACE_COMMANDS` beside `RUST_COMMANDS`; extend `recorded()` so the fixture is held to
    an empty toolchain version and exactly the WG2 commands as it is for `rust-crate` (`python3
    scripts/test-adoption.py --only rust-workspace` stops on "no fixture under …" first). Add the fixture files one
    at a time until the run reaches the recorded-candidate assertion, so each failure is the harness's own.
  - GREEN: a virtual workspace, `members = ["crates/*"]`, two member crates `ledger` and `report` (`report` may take
    a path dependency on `ledger` and nothing else), one function and one `#[test]` in each `src/lib.rs`, clippy-,
    fmt- and test-clean, a root `Cargo.lock` generated by `cargo generate-lockfile` (no dependencies other than that
    path dependency), and a `README.md` as `rust-crate` has. `adopt` records one deployable at `.` with the WG2
    commands and no version; `adopt --refresh` changes nothing; `make -f delivery/Makefile verify` is green where
    `cargo` is on the machine (both members' tests run: observe `cargo test --workspace` list both once by hand) and
    skipped with the harness's reason where it is not; `migrate` under a newer factory holds.
  - Guard, observed to have teeth by the sanctioned route (change `RUST_WORKSPACE_COMMANDS` by one flag, run, restore
    with `git checkout -- scripts/test-adoption.py`): `recorded()` fails on a commands mismatch for the workspace
    fixture. `rust-crate`'s own check stays word for word.
  - REFACTOR: the two Rust expectations share one helper in `recorded()` if that reads better; `rust-crate` still
    green (`--only rust-crate`).
  - If the harness needs anything in `src/slipwai/` beyond T002–T004, stop and report: it is a plan contradiction.

- [ ] T007 [P] [US3] **Changelog fragment extended** (FR-009; `changelog.d/README.md`; plan *Constitution Check* I).
  Files: `changelog.d/rust-cargo-adopt.md`.
  Not a RED-GREEN increment: a fragment is a document and `tests/test_changelog.py` is its check. Keep the first line
  `MINOR`, keep it saying experimental and `VERSION` at `1.4.0.dev0`. Replace "This covers one crate" and the
  "and Cargo workspaces" item under *What stays out* with what this slice does: a Cargo workspace root is one
  candidate whose check, clippy and test carry `--workspace` (no `--all-features`; add it when confirming), no member
  is proposed, a workspace below a workspace root (a cargo-fuzz `fuzz/`) is a candidate of its own, and no
  "no lockfile" finding is reported for a member. Extend the **Catch-up.** paragraph: a repository adopted with a
  snapshot before this change has its root's commands refreshed by `slipwai adopt --refresh`, and a member it
  recorded as a deployable of its own is reported as no longer recognised and left as written — the maintainer
  removes that record from `project.json` if they want it gone. Run
  `python3 -m pytest tests/test_changelog.py` under the safety wrapper.

- [ ] T008 [P] [US3] **One docs clause** (plan *Source Code*; not user-visible under `AGENTS.md`, so no fragment).
  Files: `docs/adopting.md`.
  Not a RED-GREEN increment. In the survey paragraph's list of builds that own what is below them, say a Cargo
  workspace owns its members like npm, Maven, Gradle and .NET do, and that a workspace nested below another is its own
  candidate. One clause; touch nothing else in the file.

## Design review

No screen in this slice.

## Model mockups

No white box in this slice: no event model, no screen states to write back; `check-model` has nothing to refuse.

## Phase 3: Polish

- [ ] T009 [US3] **Run the gate.** Depends on T001–T008. No files written.
  Run `make verify` as the detached unit the safety rules give, poll it to completion, then `make test-adoption`
  under the wrapper; both green (`cargo` is at `~/.cargo/bin/cargo`, so the fixtures' `verify` runs and is not
  skipped). The one failure `test_changelog.test_every_release_this_repository_has_ever_tagged_has_an_entry` is
  environmental (upstream tags this clone has and the fork lacks) and is noted, not fixed; any other failure is real.
  Report each command's outcome. Do not commit red; do not touch any file not named above to make it pass — hand the
  failure back.

## Dependencies & execution order

- T001 first: it creates the new suite and pins both seams green before anything changes.
- T002 needs T001; it owns `cargo.py` and is where `declares_workspace` first exists.
- T003 needs T002 (`declares_workspace`); T004 needs T003 (same three files plus `survey.py`, and its RED needs the
  members owned first).
- T005 needs T001 and T002 only.
- T006 needs T004: the harness RED is wrong unless the fixture surveys as one candidate.
- T007 and T008 need nothing from the others.
- T009 last.
- One task per commit; T007's fragment rides in the same pull request.

## Parallel opportunities

- **May run alongside:** T005 with T003, T004 or T006 (after T002). Its files, `tests/test_quick_wins.py` and
  `src/slipwai/quick_wins.py`, appear in no task from T002 onward, and its RED needs only `declares_workspace`.
  T007 and T008 with anything: `changelog.d/rust-cargo-adopt.md` and `docs/adopting.md` appear in no other task and
  read nothing the others write (T007's wording should be checked once T003 and T004 are done).
- **May not:** T002, T003 and T004 are sequential: T003 and T004 share `rows.py` and the suite, T004's RED depends on
  T003's behaviour, T003's on T002's. T006 follows T004 (its RED depends on their behaviour). T001 precedes all,
  and T005 edits a file T001 wrote. T009 waits for all.
- With `delegate=story` one delegate takes T001–T006 in order; a second agent for T005 saves little and T007 and T008
  are a paragraph each, so they are not worth the coordination. No two concurrent tasks write the same file.

## Not working yet (owned by other slices or out of scope)

Audit and mutation are always a written no (`optional-tools`); the toolchain version is always empty
(`toolchain-pin`); the adopted CI sets up no Rust (`ci-toolchain`). `members`, `exclude` and `default-members` are
not read, so a crate below a root that the root excludes and that declares no workspace is owned and not proposed; an
inline top-level `workspace = { … }` table is not read; no feature matrix is proposed (D11);
`make -f delivery/Makefile smoke` still says none is recorded until a person regenerates the targets (D6). No task
here fixes them.

## Convergence

(The verdict comes after implementation.)
