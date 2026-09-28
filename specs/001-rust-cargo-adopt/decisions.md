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
