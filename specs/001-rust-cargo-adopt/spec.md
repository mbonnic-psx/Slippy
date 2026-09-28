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
   typecheck is `cargo check --all-targets`, lint is `cargo clippy --all-targets -- -D warnings` followed by
   `cargo fmt --check`, test is `cargo test`, and integration and adversarial are recorded as no answer.
3. **Given** a crate with no `deny.toml`, **When** commands are proposed, **Then** audit is recorded as no answer;
   **Given** a `deny.toml` beside the manifest, **Then** audit is `cargo deny check`.
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
- A workspace member that is itself a nested workspace root is not proposed separately from the outer root.
- No `Cargo.lock`: install stays `cargo fetch --locked`, which fails loudly rather than resolving silently; the
  maintainer may override it when confirming.

### Slice `single-crate` — Gaps reviewed (2026-09-28, iteration 2)

Checked against `src/slipwai/ecosystems.py`, `survey.py`, `delivery_facts.role_of`, `project/adopted_ci.py`,
`platform.py`, `programme.py`, `structure.py` and `scripts/test-adoption.py`. The criteria and states this review
added, each owned by this slice:

- **SG1 — the toolchain before slice 3.** A crate adopted under this slice records its toolchain as kind `rust`
  with an empty version, whatever toolchain file it carries; `toolchain-pin` is what reads the pin. The survey
  page therefore shows no version for it, the way it shows none for any absent pin.
- **SG2 — no Rust setup in the adopted CI yet.** Until `ci-toolchain`, the gate's workflow writes no setup step for
  kind `rust` (`adopted_ci.SETUP` has no row, and a kind it does not know is skipped, as today). This is a
  deliberate hole, shown under *Not working yet*, and not a fault.
- **SG3 — the exact lint command.** At the root, lint is `cargo clippy --all-targets -- -D warnings && cargo fmt
  --check`; in a subdirectory `crates/ledger` every command is prefixed once, e.g.
  `cd crates/ledger && cargo clippy --all-targets -- -D warnings && cargo fmt --check`, as other ecosystems
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

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The survey MUST recognise a directory holding a `Cargo.toml` as a Rust candidate built by Cargo,
  with `Cargo.toml` (prefixed by its directory) as the evidence.
- **FR-002**: The survey MUST propose, for every one of the eight targets, either the Cargo command listed in
  User Story 1 or a written no-answer; it MUST NOT propose a command for integration or adversarial.
- **FR-003**: Audit MUST be proposed only where a `deny.toml` sits beside the manifest, and mutation only where
  cargo-mutants' configuration file is present; otherwise each is a written no-answer.
- **FR-004**: A candidate outside the root MUST have every command run in its own directory, as other
  ecosystems' commands are.
- **FR-005**: The survey MUST record the toolchain as kind Rust with the version the repository pins in
  `rust-toolchain.toml` (its `channel`) or `rust-toolchain`, or an empty version where it pins none.
- **FR-006**: A `Cargo.toml` declaring a `[workspace]` MUST aggregate the crates below it, so that no member is
  proposed as a candidate of its own.
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
