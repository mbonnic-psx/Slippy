# Feature Specification: Adopt recognises a Rust (Cargo) repository

**Feature Branch**: `001-rust-cargo-adopt`

**Created**: 2026-09-28

**Status**: Draft

**Input**: User description: "slipwai adopt recognises a Rust (Cargo) repository, so an existing Cargo project (first user: Carin) can be adopted" — fork issue mbonnic-psx/Slippy#10.

## User Scenarios & Testing *(mandatory)*

The actor is a **maintainer of an existing Rust repository** who runs `slipwai adopt` on it to bring the delivery
method in beside the code. Today the survey reports "nothing here starts a build" for such a repository, so
adoption offers nothing to confirm and the adopted gate has no commands to run.

### User Story 1 - A single-crate repository is proposed with its commands (Priority: P1)

A maintainer runs `slipwai adopt` in a repository whose root holds one `Cargo.toml` for one crate. The survey
names the directory as a candidate, says it is Rust built by Cargo, says which file decided that, and proposes a
command for every one of the eight targets — Cargo's own tool where Cargo has one, a written "no answer" where it
has none. The maintainer confirms or overrides each, exactly as for any other ecosystem.

**Why this priority**: It is the whole value of the issue for the first user; without it a Rust repository
cannot be adopted at all.

**Independent Test**: Adopt a fixture repository holding one crate and read the proposed candidate and its
commands from the survey output and the recorded `project.json`.

**Acceptance Scenarios**:

1. **Given** a repository whose root holds a `Cargo.toml` with a `[package]` table, **When** the maintainer runs
   the survey, **Then** one candidate is proposed at the root, its language is Rust, its ecosystem is Cargo, and
   its evidence is `Cargo.toml`.
2. **Given** that candidate, **When** its commands are proposed, **Then** install is `cargo fetch --locked`,
   typecheck is `cargo check --all-targets`, lint is `cargo clippy --all-targets --message-format=short -- -D warnings` followed by
   `cargo fmt --check`, test is `cargo test`, and integration and adversarial are recorded as no answer.
3. **Given** a crate with no `deny.toml`, **When** commands are proposed, **Then** audit is recorded as no answer;
   **Given** a `deny.toml` beside the manifest, **Then** audit is `cargo deny check advisories` (D18).
4. **Given** a crate with no mutation configuration, **When** commands are proposed, **Then** mutation is recorded
   as no answer; **Given** a `.cargo/mutants.toml` (cargo-mutants' own configuration file) in the crate,
   **Then** mutation is `cargo mutants`.
5. **Given** a crate that lives in a subdirectory, not the root, **When** commands are proposed, **Then** every
   command runs in that directory, the same way other ecosystems' commands do.

---

### User Story 2 - The toolchain pin is read and the adopted gate's CI sets Rust up (Priority: P2)

The survey records the Rust toolchain the repository pins, so the adopted gate's CI job installs that toolchain
before it runs the recorded commands, and the gate runs green on a repository whose own commands pass.

**Why this priority**: Without the toolchain in CI, the adopted gate fails on the first push for a reason that
has nothing to do with the code.

**Independent Test**: Adopt a fixture that pins a toolchain, read the recorded toolchain, and read the generated
CI workflow for the Rust setup step; run `make -f delivery/Makefile verify` in the adopted fixture.

**Acceptance Scenarios**:

1. **Given** a `rust-toolchain.toml` with `channel = "1.85"`, **When** the survey runs, **Then** the recorded
   toolchain is Rust at `1.85`.
2. **Given** the legacy one-line `rust-toolchain` file, **When** the survey runs, **Then** its line is the version.
3. **Given** no toolchain file, **When** the survey runs, **Then** the toolchain is Rust with an empty version,
   as other ecosystems record an absent pin.
4. **Given** an adopted Rust repository on GitHub, **When** the adopted gate's CI workflow is written, **Then**
   it contains the same Rust toolchain setup step the factory already uses for generated Rust projects,
   placed before the gate runs.
5. **Given** an adopted Rust fixture whose own commands pass, **When** `make -f delivery/Makefile verify` runs
   there, **Then** it passes.

---

### User Story 3 - A Cargo workspace is proposed once, at its root (Priority: P3)

A maintainer adopts a repository whose root `Cargo.toml` declares a `[workspace]` with member crates. The survey
proposes one candidate, the workspace root, whose commands cover every member — not one candidate per member.

**Why this priority**: Most real Rust repositories past a single crate are workspaces; proposing each member
separately would record commands that fight over one lockfile and one target directory.

**Independent Test**: Adopt a fixture with a workspace root and two members; the survey proposes exactly one
candidate at the root.

**Acceptance Scenarios**:

1. **Given** a root `Cargo.toml` with a `[workspace]` table and members under `crates/`, **When** the survey runs,
   **Then** exactly one candidate is proposed, at the root, and no member is proposed on its own.
2. **Given** a workspace root, **When** commands are proposed, **Then** they run from the root and cover the
   workspace (`--workspace` where Cargo would otherwise build only the root package).
3. **Given** a root `Cargo.toml` with both `[workspace]` and `[package]`, **When** the survey runs, **Then** it is
   still one candidate at the root.

---

### Edge Cases

- A `Cargo.toml` inside `target/`, `vendor/` or a fixture or test directory the survey already skips for other
  ecosystems is not proposed as a candidate.
- A directory holding both a `Cargo.toml` and another ecosystem's manifest is reported once, by the ecosystem
  tried first; the survey says which file decided it, as it does today.
- A `Cargo.toml` that cannot be read as the survey expects (malformed) is still recognised as Cargo by its file
  name; nothing the survey proposes depends on parsing more than whether a `[workspace]` table is present.
- A `Cargo.toml` below a workspace root that itself declares a workspace (a `[workspace]` or `[workspace.<x>]`
  table) is a separate workspace, not a member — Cargo refuses a member that is also a workspace root — so it is
  proposed as a candidate of its own, with its own lockfile, and owns the crates below it; a crate below a
  workspace root that declares no workspace is a member and is not proposed (e.g. a cargo-fuzz `fuzz/` workspace
  under a workspace root is its own candidate). *(Reworded by D12: the line first read "A workspace member that
  is itself a nested workspace root is not proposed separately from the outer root.")*
- No `Cargo.lock`: install stays `cargo fetch --locked`, which fails loudly rather than resolving silently; the
  maintainer may override it when confirming.

### Slice `single-crate` — Gaps reviewed (2026-09-28, iteration 2)

Checked against `src/slipwai/ecosystems.py`, `survey.py`, `delivery_facts.role_of`, `project/adopted_ci.py`,
`platform.py`, `programme.py`, `structure.py` and `scripts/test-adoption.py`. The criteria and states this review
added, each owned by this slice:

- **SG1 — the toolchain before slice 3.** *(Replaced by TG1–TG10 in `toolchain-pin`.)* A crate adopted under this slice records its toolchain as kind `rust`
  with an empty version, whatever toolchain file it carries; `toolchain-pin` is what reads the pin. The survey
  page therefore shows no version for it, the way it shows none for any absent pin.
- **SG2 — no Rust setup in the adopted CI yet.** Until `ci-toolchain`, the gate's workflow writes no setup step for
  kind `rust` (`adopted_ci.SETUP` has no row, and a kind it does not know is skipped, as today). This is a
  deliberate hole, shown under *Not working yet*, and not a fault.
- **SG3 — the exact lint command.** At the root, lint is `cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt
  --check`; in a subdirectory `crates/ledger` every command is prefixed once, e.g.
  `cd crates/ledger && cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt --check`, as other ecosystems
  prefix theirs — the `cd` holds for both halves.
- **SG4 — what the crate is for stays a question.** A crate's role is read only by the rules every ecosystem
  already shares — a `Dockerfile` beside it makes it a service, a directory named as a test suite makes it one —
  and otherwise stays unrecorded, as for PHP and Ruby. Reading `src/main.rs`, `src/lib.rs` or `[[bin]]` is not
  in this feature.
- **SG5 — a crate that is not a workspace owns nothing below it.** A second `Cargo.toml` under a single crate
  (e.g. a `fuzz/` crate) is its own candidate in this slice; `workspace` is the slice that makes a root own its
  members.
- **SG6 — adopting again changes nothing, and the gate is green.** The Rust fixture goes through the same
  end-to-end path as every other adoption fixture (`make test-adoption`): adopt, `adopt --refresh` with no
  change, `make -f delivery/Makefile verify` green where `cargo` is on the machine and skipped with a reason where
  it is not, and a newer factory's `migrate`. So the fixture commits its `Cargo.lock` and passes its own clippy,
  fmt and test.
- **Out of scope here, stated so the audit does not reopen it:** the pages that key on an ecosystem — an upgrade
  path (`programme.UPGRADE_PATHS`), a tool proposed for an empty target (`programme.TOOLING`), the products a
  manifest pins (`platform.manifest_products`), dependencies and entry points in the structure view — say nothing
  Rust-specific. That is the axis answers of issue #11, which the Assumptions put out of scope.

### Slice `workspace` — Gaps reviewed (2026-09-30, iteration 3)

Checked against `src/slipwai/ecosystems/` (`cargo.py`, `rows.aggregates`), `survey.buildable`,
`quick_wins.missing_lockfiles`, `resurvey.reconciled_app`, `scripts/test-adoption.py` and cargo 1.98.0 run over three
probe trees (`slices/workspace/research.md`). The criteria and states this review added, each owned by this slice:

- **WG1 — what makes a workspace root.** A `Cargo.toml` with a line that starts (after spaces) with `[workspace]` or
  `[workspace.` — `[workspace.package]` or `[workspace.dependencies]` alone make one too, as Cargo reads them. A
  `workspace = true` key inside a dependency, `package.workspace = "…"`, and a commented-out `# [workspace]` do not.
  A malformed manifest is still Cargo, and is a workspace root exactly when such a header line is in it.
- **WG2 — the commands at a workspace root** (US3 scenario 2; D11). Typecheck `cargo check --workspace --all-targets`,
  lint `cargo clippy --workspace --all-targets --message-format=short -- -D warnings && cargo fmt --check`, test
  `cargo test --workspace`; install stays `cargo fetch --locked` and the fmt half stays `cargo fmt --check`, both of
  which already cover every member at the root (`research.md`). No `--all-features` (D11). The same for a virtual
  workspace and one that is also a `[package]`; in a subdirectory, prefixed once as SG3 says.
- **WG3 — a crate that is not a workspace keeps its commands.** A `Cargo.toml` with no workspace header is proposed
  exactly what `single-crate` proposes, and SG5 still holds for it: a `fuzz/Cargo.toml` under a plain root crate is
  its own candidate.
- **WG4 — no member is proposed** (US3 scenarios 1 and 3; FR-006; SC-002). A virtual root with members under
  `crates/` is one candidate, `.`; so is a root that is both `[workspace]` and `[package]`. A member is owned
  wherever it sits below the root within the survey's depth, whatever `members` says — `members` is not read.
- **WG5 — the Tauri shape.** A root `package.json` and a `src-tauri/Cargo.toml` that is both `[workspace]` and
  `[package]`, with a member at `src-tauri/helper`, are two candidates: `.` (Node, `package.json`) and `src-tauri`
  (Cargo, `src-tauri/Cargo.toml`, every command `cd src-tauri && …--workspace…`). Node does not own a Cargo crate,
  nor Cargo a Node package, as today. Adopted with `--yes`, `project.json` records exactly those two deployables.
- **WG6 — a nested workspace root is its own candidate** (D12; the reworded edge case). Below a workspace root, a
  `Cargo.toml` that declares a workspace is proposed in its own directory with the workspace commands, and owns what
  is below it. Only Cargo does this: a nested npm `"workspaces"` under an npm workspace root stays owned (SC-004).
- **WG7 — a member's lockfile is its workspace's** (D13). No "no lockfile" quick win for a member — a `Cargo.toml`
  that declares no workspace, below one that does: its lockfile is the nearest workspace root's, and a root without
  one is reported once, at the root. Still one for a separate workspace (WG6) or a plain crate with none beside it.
  Other ecosystems' lockfile findings are unchanged.
- **WG8 — adopted end to end** (US3 independent test; SC-002). A committed virtual workspace with two members goes
  through the same path as every other adoption fixture (`make test-adoption`): one deployable at `.` with the WG2
  commands, `adopt --refresh` a no-op, `make -f delivery/Makefile verify` green where `cargo` is on the machine — so
  both members' tests run and both are clippy- and fmt-clean — and a newer factory's `migrate` clean.
- **WG9 — a repository adopted with a snapshot before this slice.** Its root's detected commands refresh to the WG2
  ones on `adopt --refresh`; a member it recorded as a deployable of its own is reported (`nothing the survey
  recognises builds at … any more; its record stands as written`), never removed. No release carried the old answer
  (Principle I), and the fragment's Catch-up says what to do.
- **Out of scope here:** reading `members`, `exclude` or `default-members` (the Assumptions' text match); an inline
  top-level `workspace = { … }` table; a feature matrix (D11); what a member is for (SG4 holds per candidate).

### Slice `optional-tools` — Gaps reviewed (2026-10-01, iteration 2)

Checked against `src/slipwai/ecosystems/` (`cargo.py`, `rows.py`), `project/native_commands.py`, `project/makefile.py`,
`convergence.py`, `project/gitignore.py`, `scripts/test-adoption.py` and cargo-deny 0.20.2 and cargo-mutants 27.1.0
run over probe trees. The criteria and states this review added, each owned by this slice:

- **OG1 — what makes audit appear** (D20). The candidate's directory holds `deny.toml`, `.deny.toml` or
  `.cargo/deny.toml`. A file only in a member, or only above the candidate, proposes nothing: audit stays no answer.
- **OG2 — the audit command** (D18, D20). `cargo deny check advisories`; at a workspace root (WG1)
  `cargo deny --workspace check advisories`; in a subdirectory prefixed once (SG3). A vulnerability scan, as every
  other ecosystem's audit is — licences, bans and sources are the maintainer's to add when confirming.
- **OG3 — what makes mutation appear, and its command** (D20). `.cargo/mutants.toml` in the candidate's directory,
  which for a workspace is its root; a member's copy and a bare `mutants.toml` propose nothing, as cargo-mutants
  reads neither. `cargo mutants`, `cargo mutants --workspace` at a workspace root, prefixed once in a subdirectory;
  no `--in-diff` and no Make variable.
- **OG4 — a missing tool is a failure, never green.** Neither command is guarded; a machine without `cargo-deny` or
  `cargo-mutants` fails `make -f delivery/Makefile audit` / `mutation` with cargo's `no such command` (exit 101).
  Neither target is ratcheted, so nothing is baselined, and `verify` runs neither.
- **OG5 — what a recorded audit claims.** A recorded audit lifts the Platform row to `audited`, as for any ecosystem;
  a recorded mutation claims no rung. `cargo mutants` writes `mutants.out/`: the adopted ignore block carries it where
  a Cargo candidate is recorded, or the plan names it under *Not working yet*.
- **OG6 — adopted again.** A repository adopted with a snapshot before this slice is proposed the new commands on
  `adopt --refresh`, reconciled as WG9 says; the fragment's Catch-up says so.
- **OG7 — tested at the survey, fixtures unchanged.** Both adoption fixtures stay unconfigured, so the no-answer half
  stays end to end; the configured halves, every file name above, the member and above-the-candidate cases and the
  workspace forms are survey-level tests. `changelog.d/rust-cargo-adopt.md` is amended (no release carried it) and
  `docs/adopting.md` names the two trigger files.
- **Out of scope here:** a `deny.toml` above the candidate, which cargo-deny honours (D20's reversal); an offline
  advisories run; scoping mutation to a diff.

### Slice `toolchain-pin` — Gaps reviewed (2026-10-01, iteration 2)

Checked against `src/slipwai/ecosystems/` (`cargo.py`, `rows.py`, `common.py`), `bounded_read.py`, `resurvey.py`,
`adopt_report.py`, `platform.py`, `project/adopted_ci.py`, `project/ci_workflows.py`, rustup's overrides page and
the README of `actions-rust-lang/setup-rust-toolchain`. **SG1 is replaced** by the criteria below, each owned by
this slice:

- **TG1 — where the pin is found** (D19). From the candidate's directory up to and including the repository root,
  never above; the first directory holding `rust-toolchain` or `rust-toolchain.toml` decides, and the search stops
  there even when that file names nothing. A file rustup cannot read — a dangling link, a directory of that name, a
  FIFO or other non-regular file, an unreadable or non-UTF-8 file — is passed over, to the other name and then upward,
  as rustup does (D24).
- **TG2 — which file wins** (D21). In one directory, `rust-toolchain` over `rust-toolchain.toml`, as rustup does.
- **TG3 — what is read** (D21, D22). `rust-toolchain.toml`: `toolchain.channel`, by `tomllib`. `rust-toolchain`: its
  one line, stripped, with no leading `v` removed, where it is one line; TOML the same way where it is more than one,
  as rustup reads it. *(Refined by D22 at plan: the line first read "TOML the same way where it starts with `[`,
  otherwise its first non-blank line".)*
- **TG4 — recorded as written** (D21). `1.85`, `1.85.0`, `stable`, `nightly`, `nightly-2025-01-01` and `1.85-beta`
  are each the version verbatim; a test per shape.
- **TG5 — no pin is empty, never an error** (US2 scenario 3; D21). No file on the way up; a table with no `channel`
  or a non-string one; a `path` toolchain (never recorded: it is a path on someone's machine); invalid TOML; an empty
  or oversize file — each records `{"kind": "rust", "version": ""}`; so does a channel that is not a toolchain name a runner can install — anything outside `^[A-Za-z0-9][A-Za-z0-9._-]*$`, such as a newline, a space or a quote (D23). A `rust-toolchain.toml` starting with a byte order mark is read, as rustup reads it (D24). A file that is passed over (D24) is not an error and not a pin, and (D26) a file that resolves outside the repository is passed over. *(Refined by D24 at
  converge: the line first read "an empty, oversize or non-regular file".)*
- **TG6 — `rust-version` is not a pin** (D21). An MSRV in `Cargo.toml` with no toolchain file records an empty
  version.
- **TG7 — what a maintainer sees.** The survey page shows `rust <version>` for a pin and nothing for an empty one; the
  candidate's evidence stays `Cargo.toml`. The `rust-crate` fixture gains a pin, so the end-to-end adoption shows it,
  and `tests/test_survey_cargo.py`'s empty-version assertions flip to it.
- **TG8 — adopted again.** An app recorded with an empty detected toolchain refreshes to the pin on `adopt --refresh`;
  a toolchain the maintainer confirmed or overrode is reported as a disagreement and not changed (`resurvey`).
- **TG9 — what `ci-toolchain` is handed** (D19). The version string and nothing else; the record's shape is
  unchanged. `adopted_ci` still writes no Rust setup step in this slice (SG2 stands).
- **TG10 — the fragment.** `changelog.d/rust-cargo-adopt.md` is amended: the pin is no longer under *What stays out*,
  which keeps only the CI half; the Catch-up names TG8.
- **Out of scope here:** support status for a Rust version (`support.json` has no `rust` product); components and
  targets a toolchain file lists.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The survey MUST recognise a directory holding a `Cargo.toml` as a Rust candidate built by Cargo,
  with `Cargo.toml` (prefixed by its directory) as the evidence.
- **FR-002**: The survey MUST propose, for every one of the eight targets, either the Cargo command listed in
  User Story 1 or a written no-answer; it MUST NOT propose a command for integration or adversarial.
- **FR-003**: Audit MUST be proposed only where a `deny.toml` sits beside the manifest (or `.deny.toml`,
  `.cargo/deny.toml`, D20), and mutation only where cargo-mutants' configuration file is present; otherwise each is
  a written no-answer.
- **FR-004**: A candidate outside the root MUST have every command run in its own directory, as other
  ecosystems' commands are.
- **FR-005**: The survey MUST record the toolchain as kind Rust with the version the repository pins in
  `rust-toolchain.toml` (its `channel`) or `rust-toolchain`, or an empty version where it pins none.
- **FR-006**: A `Cargo.toml` declaring a `[workspace]` MUST aggregate the crates below it, so that no member (a crate below it that
  declares no workspace of its own, D12) is proposed as a candidate of its own.
- **FR-007**: The adopted gate's CI workflow MUST set up the Rust toolchain for a repository with a Rust
  candidate, reusing the setup step the factory already writes for generated Rust projects.
- **FR-008**: The change MUST ship with a Rust fixture under `tests/fixtures/adopt/` and tests of the survey's
  detection, commands, toolchain and aggregation, and MUST keep every existing ecosystem's detection unchanged.
- **FR-009**: The change MUST add a changelog fragment claiming MINOR, marked experimental as the adoption path
  is, and `VERSION` MUST carry the number that claim requires.

### Key Entities

- **Candidate**: one buildable directory the survey proposes — ecosystem, language, evidence file, the eight
  proposed commands, the toolchain — which the maintainer confirms or overrides into `project.json`.
- **Toolchain pin**: the Rust version a repository asks for, read from its toolchain file; empty where absent.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Adopting a cloned single-crate Rust repository proposes exactly one candidate with all eight targets
  answered or explicitly unanswered — zero targets left blank or guessed.
- **SC-002**: Adopting a Rust workspace with N members proposes exactly one candidate, not N+1.
- **SC-003**: `make -f delivery/Makefile verify` passes in the adopted Rust fixture on a machine with the pinned
  toolchain installed.
- **SC-004**: Every existing adoption fixture surveys exactly as it did before the change.

## Assumptions

- Ecosystem order: Cargo is tried after the existing nine, so an existing mixed-language directory keeps the
  answer it has today. (Guess: a directory with `Cargo.toml` beside `package.json` stays Node; overridable.)
- cargo-mutants' configuration is `.cargo/mutants.toml`; its presence is what "when configured" means.
- The survey reads `Cargo.toml` by text match for `[workspace]` / `[package]`, not with a TOML parser, matching
  how other rows read their manifests.
- Commands are proposals: the maintainer confirms or overrides each, so a proposal that does not fit a given
  repository costs one override, not a failed adoption.
- Adoption remains experimental per `AGENTS.md`; this adds a new recognisable ecosystem and changes no existing
  answer, so it is a MINOR.
- Out of scope: generating Rust services (already covered by the factory), axis answers for Rust (issue #11), and
  execution-model delegation (issue #9).
