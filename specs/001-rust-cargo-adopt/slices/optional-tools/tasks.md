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

## Phase 4: Convergence pass 1 (2026-10-01, at 05ef327)

Graded: only `CRITICAL` and `HIGH` re-open the loop. Each task closes the class its finding sits on; the sweep is
named in its GREEN.

- [x] T008 [US1] **HIGH — an attended adoption never gets the ignore lines** (OG5; ADR 0003; constitution I, the
  fragment's promise). `slipwai adopt` in a terminal records every directory as a *candidate* and `apps` is empty when
  the block is written (`src/slipwai/cli_adopt.py:245-250`, `src/slipwai/adopt.py:265`), and
  `slipwai adopt --confirm` regenerates the Makefile but never the `.gitignore` block (`src/slipwai/confirm.py`
  writes no block). Observed in a probe: a crate with `.cargo/mutants.toml`, adopted in a terminal and then
  `--confirm ledger`ed, records `mutation: "cargo mutants"` and its block has no `mutants.out/`. OG5 says the block
  carries it "where a Cargo candidate is recorded"; `changelog.d/rust-cargo-adopt.md:30-31` says the block gains it
  "for a Cargo application"; neither holds on the path ADR 0003 makes the default. Files: `src/slipwai/adopt.py`,
  `src/slipwai/project/gitignore.py`, `tests/test_survey_cargo_tools.py`.
  - RED: at the adopt boundary, `test_candidates.adopted(...)` over a crate (no tool files) — the block contains
    `mutants.out/\nmutants.out.old/\n` — fails today on the missing lines.
  - GREEN, sweep: every writer of the block and every way a Cargo application enters the record. The writers are
    `adopt.py:265` and `converge.py:171`; the entries are `adopt --yes` (deployables, already covered), `adopt` in a
    terminal (candidates), `adopt --confirm` (candidate → deployable, block already written). Key the lines on the
    recorded ecosystem of each wrapped application **and each recorded candidate** (both carry `toolchain.ecosystem`,
    `adopt.py:152`), so the block written at adoption already covers what `--confirm` later makes an application,
    and `converge.py`'s respell keeps them for a candidate still pending. Guards: a declined Cargo candidate leaves
    the lines (an ignored path that does not exist costs nothing, as `.env`'s comment argues); a terminal adoption of
    a Node-only tree has the block it always had.
  - Alternative, if the delegating session prefers it (OG5 allows it): name the attended path under the plan's
    *Not working yet* and correct the fragment sentence instead. Either way the fragment and the code must agree.

- [x] T009 [US1] **MEDIUM — the Catch-up overclaims what `adopt --refresh` proposes** (OG6; WG9;
  `changelog.d/rust-cargo-adopt.md:62-64`). "A snapshot adoption is proposed the new audit and mutation commands by
  `slipwai adopt --refresh` as it is any detected command" holds only for an `adopt --yes` record whose commands are
  `detected`. Observed: a record whose commands were `confirmed` (`adopt --confirm`), with `deny.toml` added later,
  is reported `disagrees: ledger: commands was confirmed as … the record stands until you decide` and `audit` stays
  `null`. Files: `changelog.d/rust-cargo-adopt.md`. Reword, as the paragraph above it already does for the
  workspace commands: refreshed where *detected*; a confirmed or overridden record is a disagreement, so add
  `commands.audit` / `commands.mutation` to `project.json` by hand and run `adopt --refresh` so the Makefile
  follows. Sweep: every Catch-up and docs sentence that says refresh proposes these commands (`changelog.d/`,
  `docs/adopting.md`), and re-wrap the fragment's first paragraph at the file's width (lines 6-8 are ragged after
  T005's edit).

- [x] T010 [US1] **MEDIUM — OG6 and the re-run of a configured tree have no test** (OG6; constitution II, "refresh on
  an unchanged tree changes nothing"). `make test-adoption` proves the no-op only for the unconfigured fixtures
  (OG7), and nothing in the suite enters `adopt --refresh` with a tool file present. Both behaviours were observed
  correct by probe; pin them. Files: `tests/test_survey_cargo_tools.py` (or a sibling suite if the 350-line budget
  is reached). Characterisation, green by design; prove teeth by the sanctioned route. Sweep over both tools and both
  provenances: an `adopt --yes` crate given `deny.toml` and `.cargo/mutants.toml` afterwards is refreshed to both
  commands and its `delivery/Makefile` `audit:`/`mutation:` recipes follow; a confirmed record given the same files
  is reported as a disagreement and left as written; a second `adopt --refresh` on a configured, committed tree
  leaves `git status` empty.

- [x] T011 [US1] **LOW — a generated Rust service beside a wrapped Cargo application lists the lines twice**
  (`src/slipwai/project/gitignore.py:91-93`). `build_artifacts` de-duplicates whole chunks, and `per_backend["rust"]`
  (`target/\nmutants.out/\nmutants.out.old/\nmutants.diff\n`) and `WRAPPED_ARTIFACTS["cargo"]` are different chunks:
  `build_artifacts(False, [generated rust, wrapped cargo])` counts `mutants.out/` twice (observed). Reachable only
  where an adopted repository gains a generated Rust service and its block is respelled (`converge.py:171`).
  Harmless to Git. Sweep: de-duplicate `language_artifacts` by line, not by chunk, so every per-backend and
  per-ecosystem table shares one rule.

## Convergence

**Verdict (pass 1 of 2): not converged** — one `HIGH` (T008) re-opens the loop; T009 and T010 (`MEDIUM`) and T011
(`LOW`) ride along with it. Pass 1 ran within its budget and is complete for the levels below.

**Levels.**

- **Domain — the Cargo row's rules.** Proven: OG1–OG3 at the survey boundary over trees on disk — all three deny
  names, directory-named impostors, member and above-the-candidate files, bare `mutants.toml`, the workspace and
  subdirectory forms of both commands (`tests/test_survey_cargo_tools.py:73-141`), implemented at
  `src/slipwai/ecosystems/cargo.py:53-67,84`. Two guards re-checked for teeth this pass by mutation and restore:
  dropping `.cargo/deny.toml` from `DENY` fails `test_each_of_the_other_two_names_cargo_deny_reads_proposes_it_too`;
  replacing `wrapped_of(apps)` with `apps` fails
  `test_an_application_the_factory_generated_adds_nothing_from_the_wrapped_table`. Not proven: nothing open.
- **Use case — survey → adopt record → refresh (OG6).** Proven: `adopt --yes` records both commands as surveyed
  (`tests/test_survey_cargo_tools.py:162`); the Platform row's `audited`/`unknown` and "mutation claims no rung"
  (`:201-218`). Observed by probe, not tested: refresh of a detected record proposes both, a confirmed record is a
  disagreement, a second refresh is a no-op (T010). Not holding: the attended adopt → `--confirm` path (T008).
- **Delivery adapter — adopted Makefile, ignore block, CI.** Observed in a probe adoption of a workspace in
  `crates/site`: `delivery/Makefile` `mutation:` → `cd crates/site && cargo mutants --workspace`, `audit:` →
  `cd crates/site && cargo deny --workspace check advisories`, unguarded (OG4); `verify` runs neither and the adopted
  CI runs only `install` and `verify` (`ci:` locally runs `audit`, which fails loudly without the tool, as OG4
  intends); neither is ratcheted (`src/slipwai/project/native_commands.py:42`). The block's lines are unanchored
  and once each for `--yes` (`tests/test_survey_cargo_tools.py:156-188`); missing on the attended path (T008);
  doubled beside a generated Rust service (T011).
- **Screen — the survey page and adopt report.** The adopt report counts "6 of 8 targets have a command" and
  `delivery/docs/adoption.md` lists the `audit` and `mutation` commands; the survey page names the candidate's
  evidence as `Cargo.toml` and does not name the trigger file, as every sibling row's conditional command does not.
  No finding.
- **Published contract.** `project.json`: no new key — `commands.audit`/`commands.mutation` already existed
  (constitution VIII). Fragment: first line `MINOR` (`changelog.d/rust-cargo-adopt.md:1`), experimental stated
  (`:23`), `VERSION` `1.4.0.dev0`; its Catch-up overclaims refresh (T009) and its ignore-block sentence fails on the
  attended path (T008). `docs/adopting.md:36-37` names the two trigger files (OG7).

**Constitution, principle by principle.**

- **I. What a project was given keeps meaning what it meant** — touched. Other ecosystems unchanged: the table is
  keyed by `ecosystem` and only `cargo` has a row (`src/slipwai/project/gitignore.py:36,92`); a generated project's
  `.gitignore` is unchanged because only `wrapped_of(apps)` contributes (`:92`, proven by
  `tests/test_survey_cargo_tools.py:233`, teeth re-checked); a non-Cargo adoption's block is unchanged (`:190`). Fragment
  in the same commits, `MINOR`, under the experimental exemption (`changelog.d/rust-cargo-adopt.md:1,23`). Not yet
  met: the fragment's claims at `:30-31` and `:62-64` must match the code (T008, T009).
- **II. Re-running is safe** — touched (refresh proposes new commands). No new writing command. The no-op is proven
  for the unchanged fixtures by `make test-adoption` (green at 633b34d) and observed for a configured tree by probe;
  T010 pins it.
- **III. Simplicity** — holds: two constants and one function (`src/slipwai/ecosystems/cargo.py:53-67`), one table and
  one expression (`src/slipwai/project/gitignore.py:36,91-93`); no abstraction added.
- **V. Acceptance-driven** — holds: every new test enters at `survey` or `slipwai adopt` except
  `IgnoreBlockTableTest`, which calls the public `build_artifacts` for the generated-app case adopt cannot reach; no
  mocking library (`tests/test_survey_cargo_tools.py:11-16`).
- **VIII. Versioning** — holds: no `project.json` field added or retyped; the commands are strings the record already
  carries.
- **IX. Security (dependency scanning)** — holds: the proposed audit is a vulnerability scan only,
  `check advisories` (`src/slipwai/ecosystems/cargo.py:65`, D18), so a recorded audit lifts the Platform row by the
  shared rule (`src/slipwai/convergence.py:182`; `tests/test_survey_cargo_tools.py:207`).
- **X. One pull request per slice** — holds: the slice's commits are its own branch, `dc904ea..05ef327`.
- IV, VI, VII, XI, XII — not touched: no domain/port code, no third-party adapter called at runtime, no long-running
  process, no pipeline change.

**Sweeps performed.** The three deny names × root/workspace/subdirectory (shared `here`, one test per name at the
root and the forms with `deny.toml` — one code path, no gap); both tools × both provenances on refresh (probe;
T010); every writer of the ignore block (`adopt.py:265`, `converge.py:171`) and every entry of a Cargo application
into the record (`--yes`, terminal candidates, `--confirm`) — found the attended gap (T008); every per-backend and
per-ecosystem ignore chunk for duplication — found T011.

**Not this slice's.** Smoke: the slice touched no start-up. Convergence map: no rung moved (Platform stays `unknown`
for a pure Rust repository; research). `delivery/survey/pinned.md` row is the delegating session's to append (plan
*Pin*).

**Verdict (pass 2 of 2): converged at the bound with one Phase 4 task** — T008–T011 each closed what they were
written for; no `CRITICAL` or `HIGH` remains. T012 (`LOW`) is appended below and does not re-open the loop. Pass 2 ran
at `9808f6b` within its budget and is complete. No codegraph index exists in this worktree (`delivery/scripts/codegraph
callers build_artifacts` → "not initialized"), so the writer sweep was a text search of `src/`.

**Each fix against its GREEN.**

- **T008 (closed).** `adopt` keys the block on each recorded candidate's ecosystem as well as each wrapped application's
  (`src/slipwai/adopt.py:265-267` → `src/slipwai/project/gitignore.py:65,95-104`). Proven at the adopt boundary in a
  terminal: a crate recorded only as a candidate (`deployables == {}`) has both lines; `--confirm ledger` leaves each
  once; a Node-only terminal adoption has no `mutants.out` (`tests/test_survey_cargo_tools_attended.py:25-46`). Teeth:
  `ecosystems = ()` at `adopt.py:265` fails the first two. Writers swept: `build_artifacts` has three callers —
  `scaffold.py:102` (generated, no candidates, so untouched), `adopt.py:267` (fixed), `converge.py:171` (applications
  only; named under the plan's *Not working yet*, `plan.md:140`; T012).
- **T011 (closed).** Wrapped and candidate lines are de-duplicated by line against what the generated backends already
  list (`gitignore.py:96-104`); the backends' own chunk text is untouched. Byte-identity checked by loading
  `git show dc904ea:src/slipwai/project/gitignore.py` as a scratch module (`$HOME/.cache/slippy-optional-tools-tmp/compare.py`)
  and comparing with HEAD's `build_artifacts`: identical for a generated Rust service (alone and with a web app), for
  `java`+`quarkus` beside `java`+`spring-boot` in both orders, and for wrapped Maven+npm, each across event ×
  `none`/`aws` (20 cases). Against the pre-fix `05ef327`: identical for one wrapped Cargo app, two, and Cargo+Maven; the
  only change is generated Rust beside wrapped Cargo, `mutants.out/` 2 → 1. Teeth: `if line not in listed` → `if True`
  fails `test_a_generated_rust_service_beside_a_wrapped_cargo_application_lists_each_line_once` and
  `test_two_wrapped_cargo_applications_carry_each_line_once`.
- **T010 (closed).** `tests/test_survey_cargo_tools_refresh.py:43-81` enters `adopt --refresh` with both tool files
  present across both provenances and the configured re-run. Teeth: `DENY = ()` (`ecosystems/cargo.py:53`) fails the
  detected-record test; refreshing regardless of provenance (`resurvey.py:80` → `if True`) fails the confirmed-record
  test. The second-refresh no-op is characterisation of the existing re-survey property (constitution II), not mutated.
- **T009 (closed).** The Catch-up now splits detected from confirmed/overridden and gives the by-hand route
  (`changelog.d/rust-cargo-adopt.md:61-66`), matching what the refresh test pins; first paragraph re-wrapped (`:3-21`).
  First line `MINOR` (`:1`), experimental stated (`:21`), `VERSION` `1.4.0.dev0`. `docs/adopting.md` makes no refresh
  claim about these commands (swept).

**Constitution, as the fix diff touches it.**

- **I. What a project was given keeps meaning what it meant** — holds. A generated project's `.gitignore` is
  byte-identical to `dc904ea` (above), so `make test-migration`'s exactness is not disturbed; the change is adoption-only,
  under the experimental exemption the fragment states (`changelog.d/rust-cargo-adopt.md:1,21`); the fragment's ignore
  sentence (`:28-30`) and Catch-up (`:61-66`) now say what the code does.
- **II. Re-running is safe** — holds: no new writing command; `adopt --refresh` on a configured, committed tree leaves
  `git status` empty (`tests/test_survey_cargo_tools_refresh.py:72-81`).
- **III. Simplicity** — holds: one optional parameter and one loop (`gitignore.py:65,100-104`), one expression in
  `adopt.py:265`; no new table or abstraction.
- **V. Acceptance-driven** — holds: the new tests enter at `slipwai adopt` / `--confirm` / `--refresh` through a terminal
  or the CLI; the dedupe test calls the public `build_artifacts`; fakes only, no mocking library.
- **X. One pull request per slice** — holds: fixes `fa23e9c..9808f6b` are on this slice's branch.
- **XIV. Agent-generated change meets the same bar** — holds: the spec and constitution are unedited; the converge edge
  is named in the plan rather than decided past the manifest.
- IV, VI–IX, XI–XIII, XV — not touched by the fix diff.

**Sweeps performed.** Every caller of `build_artifacts` (three); every way a Cargo directory enters the record
(`--yes`, terminal candidate, `--confirm`, declined — a declined candidate keeps the lines, harmlessly); both tools ×
both provenances on refresh; generated-project byte identity over Rust and the two Java frameworks × profile × target;
every Catch-up and docs sentence about refresh proposing audit/mutation. Targeted suites green: `test_survey_cargo_tools`,
`_attended`, `_refresh`, `test_adopt`, `test_mutation`, `test_candidates` — 76 tests OK.

- [ ] T012 [US1] **LOW — `slipwai converge` drops the Cargo lines for a still-pending candidate** (OG5;
  `src/slipwai/converge.py:171`; `plan.md:140`). `converge` respells the block from `apps_from_manifest(document)`
  alone, so a Cargo directory still in `project.json`'s `candidates` loses `mutants.out/` / `mutants.out.old/` that
  `adopt` wrote; the fragment (`changelog.d/rust-cargo-adopt.md:28-30`) does not say so. Harmless until `cargo mutants`
  runs there. Fix: pass `tuple((row.get("toolchain") or {}).get("ecosystem", "") for row in document.get("candidates",
  []))` as `candidate_ecosystems`, with a test at the `converge` boundary over a terminal-adopted crate. Needs
  `converge.py` in a manifest — the next slice that touches `converge`, or a follow-up PR.

## Phase 4 — from the gaps pass after convergence (2026-10-01; drive-gaps, opus, fresh context; gaps=6)

Traced OG1–OG7, FR-003, D18/D20, US1 scenarios 3–4, the fragment and the docs (14 items). Finding 3 is T012 again
and is not repeated. None reopens the loop (bound reached; none CRITICAL or HIGH).

- [ ] T013 [US1] **MEDIUM — a candidate overridden to Rust at `--confirm` gets no ignore lines** (OG5;
  `src/slipwai/adopt.py:265`, `src/slipwai/confirm.py:80`; `changelog.d/rust-cargo-adopt.md:28-30`). A napi-rs root
  (`package.json` + a `[workspace]`/`[package]` `Cargo.toml` + `.cargo/mutants.toml`) is a Node candidate at adopt,
  so its block has no `mutants.out/`; `--confirm napi --language napi=rust` records `ecosystem: cargo` and writes no
  block. Reproduction: the gaps probe `$HOME/.cache/slippy-optional-tools-tmp/gaps2/probe.py`. GREEN (recommended):
  key the Cargo lines on a `Cargo.toml` in a candidate's directory as well as on its surveyed ecosystem, so every
  path into a Cargo record (`--yes`, attended candidate, `--confirm`, `--language …=rust`) is covered — the sweep is
  every way an application's ecosystem becomes `cargo`. Alternative: narrow the fragment and add a by-hand sentence.
  Needs `adopt.py`'s candidate read (in this slice's manifest) and a test at the confirm boundary.
- [ ] T014 [US1] **LOW — a Cargo crate added after adoption gets no ignore lines, and the Catch-up says so only for
  earlier adoptions** (`changelog.d/rust-cargo-adopt.md:50-53,65-66`). GREEN: the by-hand sentence covers any Cargo
  record added after `adopt` wrote the block, not only a repository adopted before this release.
- [ ] T015 [US1] **LOW — OG4 at the Makefile level is asserted with `assertIn`** (`tests/test_survey_cargo_tools_refresh.py:55`).
  GREEN: assert the adopted `delivery/Makefile` `audit:` and `mutation:` recipes are exactly the recorded command
  (no `command -v` wrapper), and that neither target is in `native_commands.RATCHETED` nor run by `verify` — both
  tools, both forms.
- [ ] T016 [US1] **LOW — a symlinked `deny.toml` or `.cargo/` counts, though the comment says "regular file"**
  (`src/slipwai/ecosystems/cargo.py:52`; probe `gaps2/probe2.py`: a link to `/etc/hostname` proposes audit). GREEN:
  decide one reading for every trigger (all three deny names and `.cargo/mutants.toml`): follow links inside the
  repository and say so, or refuse links; make the comment and a test say the same.
- [ ] T017 [US1] **LOW — OG5's wording reads as if a recorded audit always lifts the Platform row** (`spec.md` OG5;
  `tests/test_survey_cargo_tools.py:201,207`). A pure Rust repository stays `unknown` (no `support.json` product).
  GREEN: the criterion and the fragment say "by the shared rule, once a product the table dates is in support", as
  `plan.md` *Not working yet* already does.
