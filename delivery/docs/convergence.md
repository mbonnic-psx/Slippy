<!-- convergence: 7099d7d5f26bd32d -->
# Where `slipwai` stands

> **Experimental.** Brownfield adoption is new and will change shape while real repositories teach it what it got wrong: the files under the delivery directory, the facts `project.json` records and the questions `adopt` asks may change in a MINOR release, and `slipwai migrate` brings each change here with a note saying what to do. Every place this reaches you says so until it stops being true. What surprised you — a detection that was wrong, a gate that went red, a sentence this page should have had — belongs on the public issue tracker.

A generated project starts at the top of every ladder below and the method keeps it there. This repository
started wherever it was; this page says where that is, axis by axis, and the loop climbs one rung per slice
until nothing here differs from a generated project — at which point `slipwai converge` makes it one. **1** of
9 axes are at their target, **8** below it, **0** unrecorded.

Every row is a fact `project.json` holds under `convergence`, with where it came from: `detected` from the tree,
`confirmed` or `overridden` by a person, `unrecorded` where nothing has said. Nothing is a default. To move a
row, establish the rung — a slice, a decision, a pinned seam — then say so in `project.json`; `/survey`
(`slipwai adopt --refresh`) re-reads the tree and regenerates this page, and `make -f delivery/Makefile check-convergence`
fails a row the tree contradicts or a page that no longer matches the record. `/drive` reads this before the
first slice and offers the next unplanned row as a method slice beside the product's.

| Axis | Where it stands | Target | Evidence | Planned as | Provenance |
|---|---|---|---|---|---|
| Path to production | `pipeline` | `pipeline-decides` | pipeline: .github/workflows/package.yml; pipeline: .github/workflows/publish-package.yml; pipeline: .github/workflows/release.yml; pipeline: .github/workflows/verify.yml; scripted: assets/targets/aws/scripts/deploy.py; scripted: assets/targets/azure/scripts/deploy.py; scripted: tests/fixtures/adopt/converging/deploy.sh; scripted: tests/fixtures/adopt/javascript-gitlab/deploy.sh; scripted: Makefile | *not yet* | `detected` |
| Integration | `trunk` | `continuous` | short-lived PR branches (most merged within an hour, 2026-09-23..28); CI does not run on the fork: zero workflow runs on mbonnic-psx/Slippy, no checks on PRs #8 and #12 (2026-09-28) | *not yet* | `confirmed` |
| Safety net | `tests-exist` | `mutation-measured` | test recorded for slipwai | *not yet* | `detected` |
| Structure | `named` | `typed` | slipwai: tool; not under apps/: . | *not yet* | `detected` |
| Platform | `supported` | `audited` | in support on 2026-09-28: Python 3.11; no audit command recorded for slipwai | *not yet* | `detected` |
| Constitution | `template` | `in-full` | .specify/memory/constitution.md | *not yet* | `detected` |
| Data | `recorded` | `settled` | schema: unmanaged | *not yet* | `detected` |
| Infrastructure | `settled` | `settled` | home: here | *not yet* | `detected` |
| Strategy | `recommended` | `done` | why: Run /cruise on the factory itself to build Rust support and execution-model delegation; recommended: leave-it | *not yet* | `detected` |

## The ladders

### Path to production

1. `unknown` — nobody has said how a change reaches production
2. `manual` — by hand: files copied, a console clicked, a package installed
3. `scripted` — somebody runs a script that deploys it
4. `pipeline` — a CI job deploys it, on some trigger
5. `one-path` — every change reaches production the same automated way, and no other
6. `pipeline-decides` — the pipeline decides releasability; artefacts immutable, rollback on demand, configuration shipped with the artefact ← a generated project sits here

### Integration

1. `unknown` — nobody has said how work is integrated
2. `branches` — long-lived branches, merged when a feature is done
3. `trunk` — trunk-based: every branch lives less than a day
4. `continuous` — integrated at least daily, the build stops on red, CI runs on every commit on this forge ← a generated project sits here

### Safety net

1. `none` — no test command is recorded for any application that was here
2. `tests-exist` — a test suite is recorded; whether it is green is not established
3. `tests-pass` — the suite is green in the gate, with no quarantine
4. `fast` — the suite is deterministic and fast enough to run on every change
5. `pinned` — the seams a slice touches are pinned by `/characterise` before they change
6. `mutation-measured` — the suite's strength is measured by mutation testing ← a generated project sits here

### Structure

1. `as-found` — at least one buildable directory's role — service, library, tool, tests — is not established
2. `named` — every application is recorded as what it is
3. `laid-out` — every application lives under `apps/`, the layout a generated project has — or at a Go module's root, which is its import path and lives where it stands
4. `hexagonal` — every application declares the hexagonal layers and the import gate holds it to them
5. `typed` — every application has a type check the gate runs ← a generated project sits here

### Platform

1. `unknown` — nothing an application runs on could be read from the tree: no runtime pin, no framework version, no image
2. `inventoried` — what the applications run on is read and dated; something is out of support, or not in the support table
3. `supported` — every runtime, framework and image read is within support on the day it was dated
4. `audited` — every runtime, framework and image is in support, and every application records an `audit` command the gate can run ← a generated project sits here

### Constitution

1. `template` — `./delivery/init` has not run, or the constitution is still the template it installed
2. `ratified` — a constitution is ratified for the rungs this repository actually stands on
3. `in-full` — the floor a generated project ratifies is in force here, and `check-constitution` holds it ← a generated project sits here

### Data

1. `open` — where the schema is versioned is not recorded
2. `recorded` — the schema's home is recorded: `here`, `elsewhere`, `unmanaged` or `none`
3. `settled` — the schema is versioned in one place — here, in a named repository, or there is none ← a generated project sits here

### Infrastructure

1. `open` — where the infrastructure is described is not recorded
2. `recorded` — the infrastructure's home is recorded: `here`, `elsewhere`, `unmanaged` or `none`
3. `settled` — the infrastructure is described in one place — here, in a named repository, or there is none ← a generated project sits here

### Strategy

1. `open` — why this work is happening is not recorded
2. `why-recorded` — the business trigger is recorded; no strategy is recommended yet
3. `recommended` — a strategy is recommended from the trigger and this map — *leave it* included
4. `decided` — the strategy is decided and recorded as an ADR
5. `done` — the programme is finished: every retirement-ledger row reads *removed*, or the decision was to leave it ← a generated project sits here
