---

description: "Tasks for slice optional-tools: audit and mutation are proposed where the crate configures them"
---

# Tasks: optional-tools — audit and mutation where the crate configures them

**Input**: `specs/001-rust-cargo-adopt/slices/optional-tools/plan.md` (Design, Pin, Source Code, Not working yet),
`research.md`, `specs/001-rust-cargo-adopt/spec.md` (US1 scenarios 3 and 4; FR-003; *Slice `optional-tools` — Gaps
reviewed*, OG1–OG7), `decisions.md` D18, D20 (and D7, D8). There is no `data-model.md` or `contracts/`: the slice adds
two constants and one function to a row, one keyed table to the ignore-block builder, and no interface.

**Cycle**: `.specify/drive.json` says `delegate=story`, `cycle=rule`. One delegate takes the whole of US1's configured
halves and each rule below is its own RED-GREEN-REFACTOR, one per commit. Every test enters at the survey's boundary
(`survey` / `buildable` over a tree written on disk) or at adopt's (`slipwai adopt --yes` in a throwaway repository,
through `tests/test_adopt.py`'s `repository` / `slipwai` helpers) and uses no mocking library (`AGENTS.md`): fakes
written in the test tree only.

**Format**: `[ID] [P?] [Story] Description` — `[P]` only where the task's files are disjoint from its siblings' and no
RED depends on another task's behaviour.

## HARD SAFETY RULES (every implementer reads these; follow them for every command you run)

- Wrap EVERY test run, `make verify`, `slipwai` invocation, `cargo` build, and any adversary or mutation probe in:
  `systemd-run --user --scope -q -p MemoryMax=4G -p MemorySwapMax=0 env -u CRUISE_RUNNER -u CRUISE_ITERATION TMPDIR=$HOME/.cache/slippy-optional-tools-tmp timeout <seconds> <command>`
  (quote PATH if you pass it; it contains spaces). `CRUISE_RUNNER` and `CRUISE_ITERATION` are unset for every test run
  and for `make verify` (D8). `make verify` takes about 35 minutes, beyond a single tool call's limit. Run it instead as a detached unit:
  `systemd-run --user --unit=slippy-optional-tools-verify -p MemoryMax=4G -p MemorySwapMax=0 --working-directory=/home/mbonnic/Slippy-worktrees/optional-tools --setenv="PATH=$PATH" --setenv="HOME=$HOME" --setenv=TMPDIR=$HOME/.cache/slippy-optional-tools-tmp /bin/bash -c "env -u CRUISE_RUNNER -u CRUISE_ITERATION make verify > $HOME/.cache/slippy-optional-tools-verify.log 2>&1"`
  and poll `systemctl --user show slippy-optional-tools-verify -p ActiveState,Result`.
- `/tmp` is a 7.6 GB RAM-backed tmpfs. Never put probe repos or build dirs there; use `$HOME/.cache/slippy-optional-tools-tmp/`.
- Known, not yours: `test_changelog.test_every_release_this_repository_has_ever_tagged_has_an_entry` fails locally only because this clone has upstream's `v1.4.0`, `v1.5.0` and `v1.5.1` tags, which the fork doesn't have. Treat that one failure as environmental. Any other failure is real.

Also (`delivery/docs/delegated-agent-safety.md`): the one sanctioned way to see a RED on code that exists, or to check
a guard has teeth, is to change the production file, run the test, and restore it with `git checkout -- <exact path>`.
Never `git stash`, never copy a tracked file aside. Do not commit from the tasks session; one task per commit in the
implementation session. `delivery/survey/pinned.md` is not this slice's to write (shared surface): the row the plan's
*Pin* names is handed back to the delegating session.

## Layers this slice covers

The Cargo survey row (T002, T003), the adopted record end to end through `slipwai adopt` and the adopted ignore block
(T004), the changelog fragment and one docs sentence (T005, T006) and the gate (T007). No screen, no service, no store:
the slice is a CLI's survey and its adoption.

## The example trees

Written on disk by named tests (helper `write` from `tests/test_survey.py`); the committed fixtures `rust-crate` and
`rust-workspace` are **not** changed (OG7):

- **Configured crate** — a root `Cargo.toml` with `[package]` and one of `deny.toml`, `.deny.toml`,
  `.cargo/deny.toml` (audit) or `.cargo/mutants.toml` (mutation). Examples of T002 and T003.
- **Configured workspace** — a root `[workspace] members = ["m"]` (also `[package]`, as the plan's research case) with
  the same files at the root, and a member `m/Cargo.toml` that carries copies of them. Examples of T002 and T003.
- **Subdirectory crate** — `crates/site/Cargo.toml` (plain, and a workspace root) with the files beside it. Examples
  of T002 and T003.

## Phase 1: Setup — the Pin

- [x] T001 [US1] **Pin — observe today's answers green before any production change** (plan *Pin*; SC-004). Files:
  `tests/test_survey_cargo_tools.py` (new; holds this slice's tests). No production file is touched. Not a
  RED-GREEN increment: a characterisation is green by design.
  - First run `make test TESTS="test_survey test_survey_cargo test_adopt test_mutation"` and confirm green, so a later
    red is the slice's and not the tree's.
  - Characterise at the adopt boundary: `slipwai adopt --yes` in a throwaway Node repository leaves an ignore block
    with neither `node_modules/` nor `mutants.out/` added by the factory (the plan's *Pin* row 1). Characterise a
    Cargo crate with none of the four files: audit and mutation are absent from the candidate and `slipwai adopt --yes`
    records `null` for both. Name each test for what it pins.
  - Characterise a Cargo crate adopted with `--yes`: its `.gitignore` block has no `mutants.out/`. This is the one
    assertion T004 inverts on purpose; name it so.
  - Observe each green.

## Phase 2: User Story 1 — Audit and mutation where the crate configures them (P1)

**Goal**: a Cargo candidate whose directory carries a cargo-deny configuration is offered `cargo deny check advisories`
as audit, and one carrying `.cargo/mutants.toml` is offered `cargo mutants`; elsewhere both stay a written no. The
adopted ignore block carries what `cargo mutants` writes.

**Independent test**: survey and adopt the example trees; `make test-adoption` still proves the unconfigured half
through the unchanged fixtures.

- [x] T002 [US1] **Rule OG1 + OG2 — a cargo-deny configuration in the candidate's directory proposes
  `cargo deny check advisories` as audit, and nothing else does** (scenario 3; FR-003; D18; D20; OG1, OG2). Depends
  on T001. Files: `tests/test_survey_cargo_tools.py`, `src/slipwai/ecosystems/cargo.py`.
  - RED: one test over the **configured crate** tree with a root `deny.toml` — `survey` returns candidate `.` whose
    audit is `cargo deny check advisories` and whose mutation is still absent. Observe it fail on the missing audit
    command, not on an import or a build error.
  - GREEN: add `DENY` and `optional_tools(root, directory, flag)` to `cargo.py` as the plan's Design table has them,
    returning **only the `audit` key** in this increment, and pass its result to `complete(…)` in
    `cargo(root, directory)`. `MUTANTS` and the `mutation` key are T003's, so its RED fails for its own reason.
  - Guards written in this increment, each observed to have teeth by the sanctioned route (break the row, run,
    `git checkout -- src/slipwai/ecosystems/cargo.py`) rather than as tests born green: `.deny.toml` and
    `.cargo/deny.toml` each propose it too (OG1); none of the three names present proposes no audit (FR-003); a
    `deny.toml` only in a member `m/`, and one only in the parent of the candidate, propose nothing (OG1, D20; the
    latter is the reversal the spec names under *Out of scope*); a directory named `deny.toml` is not a file and
    proposes nothing; the **configured workspace** root (WG1) gets `cargo deny --workspace check advisories` and a
    workspace in `crates/site` gets it prefixed once, `cd crates/site && cargo deny --workspace check advisories`
    (OG2, SG3); a plain crate in `crates/site` gets `cd crates/site && cargo deny check advisories`.
  - REFACTOR: the module docstring of `cargo.py` names the two configuration facts the row reads beside the manifest;
    no behaviour change, suite green.

- [x] T003 [US1] **Rule OG3 — `.cargo/mutants.toml` in the candidate's directory proposes `cargo mutants` as
  mutation, and nothing else does** (scenario 4; FR-003; D20; OG3). Depends on T002 (same two files; its RED needs
  `optional_tools` to exist). Files: `tests/test_survey_cargo_tools.py`, `src/slipwai/ecosystems/cargo.py`.
  - RED: one test over the **configured crate** tree with `.cargo/mutants.toml` — `survey` returns candidate `.`
    whose mutation is `cargo mutants` and whose audit is absent. Observe it fail on the value, not on setup. If T002's
    GREEN already produced it, the rule is cut too small: stop and report rather than writing a test born green.
  - GREEN: add `MUTANTS` and the `mutation` key to `optional_tools`, the minimum that makes it pass without breaking
    T002's examples.
  - Guards, each observed to have teeth by the sanctioned route (`git checkout -- src/slipwai/ecosystems/cargo.py`):
    a bare `mutants.toml` at the root proposes nothing (cargo-mutants does not read it); a member's
    `m/.cargo/mutants.toml` proposes nothing at the workspace root (the root's is the one read); the **configured
    workspace** root gets `cargo mutants --workspace`, a workspace in `crates/site` gets
    `cd crates/site && cargo mutants --workspace`, a plain crate there `cd crates/site && cargo mutants`; both files
    at once give both commands, independently; no `--in-diff` and no Make variable appear in either command.
  - REFACTOR: none expected beyond the docstring; suite green, and `python3 scripts/check-structure.py` holds the new
    suite under the 350-line budget (split by example tree if not).

- [x] T004 [US1] **Rule OG4 + OG5 — an adopted Cargo repository records both commands unguarded, carries
  `mutants.out/` in its ignore block, and a recorded mutation claims no rung** (OG4; OG5; D20; constitution I).
  Depends on T003. Files: `tests/test_survey_cargo_tools.py`, `src/slipwai/project/gitignore.py`.
  - RED: invert T001's ignore assertion — `slipwai adopt --yes` in a throwaway repository holding a Cargo crate with
    `.cargo/mutants.toml` produces a `.gitignore` block containing `mutants.out/` and `mutants.out.old/`; today it
    holds neither. Observe it fail on the missing lines.
  - GREEN: add `WRAPPED_ARTIFACTS = {"cargo": "mutants.out/\nmutants.out.old/\n"}` and the one line in
    `build_artifacts` that appends it once per ecosystem over `wrapped_of(apps)` in first-appearance order, exactly as
    the plan's Design table has them.
  - Guards written in this increment, each observed to have teeth by the sanctioned route (break the table or the
    line, run, `git checkout -- src/slipwai/project/gitignore.py`): the same adoption records `commands.audit` and
    `commands.mutation` in `project.json` exactly as surveyed, with no `command -v` guard and no Make variable around
    either (OG4); the block carries the two lines when the crate has neither configuration file (keyed on a recorded
    Cargo application, not on a recorded mutation command, OG5); a Cargo workspace with two members, or two Cargo
    candidates, carries each line once; a non-Cargo adoption (the Node repository of T001, and one more ecosystem)
    has a block byte-identical to T001's pin (SC-004); `build_artifacts` for a generated project is unchanged
    (`tests/test_mutation.py` stays green); the block is unanchored, so a crate in a subdirectory is covered.
  - Platform row (OG5): in a throwaway repository that holds a Cargo crate with `deny.toml` **and** a product the
    table dates, adopted with `--yes`, the Platform row reaches `audited` by the shared `platform_row` rule; a repo
    whose only product is the Rust crate stays `unknown` with the audit recorded (research; plan *Not working yet*).
    A recorded mutation alone moves no rung of any ladder. If the shared rule gives another answer, stop and report:
    it is a plan contradiction and not an addition to this task.
  - REFACTOR: none expected; suite green.

- [x] T005 [P] [US1] **Changelog fragment amended** (OG6; FR-009; `changelog.d/README.md`; plan *Constitution Check*
  I). Files: `changelog.d/rust-cargo-adopt.md`. Not a RED-GREEN increment: a fragment is a document and
  `tests/test_changelog.py` is its check. Keep the first line `MINOR`, keep it saying experimental and `VERSION` at
  `1.4.0.dev0`. Move audit and mutation out of *What stays out* into what the release does: with a cargo-deny
  configuration (`deny.toml`, `.deny.toml` or `.cargo/deny.toml`) in the candidate's directory audit is
  `cargo deny check advisories`, with `.cargo/mutants.toml` mutation is `cargo mutants`, each with `--workspace` at a
  workspace root and prefixed once in a subdirectory; neither is guarded, so a machine without the tool fails loudly;
  the adopted `.gitignore` block gains `mutants.out/` and `mutants.out.old/` for a Cargo application. Say what stays
  out: a `deny.toml` above the candidate, an offline run, a diff scope. Extend **Catch-up.**: a snapshot adoption is
  proposed the new commands by `slipwai adopt --refresh` as any detected command is, and the two ignore lines are
  added by hand to a repository adopted earlier because `migrate` and `adopt --refresh` never rewrite the block. Wrap
  at the file's width. Run `python3 -m pytest tests/test_changelog.py` under the safety wrapper.

- [x] T006 [P] [US1] **One docs sentence** (OG7; plan *Source Code*; not user-visible under `AGENTS.md`, so no
  fragment). Files: `docs/adopting.md`. Not a RED-GREEN increment. In the survey paragraph's treatment of the Cargo
  row, add one sentence naming the two trigger files for a Cargo candidate: a cargo-deny configuration in its
  directory for audit, `.cargo/mutants.toml` for mutation. Touch nothing else in the file.

## Design review

No screen in this slice.

## Model mockups

No white box in this slice: no event model, no screen states to write back; `check-model` has nothing to refuse.

## Phase 3: Polish

- [x] T007 [US1] **Run the gate** (OG7). Depends on T001–T006. No files written. Run `make verify` as the detached unit
  the safety rules give, poll it to completion, then `make test-adoption` under the wrapper (all with
  `env -u CRUISE_RUNNER -u CRUISE_ITERATION`); both green. Confirm `git status` shows no change under
  `tests/fixtures/adopt/` (the fixtures are unchanged, so the no-answer half stays end to end). The one failure
  `test_changelog.test_every_release_this_repository_has_ever_tagged_has_an_entry` is environmental and is noted, not
  fixed; any other failure is real. Report each command's outcome. Do not commit red; do not touch any file not named
  above to make it pass — hand the failure back.

## Dependencies & execution order

- T001 first: it creates the new suite and pins the seams green before anything changes.
- T002 needs T001; it owns `cargo.py` and is where `optional_tools` first exists.
- T003 needs T002 (same two files; its RED needs the function). T004 needs T003 (the same suite, and the adopted
  record is only meaningful once both commands survey).
- T005 and T006 need nothing from the others for their files; check T005's wording once T004 is done.
- T007 last.
- One task per commit; T005's fragment rides in the same pull request.

## Parallel opportunities

- **May run alongside:** T005 and T006 with each other and with T002–T004. `changelog.d/rust-cargo-adopt.md` and
  `docs/adopting.md` appear in no other task and read nothing the others write; neither has a RED.
- **May not:** T001–T004 are sequential: T002 and T003 share `cargo.py` and the suite, T003's RED depends on T002's
  function, T004 shares the suite and depends on both commands surveying. T001 precedes all and creates the suite.
  T007 waits for all.
- With `delegate=story` one delegate takes T001–T004 in order; T005 and T006 are a paragraph and a sentence, so a
  second agent saves little. No two concurrent tasks write the same file.

## Not working yet (owned by other slices or out of scope)

A `deny.toml` above the candidate proposes nothing (D20; spec *Out of scope*); a pure Rust repository's Platform row
stays `unknown` with an audit recorded until Rust has a `support.json` product; a repository adopted before this slice
gains the ignore lines only by hand; no offline advisories run and no `--in-diff` scope; the toolchain version is
always empty (`toolchain-pin`) and the adopted CI sets up no Rust (`ci-toolchain`); `delivery/survey/pinned.md` rows
are the delegating session's; `story-split.md` row 2 still names `cargo deny check`. No task here fixes them.
