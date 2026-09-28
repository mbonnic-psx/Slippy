# Decisions — 001-rust-cargo-adopt

## D1 — Ground: the map rows this feature touches are still the survey's reading; proceed on them?
- **Stage:** ground · **Slice:** feature · **When:** 2026-09-28T20:51:00Z · **Iteration:** 1
- **Question:** Safety net (`tests-exist`), Structure (`named`), Path to production (`pipeline`) and Strategy (`recommended`) are `detected`, not a person's answer; the feature changes the survey under `src/slipwai/` and touches all four. Place them, or proceed?
- **Options:** ask a person and park · proceed on the detected values as stated assumptions, rows left as they are (stop table row 13, recommended)
- **Decision:** Proceed on the detected values as stated assumptions: the suite recorded for `slipwai` is the safety net (`make verify`), the one application is the `slipwai` tool at the root, and a change reaches users through the existing publish pipeline on a `v*` tag. No row is moved or marked `confirmed`.
- **Why:** A fact about the world is not a decision; the rows stay the tree's reading until the owner places them, and the work is additive inside the tool the map already names.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** the owner places a row differently with `/ground` — for example says the suite is not green on trunk
- **Written to:** specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D2 — Change strategy: which strategy do slices land under?
- **Stage:** ground · **Slice:** feature · **When:** 2026-09-28T20:52:00Z · **Iteration:** 1
- **Question:** The map recommends `leave-it` and no ADR records a decision; the word `Accepted` is a person's.
- **Options:** leave-it (recommended by the survey) · in-place · modular-monolith · strangler-fig · rewrite
- **Decision:** Proceed on `leave-it`, drafted as ADR 0002 at `Proposed`; slices land in the existing `slipwai` package.
- **Why:** The trigger is two additive capabilities inside the one tool; nothing in it asks for a new home.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** the owner rejects ADR 0002 or names a structural trigger
- **Written to:** delivery/docs/adr/0002-change-strategy-leave-it.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D3 — Principles: the constitution is still the template; ratify one drafted from the repo's own rules, or park?
- **Stage:** principles · **Slice:** feature · **When:** 2026-09-28T20:54:15Z · **Iteration:** 1
- **Question:** `.specify/memory/constitution.md` is still the template `./delivery/init` installed, and `specs/` already has a feature, so `check-constitution` fails and every later phase would plan against nothing (stop table row 2, `constitution: ratify`).
- **Options:** ratify a constitution drafted from the repo's own rules (recommended by the stop table) · park for a person
- **Decision:** Ratify version 1.0.0, drafted from AGENTS.md, docs/maintaining.md, the convergence map and D1/D2, and marked `ratified by cruise (skipper) — pending human review`. The domain invariant (Principle I) is that an answer a project already gave keeps its meaning, a released version is spent and migration is exact; Principle II says re-running a command is safe. Trunk-based integration is written in force, because the Integration row is `trunk` and `confirmed`. The seven principles whose axis is below its rung (one-path, build-once, fast-feedback, acceptance-driven, hexagonal, ubiquitous-language, strict-typing) are targets, each with its marker, what holds today and the next rung, and none is planned. Nothing the repo lacks is set: no review SLA, no deprecation window, no flag lifetime, no rollback or detection target. Dependency scanning is written as an obligation that starts once an audit command is recorded, not as one in force.
- **Why:** AGENTS.md says the factory is held to the standard it generates, so the floor is this repo's own declared rule and not an invention; what it adds is how the floor applies to a Python CLI with no service, no money and no personal data. Parking would leave feature 001 unplannable over a document whose content the repo already states in its own rules. Whoever runs `slipwai generate` or adopts a repository with it is protected by exactly the release and migration rules Principle I encodes.
- **Decided by:** drive-skipper (claude-opus-5-5[1m])
- **Confidence:** medium · **Would reverse if:** the person reviewing it rejects Principle I as this repository's domain invariant
- **Written to:** .specify/memory/constitution.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D4 — Split: which slice first, and in what order do the rest follow?
- **Stage:** split · **Slice:** feature · **When:** 2026-09-28T21:10:00Z · **Iteration:** 1
- **Question:** The specification's three stories are P1–P3; how are they cut into vertical slices, and which is the walking skeleton?
- **Options:** one slice per user story · five slices by rule and data variation, single crate first (recommended) · workspace first, since most real repositories are workspaces
- **Decision:** Five slices — `single-crate`, then `optional-tools`, `toolchain-pin` and `workspace` together, then `ci-toolchain` after `toolchain-pin` — as `story-split.md` records.
- **Why:** The specification ranks the single crate P1 as the whole value for the first user; each later slice is one variation a maintainer can see on its own, and three of them can run beside each other against the Cargo row slice 1 leaves.
- **Decided by:** host (stage recommendation)
- **Confidence:** medium · **Would reverse if:** the first user's repository is a workspace, which would pull `workspace` up to second
- **Written to:** specs/001-rust-cargo-adopt/story-split.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D5 — Programme: the thirteen quick wins the survey lists — are any of them a slice ahead of this feature?
- **Stage:** convergence programme (read at pin) · **Slice:** single-crate · **When:** 2026-09-28T21:40:00Z · **Iteration:** 2
- **Question:** `delivery/docs/change-strategy.md` opens its programme with four "connection string with a password" findings and nine "no lockfile" findings; a secret in the tree goes ahead of every product slice (stop table row 15).
- **Options:** open a quick-win slice per finding · record them as false positives read off the tree, no slice (recommended)
- **Decision:** No slice. The four "secrets" are documentation and test placeholders (`postgres://user:pass@host:port/db` in a Javadoc at `assets/backing-services/java/database_url.java:10`, `app:secret@localhost` in its unit test, `someone:a-token@git.example` in `tests/test_release.py:295` and `tests/test_upgrade.py:187`); the nine lockfile findings are templates under `assets/` that the factory copies into a generated project, where the project's own install writes the lockfile.
- **Why:** Each line was read, and none names a host anyone can reach; opening a rotation slice for a string that authenticates to nothing would spend the run on nothing a maintainer adopting a Cargo repository sees.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** any of those credentials is found to work against a real system, or the owner wants the survey itself to stop reporting factory templates (a factory change of its own)
- **Written to:** specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D6 — Pin: the application has no proven run path and no `smoke`; how does the slice get past the refusal?
- **Stage:** pin · **Slice:** single-crate · **When:** 2026-09-28T22:15:00Z · **Iteration:** 2
- **Question:** `delivery/survey/running.md` read *Not yet proven* for `.` and `project.json` had no `commands.smoke`, so Pin refuses every slice that changes code that was here until the run path is proven — and the generated `smoke` target lives in `delivery/Makefile`, a control file a run may not change.
- **Options:** park for a person to prove it · prove the run path in the tree and record `commands.smoke`, leaving the generated target for a person to regenerate (recommended by the bosun's standing moves)
- **Decision:** Proved and recorded: `./slipwai --version && ./slipwai adopt --next`, from the root, exits 0 and leaves `git status` unchanged (re-run by the host on 2026-09-28 after the bosun's session was cut off); written into `delivery/survey/running.md` and `project.json` `deployables.slipwai.commands.smoke`. `delivery/Makefile`'s `smoke` target was not regenerated and still says none is recorded.
- **Why:** The rule is that the application starts before its behaviour is pinned; that is a fact of the tree, provable here with the checkout's own launcher, and nobody has to supply anything for it. Regenerating the Makefile is a control-file change, which only a person makes.
- **Decided by:** drive-bosun
- **Confidence:** high · **Would reverse if:** a person regenerates the targets (`./slipwai adopt --refresh`) and `make -f delivery/Makefile smoke` fails, or wants a different smoke — a task in the next slice is to confirm `make -f delivery/Makefile smoke` runs the recorded command once regenerated
- **Written to:** delivery/survey/running.md, project.json, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D7 — Release constraint: how does single-crate reach users, and what gates it?
- **Stage:** release constraint · **Slice:** single-crate · **When:** 2026-09-28T22:30:00Z · **Iteration:** 2
- **Question:** `release: flagged` asks every slice to land dark behind a flag seeded off, and `delivery/docs/deployment.md` says no flag mechanism is installed here; how is this slice held?
- **Options:** releasable on merge · held behind the toggle this repository already has — the `.dev` pre-release that only `make release` turns into a release (recommended) · a coordinated deploy
- **Decision:** Held behind the pre-release. A merge to `main` publishes `1.4.0.dev<N>`, which installers pass over unless asked; only a person's `make release` makes it a release, and a person merges the PR. No flag file is opened.
- **Why:** That is the repository's own dark-launch: a maintainer who did not opt into snapshots never sees the change, and the adoption path it extends is labelled experimental. A code flag around one row of a recognition table would add a switch with nothing to switch between.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** the owner wants snapshots treated as released, so that a merge needs a flag of its own
- **Written to:** specs/001-rust-cargo-adopt/slices/single-crate/plan.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D8 — Implementation pre-flight: `make verify` is red in the runner's session for seven cruise tests; whose red is it?
- **Stage:** implementation · **Slice:** single-crate · **When:** 2026-09-28T23:40:00Z · **Iteration:** 2
- **Question:** The slice starts from a green `make verify`; inside this iteration it ends `FAILED (failures=6, errors=1)`, every one in `test_cruise_start`, `test_cruise_watch`, `test_cruise_where`, `test_cruise_index` and `test_benchmark_brackets` — tests that spawn `cruise.py` and read `CRUISE_RUNNER` / `CRUISE_ITERATION`, which the runner sets on this session.
- **Options:** park on the gate's own output · treat the tree as green where those five modules pass with the runner's two variables unset, and run every gate this iteration that way (recommended) · change the tests to clear the variables
- **Decision:** The tree is green: lint, typecheck and check-structure passed, and the five modules pass (16 tests, OK) under `env -u CRUISE_RUNNER -u CRUISE_ITERATION`. Every `make verify` in this run is invoked with those two variables unset. The tests are not changed in this slice.
- **Why:** The failures are this session's environment reaching into a subprocess, not anything in the tree a maintainer would get; the gate itself is untouched, and the same command a person runs is what passes. Making those tests isolate their environment is a change to the method's own test tree, outside a Cargo slice.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** any of the seven fails with the variables unset, or a person wants the suite to clear them itself — a task for a later slice
- **Written to:** specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D9 — Converge: should the survey read database drivers out of `Cargo.toml` in this feature?
- **Stage:** convergence · **Slice:** single-crate · **When:** 2026-09-29T01:35:00Z · **Iteration:** 2
- **Question:** `survey.DEPENDENCY_MANIFESTS` does not list `Cargo.toml`, so a driver a crate declares (`sqlx`, `diesel`, `tokio-postgres`) is not reported on the database axis; a task here, or issue #11?
- **Options:** a task in this slice · left to issue #11 with the other Rust axis answers (the specification's out-of-scope line)
- **Decision:** Left to issue #11. The survey still proposes the candidate and its commands, which is what this feature promises; which database a crate talks to is an axis answer, and the Assumptions and the slice's gaps review put every Rust axis answer in #11.
- **Why:** The specification answers it outright (Assumptions: axis answers for Rust are issue #11), which is the recommendation taken. A maintainer adopting a crate gets its build and gate from this feature; the database row reads `none` for them exactly as it does for any ecosystem whose drivers the survey does not read yet, and they can place it with `/ground`.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** the owner wants the database axis answered for Rust before #11
- **Written to:** specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D10 — Gaps after converge: the proposed lint lets new clippy findings through the ratchet; what does lint propose?
- **Stage:** gaps (after converge) · **Slice:** single-crate · **When:** 2026-09-29T01:55:00Z · **Iteration:** 2
- **Question:** The adopted gate's ratchet keys a finding by a line naming a file at a position; clippy's default output puts the position on its own ` --> file:line:col` line, so every finding in a file is one key and a baselined crate passes with any number of new warnings (HIGH). SG3 and US1 scenario 2 fix the command's text.
- **Options:** keep SG3 as written and record the hole · add `--message-format=short` to the clippy half, one line per finding (recommended by the gaps pass) · split lint and format, or run both with `;` keeping the worst exit
- **Decision:** Lint is `cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt --check`, prefixed once as SG3 says. The `&&` stays: it is the order-and-chain the factory's own generated Rust gate uses (`src/slipwai/project/languages/rust.py:120`), and a red clippy that hides fmt drift until it is fixed is the same trade every combined lint makes. A clippy that is absent (exit 101, not 127) and the Catch-up wording are Phase 4 tasks.
- **Why:** A maintainer who adopts a crate with existing warnings is promised that the gate only lets the count go down; with the default format that promise is false for every file that already has one warning. The short format keeps every other property of the command.
- **Decided by:** host (stage recommendation)
- **Confidence:** medium · **Would reverse if:** the owner wants lint and format as separate proposals, which would change the eight-target shape the survey records
- **Written to:** specs/001-rust-cargo-adopt/spec.md, specs/001-rust-cargo-adopt/slices/single-crate/plan.md, specs/001-rust-cargo-adopt/slices/single-crate/research.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing
