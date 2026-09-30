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

- [x] T001 **Pin — observe today's answers green before any production change** (plan *Pin* rows 1 and 2; SC-004).
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

- [x] T002 [US3] **Rule WG1 + WG2 (+ WG3) — a manifest with a workspace header is proposed the workspace commands,
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

- [x] T003 [US3] **Rule WG4 (+ WG5, WG9) — a Cargo workspace root owns the crates below it, so no member is proposed**
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

- [x] T004 [US3] **Rule WG6 — a Cargo workspace root is never owned by an outer owner; it is a candidate of its own**
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

- [x] T005 [P] [US3] **Rule WG7 — a member's lockfile is its workspace root's** (D13; WG7; SC-004). Depends on T001
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

- [x] T006 [US3] **Rule WG8 — an adopted Cargo workspace is recorded once, left alone by a second adopt, passes its
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

- [x] T007 [P] [US3] **Changelog fragment extended** (FR-009; `changelog.d/README.md`; plan *Constitution Check* I).
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

- [x] T008 [P] [US3] **One docs clause** (plan *Source Code*; not user-visible under `AGENTS.md`, so no fragment).
  Files: `docs/adopting.md`.
  Not a RED-GREEN increment. In the survey paragraph's list of builds that own what is below them, say a Cargo
  workspace owns its members like npm, Maven, Gradle and .NET do, and that a workspace nested below another is its own
  candidate. One clause; touch nothing else in the file.

## Design review

No screen in this slice.

## Model mockups

No white box in this slice: no event model, no screen states to write back; `check-model` has nothing to refuse.

## Phase 3: Polish

- [x] T009 [US3] **Run the gate.** Depends on T001–T008. No files written.
  Run `make verify` as the detached unit the safety rules give, poll it to completion, then `make test-adoption`
  under the wrapper; both green (`cargo` is at `~/.cargo/bin/cargo`, so the fixtures' `verify` runs and is not
  skipped). The one failure `test_changelog.test_every_release_this_repository_has_ever_tagged_has_an_entry` is
  environmental (upstream tags this clone has and the fork lacks) and is noted, not fixed; any other failure is real.
  Report each command's outcome. Do not commit red; do not touch any file not named above to make it pass — hand the
  failure back.


T009 done 2026-09-30 at `cb1b4ca`: `make verify` ran lint, typecheck, check-structure (152 modules) and 854 tests, `FAILED (failures=1, skipped=7)`; the one failure is `test_changelog.test_every_release_this_repository_has_ever_tagged_has_an_entry`, environmental (upstream's tags in this clone). `make test-adoption`: 8 fixtures adopted, re-surveyed, verified and migrated; `rust-workspace` and `rust-crate` verify green on day one and after `migrate`. Both under `systemd-run … MemoryMax=4G`.

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

## Phase 4: Convergence pass 1 — what the slice still owes

- [x] T010 **HIGH** [US3] **A Cargo workspace root that shares its directory with a manifest tried earlier owns
  nothing, so each member is proposed as a candidate of its own** (FR-006; SC-002; the edge case "reported once, by
  the ecosystem tried first"). Evidence: a probe tree with a root `package.json`, a root `Cargo.toml` holding
  `[workspace] members = ["crates/*"]` and `crates/a/Cargo.toml` (the napi-rs shape) surveys at HEAD as `.` (Node)
  and `crates/a` (Cargo, `cd crates/a && cargo check --all-targets`, no `--workspace`); a root `pyproject.toml`
  (maturin) beside the same workspace gives `.` (Python) and `crates/core` (`cd crates/core && cargo test`). A
  workspace of N members there is N Cargo candidates, and each runs one member's build against the root's lock.
  `buildable` (`survey.py:231-241`) stops at the first ecosystem that detects a directory, so the Cargo row, tried
  last, is never asked whether it owns anything. The two rules this slice wrote also disagree on that tree:
  `missing_lockfiles` treats `crates/a` as a member (no finding for it, `quick_wins.py:175-193`) while the survey
  proposes it as a build. **Product question for the delegating session, not decided here:** once the members are
  owned, what covers the Rust — (a) no Cargo candidate, the members owned and the survey page saying why; (b) a Cargo
  candidate at the same path beside the Node or Python one, which changes the one-candidate-per-directory shape; or
  (c) leave it and name it under *Not working yet* and in the fragment. **Sweep:** every row tried before Cargo
  (`node`, `python`, `go`, `maven`, `gradle`, `ant`, `dotnet`, `php`, `ruby`) × a Cargo workspace root in the same
  directory, one survey-boundary test per row family in `tests/test_survey_cargo_workspace.py`; and one membership
  predicate shared by `buildable`'s ownership and `missing_lockfiles`, so no tree can have a crate that is a member
  for one and a build for the other. Files (once decided): `src/slipwai/survey.py`, `src/slipwai/ecosystems/rows.py`,
  `src/slipwai/quick_wins.py`, the two suites, `changelog.d/rust-cargo-adopt.md`.
  **Decided (D14): option (a)** — the directory stays one candidate as the ecosystem tried first and is also a Cargo
  owner where its `Cargo.toml` declares a workspace; no member is proposed; one membership predicate for both rules;
  nothing new on the survey page. The sweep carries a member directory that also holds a `package.json` and stays Node.

- [x] T011 **MEDIUM** [US3] **Two conjuncts of the new rules have no test that fails without them** (Principle V's
  evidence gate). Evidence, by the sanctioned route, each file restored with `git checkout -- <path>` before the next:
  (1) `quick_wins.py:181`, dropping `str(manifest) in paths and` — so an *untracked* ancestor `Cargo.toml` that
  declares a workspace hides a tracked member's missing lock — leaves `make test TESTS="test_survey_cargo_workspace
  test_quick_wins test_survey_cargo test_survey"` green (50 tests, OK); (2) `cargo.py:14`, dropping the
  `[ \t]*[.\]]` terminator — so `[workspacefoo]` or `[workspace-x]` makes a root — also green. Two other probes were
  killed: `stands_alone` returning `False` (three `NestedWorkspaceTest` failures) and dropping the member's own-header
  check in `member_of_workspace` (`test_a_workspace_below_a_workspace_root_with_no_lock_of_its_own_is_still_reported`
  fails). **Sweep:** every conjunct of `WORKSPACE`, `declares_workspace`, `stands_alone` and `member_of_workspace`
  has an example at the boundary that fails when that conjunct alone is removed, each observed red by the sanctioned
  route; or run `/mutation` over the four and close each survivor. Files: `tests/test_survey_cargo_workspace.py`,
  `tests/test_quick_wins.py`.

- [x] T012 **MEDIUM** [US3] **The fragment says less than the code does, and once contradicts itself** (FR-009;
  `changelog.d/README.md`). Evidence, against `changelog.d/rust-cargo-adopt.md`: (a) *What stays out* no longer
  names that `members`, `exclude` and `default-members` are not read, nor an inline `workspace = { … }` table, so a
  crate below a root that the root excludes, and that declares no workspace, is owned and never proposed — a build
  the maintainer is not shown, which the fragment's "No member is proposed" does not tell them (plan *Not working
  yet*); (b) the **Catch-up.** paragraph opens "None is needed." and ends with a snapshot catch-up that asks the
  maintainer to remove a record — say instead that no release needs one and a snapshot adoption does this; (c) "A
  Cargo workspace root is one candidate, not one per member" is false for T010's shape until T010 is decided, and
  the sentence has to follow that decision; (d) line 7 runs past the file's wrap width. **Sweep:** each sentence
  of the fragment's Cargo text read against the test that proves it, with any sentence no test proves reworded or
  removed; `python3 -m pytest tests/test_changelog.py` under the wrapper. Files: `changelog.d/rust-cargo-adopt.md`.

- [x] T013 **LOW** **The `docs/adopting.md` clause breaks its sentence.** Evidence: `docs/adopting.md:27` now
  reads "every directory that builds — Node, …, Rust (Cargo, tried last), by the manifest that starts the build — a
  workspace root (…) owns its members …, except that a Cargo workspace nested below another is a candidate of its
  own — with its language, …": the second dash pair leaves "every directory that builds … with its language" with
  no verb in reach. **Sweep:** the survey paragraph read aloud end to end; the clause moved to a sentence of its
  own after the list. Files: `docs/adopting.md`.

- [ ] T014 **LOW** [US3] **The survey and the lockfile rule still disagree on a workspace root Git does not track**
  (T010's sweep: "no tree can have a crate that is a member for one and a build for the other"). Evidence: a
  repository committing only `a/Cargo.toml` (`[package]`), with `Cargo.toml` (`[workspace] members = ["a"]`) on disk
  and untracked, surveys at `f920ccb` as one candidate `.` (cargo) with `a` owned, while its quick wins report
  `no-lockfile` at `a/Cargo.toml`. The two callers pass `member_of_workspace` different `present` sets — the survey
  the files on disk (`survey.py:236`, the default at `cargo.py:30`), the lockfile rule the tracked paths
  (`quick_wins.py:182-184`) — so `cargo.py:27`'s "so they cannot disagree" holds only where the two sets agree, and
  `test_the_survey_and_the_lockfile_rule_name_the_same_members` (`test_survey_cargo_workspace.py:275`) passes every
  file as tracked. Rare (a member committed without its root) and the finding is advice, not a build, so it ships
  without. **Sweep:** either say in the docstring and the D13 text that ownership is read off the disk and the lock
  rule off what Git tracks, or give the shared-predicate test a tree with an untracked root and pin whichever answer
  is chosen. Files: `src/slipwai/ecosystems/cargo.py`, `tests/test_survey_cargo_workspace.py`.

## Convergence

**Pass 1 of 2 (2026-09-30): not converged.** One HIGH owed (T010), which re-opens the loop and needs a product
answer first; T011–T012 are MEDIUM and T013 LOW, none of which re-opens it. Reviewed diff `a07f1e0..HEAD` at
`18da4ab`. `make test TESTS="test_survey_cargo_workspace test_survey_cargo test_survey test_quick_wins test_adopt"`:
55 tests, OK. The full gate and `make test-adoption` were not re-run (green at `cb1b4ca`, T009; HEAD since changed
only this slice's `tasks.md`). Every run under `systemd-run … MemoryMax=4G`, probe trees under
`$HOME/.cache/slippy-ws-tmp/`.

Per level:

- **Survey table** (`src/slipwai/ecosystems/`) — `WORKSPACE` and `declares_workspace` (`cargo.py:14-19`) and the
  flag (`cargo.py:25`) are the Design table's, proved at `test_survey_cargo_workspace.py:46-104` (WG1, WG2, SG3,
  malformed and unreadable manifests). `aggregates`' Cargo branch (`rows.py:279-280`) and `stands_alone`
  (`rows.py:288-291`) are keyed on `found.ecosystem == "cargo"` alone; `__init__.py` re-exports both. A single crate
  keeps `single-crate`'s commands word for word: `test_survey_cargo_workspace.py:89`, and the `rust-crate` fixture's
  whole survey is identical under `0e3bab3` and HEAD (below). Not proved: the regex terminator (T011).
- **Use case** (`survey.buildable` / `survey.survey`, `quick_wins`, `adopt`, `adopt --refresh`) — `buildable`'s
  exception (`survey.py:235`) gives WG4 and scenario 3 (`:113-131`), WG5 (`:149-165`, including `slipwai adopt
  --yes` recording exactly `.` and `src-tauri`), WG6 (`:202-218`) and the npm regression (`:222`). WG7 at
  `quick_wins.py:175-193`, proved at `test_quick_wins.py:231-277`, other ecosystems at `:268` and `:274`. WG9
  (`test_survey_cargo_workspace.py:169`) enters through `slipwai adopt --refresh`: root refreshed, member reported
  and left byte-for-byte. `adopt --refresh` as a no-op on the committed fixture and its `verify` and `migrate`: the
  harness at T009 (`scripts/test-adoption.py:58`, `:96`, `:127`). **SC-004, observed:** every fixture under
  `tests/fixtures/adopt/` was committed into a throwaway repository and surveyed with `0e3bab3:src` and with HEAD;
  the whole `Survey` (roots, commands, quick wins, every other field) is identical for all eight existing fixtures
  and differs only for `rust-workspace`. Not proved, and wrong: a Cargo workspace root beside a manifest tried
  earlier (T010).
- **Delivery adapter** (`cli_adopt` and `delivery/survey/survey.md`) — no code changed there. Adopting a copy of
  `rust-workspace` printed `adopt1: . (rust; what it is for is not recorded), 4 of 8 targets have a command`, the
  shared template; its survey page carries `` `.` — cargo, rust, from `Cargo.toml` `` and "no missing lockfile". No
  new outcome, so no adapter test is owed.
- **Screen** — none; a CLI.
- **Published contract** — the adopted `project.json` records one deployable at `.` with exactly the WG2 commands
  and `"version": ""`; `delivery/Makefile` carries `cargo fetch --locked` and the three ratchet lines with
  `--workspace` (read off the same adoption). The fragment keeps `MINOR`, experimental, and `VERSION` `1.4.0.dev0`,
  but under-says and once contradicts itself (T012); `docs/adopting.md:27` states the rule but breaks its sentence
  (T013). `delivery/survey/pinned.md` has not had the plan's two *Pin* rows appended — handed back to the delegating
  session by design, not a task here.

Constitution, for each principle the diff touches:

- **I. What a project was given keeps meaning what it meant** — holds for released answers: the eight existing
  fixtures survey identically (above); the ownership exception is `rows.py:291` and the lockfile one
  `quick_wins.py:193`, each keyed on Cargo; no release carried the Cargo row. The fragment's first line is `MINOR`
  and `VERSION` is `1.4.0.dev0`, checked by `tests/test_changelog.py` in the T009 gate. The experimental label
  is on the fragment's Cargo paragraph. What the fragment says is owed to T012.
- **II. Re-running is safe** — no new writing command. `adopt --refresh` is a no-op on the adopted fixture (T009's
  harness run); WG9 proves a member recorded by an earlier snapshot is reported and left exactly as written
  (`test_survey_cargo_workspace.py:169-198`).
- **III. Simplicity** — one regular expression (`cargo.py:14`), one predicate beside `aggregates` (`rows.py:288`),
  one guard in `buildable` (`survey.py:235`), one helper in `quick_wins` (`quick_wins.py:175`). T010 asks for the
  last two to become one membership rule.
- **V. Acceptance from Given-When-Then** — the use-case scenarios enter through `survey`/`buildable` over trees on
  disk and through `slipwai adopt` (`test_survey_cargo_workspace.py:159`, `:169`); no mocking library anywhere in
  the diff's tests or harness. The evidence gate is not met yet: two surviving probes (T011).
- **VIII. Versioning** — no new value in `project.json`; `VERSION` untouched at `1.4.0.dev0`, which the fragment's
  `MINOR` over the last released entry requires.
- **XIV. Agent-generated change meets the same bar** — the full gate green at `cb1b4ca` except the one
  environmental `test_changelog` tag failure (T009).
- IV, VI, VII, IX–XIII, XV — not touched: no port, no integration, no process, no money, time or identity value, no
  pipeline change.

**Pass 2 of 2 (2026-09-30): converged — the loop stopped at its bound with nothing CRITICAL or HIGH owed.** Reviewed
`3c3d8bb..f920ccb` (T010–T013, D14). One LOW appended (T014); it does not re-open the loop. `make test
TESTS="test_survey_cargo_workspace test_survey_cargo test_survey test_quick_wins test_adopt"`: 62 tests, OK.
`scripts/test-adoption.py --only rust-workspace` (`CARGO_TARGET_DIR` under `$HOME/.cache/slippy-ws-tmp/`): adopted,
re-survey a no-op, verify green on day one and after the migration. `make test TESTS=test_changelog`: 13 tests, the one
environmental tag failure only. The full gate was not run (it runs after demo acceptance). Every run under
`systemd-run … MemoryMax=4G`, probe trees under `$HOME/.cache/slippy-ws-tmp/p2/`.

Each pass-1 task, against its sweep:

- **T010 (HIGH) — closed, the whole class.** `buildable` asks `member_of_workspace` for every Cargo detection
  (`survey.py:236`), which reads each ancestor `Cargo.toml` whatever ecosystem reported its directory
  (`cargo.py:23-34`); `aggregates` no longer has a Cargo branch (`rows.py:271`), and `stands_alone` is gone (its one
  caller was `survey.py`; text search, as the tree has no `.codegraph/`). The row sweep is
  `test_survey_cargo_workspace.py:258` (all nine earlier rows as subtests), plus D14's member-with-`package.json` case
  (`:263`), a nested workspace beside an earlier manifest (`:267`) and a plain crate that owns nothing (`:271`). One
  predicate serves both rules (`quick_wins.py:182-184`), checked across four tree shapes at `:275` — except where
  the two are given different presence sets (T014, LOW). The override the fragment names works end to end: adopting
  the napi shape with `--language napi=rust --command 'napi:test=cargo test --workspace'` records one deployable at
  `.`, language `rust`, that test command, provenance `overridden`. A snapshot-recorded member in that shape is
  reported on `adopt --refresh` as `disagrees: a: nothing the survey recognises builds at crates/a any more; its
  record stands as written`, with the record byte-for-byte unchanged — the fragment's snapshot sentence holds there too.
- **T011 (MEDIUM) — closed.** Ten mutants, each applied alone to a committed file and restored with `git checkout --
  <path>` before the next, all killed by the five suites: the regex's line anchor, inner and trailing whitespace,
  the dotted form, and the `[.\]]` terminator (`:79`); the member's own-header check (`cargo.py:28`); the presence
  conjunct (`cargo.py:32`) and the lockfile rule's tracked set, each by `test_quick_wins.py:279`; the Cargo branch
  in `buildable`; and limiting the ancestors to the nearest one.
- **T012 (MEDIUM) — closed.** Each Cargo sentence of `changelog.d/rust-cargo-adopt.md` read against code and tests:
  the mixed-directory rule (`:12-18`) against `:258-271` and the override probe above; *What stays out* (`:21-23`)
  against `:130` (members not read) and `WORKSPACE`, which matches no inline table; **Catch-up.** (`:33`) now opens
  "No release needs one." and the snapshot sentence is the probe above and WG9. No line past the wrap width.
- **T013 (LOW) — closed.** `docs/adopting.md:27-33` reads as whole sentences, with the mixed-directory rule after the
  list.

Nothing pass 1 found closed was re-opened. WG1–WG9 and D11–D13 are held by the same tests, all green. **SC-004,
observed again:** every fixture under `tests/fixtures/adopt/` committed into a throwaway repository and surveyed
with `0e3bab3:src`, with pass 1's source (`3c3d8bb:src`) and with HEAD — the whole `Survey` is identical across all
three for the eight other fixtures (a single crate, `rust-crate`, included), and `rust-workspace` is identical
between pass 1 and HEAD.

Per level: **survey table** — `cargo.py:15` and `:23-34`, proved above; **use case** — `survey.py:236`,
`quick_wins.py:182-184`; **delivery adapter** — no code changed, the override and refresh outcomes observed through
`slipwai adopt`; **screen** — none, a CLI; **published contract** — the fragment and `docs/adopting.md`, above;
`project.json`'s shape unchanged (one deployable per directory, D14 option (a)).

Constitution, for each principle this diff touches:

- **I. What a project was given keeps meaning what it meant** — the eight existing fixtures survey identically to
  `0e3bab3`; the new ownership is keyed on `found.ecosystem == "cargo"` at `survey.py:236`, so no other row's
  candidates change (`test_survey_cargo_workspace.py:263`, a `package.json` in a member stays Node); no release
  carried the Cargo row. The fragment's first line is `MINOR` (`changelog.d/rust-cargo-adopt.md:1`), and it keeps
  the experimental label (`:17`).
- **II. Re-running is safe** — no new writing command; `adopt --refresh` is a no-op on the adopted fixture
  (harness, above) and leaves a snapshot-recorded member exactly as written (probe above; WG9 at
  `test_survey_cargo_workspace.py:169`).
- **III. Simplicity** — the diff removes more than it adds in `src/`: `stands_alone`, the Cargo branch of
  `aggregates` and `quick_wins`' private helper give way to one predicate (`cargo.py:23`) with two callers.
- **V. Acceptance from Given-When-Then** — every new example enters at `buildable` or `missing_lockfiles` over a
  tree on disk (`test_survey_cargo_workspace.py:250-289`, `test_quick_wins.py:279`); no mocking library; each
  conjunct fails a test when removed (T011 above).
- **VIII. Versioning** — `VERSION` reads `1.4.0.dev0`, which the fragment's `MINOR` over the last released entry
  requires; `test_changelog` agrees apart from the one environmental tag failure.
- **XIV. Agent-generated change meets the same bar** — the quick suites, the adoption harness and the changelog
  suite above; the full `make verify` is the delegating session's next step.
- IV, VI, VII, IX–XIII, XV — not touched: no port, no integration, no process, no money, time or identity value, no
  pipeline change.

## Owed after the adversary pass and the post-converge gaps pass (2026-09-30)

From `specs/001-rust-cargo-adopt/adversary-log.md` rows W1–W4 and the post-converge gaps pass (G1–G8). Each is RED →
GREEN through a failing test where it is behaviour, under a new `implement` entry. Left open: W3's string and
quoted-header cases (named in the fragment instead, T017); G4 and G5's product halves (below, for a person); G8
(`delivery/survey/pinned.md` rows, not this slice's file).

- [ ] T015 [US3] **HIGH (W1, D15) — a Cargo workspace in a directory an outer owner hides is proposed as Cargo.**
  Files: `src/slipwai/survey.py`, `tests/test_survey_cargo_workspace.py` (or a new suite beside it if the budget
  needs). RED: root `package.json` with `"workspaces": ["packages/*"]`, `packages/native/package.json`,
  `packages/native/Cargo.toml` `[workspace] members = ["a"]`, `packages/native/a/Cargo.toml` plain → `buildable` is
  `.` (node) and `packages/native` (cargo, `cd packages/native && cargo test --workspace`), no `packages/native/a`;
  today `[('.', 'node')]`. GREEN: in `buildable`, when a directory's first detection is owned, try the Cargo row
  there and propose it where its manifest declares a workspace. Sweep: the same under a Maven `<modules>` root and a
  Gradle settings root; guards — an owned npm package with a plain `Cargo.toml` (no workspace) stays hidden, an owned
  npm package with a `go.mod` stays hidden (SC-004), `missing_lockfiles` agrees on each tree.
- [ ] T016 [US3] **MEDIUM (W2, G2) — a BOM before the header.** Files: `src/slipwai/ecosystems/cargo.py`,
  `tests/test_survey_cargo_workspace.py`. RED: a root `﻿[workspace]` with a member is one candidate with
  `--workspace`, and no `no-lockfile` for the member. GREEN: a leading U+FEFF counts as leading space, on the first
  line only or anywhere, as the implementer chooses and says.
- [ ] T017 [US3] **MEDIUM (G1, G6, W3, G5's statement) — the fragment and docs say what the code does.** Files:
  `changelog.d/rust-cargo-adopt.md`, `docs/adopting.md`, `tests/test_survey_cargo_workspace.py`. G1: the snapshot
  Catch-up says a root whose commands were *detected* (`adopt --yes`) is refreshed, and one confirmed or overridden
  is reported as a disagreement and edited in `project.json` by hand; add the WG9 example with `confirmed`
  provenance, asserting the disagreement line and that the record stands. G6: "No member is proposed" says a member
  is not proposed *as Cargo* (a member directory with a `package.json` is still Node, D14); "the `[workspace]` table
  header" says `[workspace]` or `[workspace.<x>]`. W3: *What stays out* adds a quoted `["workspace"]` header and a
  top-level `workspace.members` key. G5: the override sentence says the Node or Python gate is replaced by the
  commands given, so a maintainer who wants both writes a command that runs both. D15's case in one clause.
- [ ] T018 [US3] **LOW (W4, G7) — each manifest read once per survey.** Files: `src/slipwai/ecosystems/cargo.py`,
  `src/slipwai/survey.py`, `src/slipwai/quick_wins.py`, a test. GREEN: `declares_workspace` answers from a per-call
  memo (no module-level state that outlives a survey), and `missing_lockfiles` tests membership against a set. Test:
  a counting fake of the read (a function in the test tree passed through a seam, never a mock) proves one read per
  manifest for a root with many members.
- [ ] T019 [US3] **LOW (G3) — the fixture proves one deployable and that `--workspace` matters.** Files:
  `tests/fixtures/adopt/rust-workspace/Cargo.toml`, `scripts/test-adoption.py`. `default-members =
  ["crates/ledger"]` in the fixture (research: without `--workspace` a virtual root then tests only `ledger`), and
  `recorded()` asserts the Rust fixtures' `deployables` are exactly `[name]`. Observe red by dropping `--workspace`
  from the test command in `RUST_WORKSPACE_COMMANDS` and the row, restored.

For a person (not decided here, LOW):
- **G4** — a crate below a workspace root that the root `exclude`s and that holds its own `Cargo.lock` is hidden;
  a hand-written `"generated": false` record for it works, but every `adopt --refresh` then says nothing builds
  there. Should a `Cargo.lock` beside a crate below a workspace root make it a candidate of its own?
- **G5** — whether the survey should ever propose a combined Node-and-Cargo command for a mixed directory, rather
  than the fragment only saying the override replaces one gate with the other (T017 says the latter).
