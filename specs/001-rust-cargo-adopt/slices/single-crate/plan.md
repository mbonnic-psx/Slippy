# Implementation Plan: single-crate — adopt proposes a one-crate Cargo repository

**Branch**: `001-rust-cargo-adopt` | **Date**: 2026-09-28 | **Spec**: [spec.md](../../spec.md) · slice 1 of
[story-split.md](../../story-split.md)

**Input**: US1 scenarios 1, 2, 5 and the no-answer halves of 3 and 4; FR-001, FR-002, FR-004, FR-008, FR-009;
SC-001, SC-004; the edge cases (skipped directories, mixed-manifest directory, malformed `Cargo.toml`, no
`Cargo.lock`); the slice's gaps SG1–SG6.

## Summary

A maintainer runs `slipwai adopt` on a repository holding one Cargo crate and is offered one Rust / Cargo
candidate with `Cargo.toml` as its evidence and an answer — a command or a written no — for all eight targets.
The survey's ecosystem table (`src/slipwai/ecosystems.py`) gains one row, `cargo`, tried after the nine that are
there; nothing else in the survey changes shape, because every consumer reads a `Detected` and not an ecosystem
name.

## Release constraint

**Held behind the toggle this repository already has: the pre-release.** Merging to `main` publishes
`1.4.0.dev<N>`, a `.dev` pre-release every installer passes over unless asked, and only a person's
`make release` turns it into a version anyone gets by default (`AGENTS.md`, *Versioning*; D7). The adoption path
is experimental, and the fragment says so. Nothing this slice adds can surprise a maintainer who did not ask for
it: a repository that has no `Cargo.toml` surveys exactly as before (SC-004). A person merges the pull request
(owner brief); the run ends the slice at an open PR.

## Technical Context

**Language/Version**: Python ≥ 3.11 (`requires-python`); run here on 3.14.4.

**Primary Dependencies**: none added — the standard library, as every ecosystem row is written.

**Storage**: files — the surveyed tree is read; `project.json` in the adopted repository is written by the
existing `adopt` path.

**Testing**: `python3 -m pytest` (`tests/test_survey.py` for detection and commands, `tests/test_adopt.py` for
the adopted record); `make test-adoption` for the end-to-end fixture path (SG6), which runs `verify` where
`cargo` is on the machine (it is here: `~/.cargo/bin/cargo`) and skips with a reason where it is not.

**Target Platform**: wherever `slipwai` runs — Linux, macOS, WSL.

**Project Type**: CLI.

**Performance Goals / Constraints / Scale**: none new; one more `is_file()` per surveyed directory.

## Constitution Check

- **I. What a project was given keeps meaning what it meant** — holds. No existing ecosystem's detection or
  commands change: `cargo` is last in `ECOSYSTEMS`, so a directory with another manifest beside `Cargo.toml` keeps
  the answer it has today (SC-004, pinned below). The change is user-visible under `src/slipwai/`, so it carries
  `changelog.d/rust-cargo-adopt.md` claiming MINOR, marked experimental; `VERSION` already reads `1.4.0.dev0`, the
  MINOR that claim requires over `1.3.0`, so it is not raised again.
- **II. Re-running is safe** — holds. The slice adds no command that writes; SG6 proves `adopt --refresh` on the
  adopted Rust fixture changes nothing, through the harness every fixture goes through.
- **III. Simplicity** — holds. One function in the table beside nine of the same shape; no abstraction added.
- **IV–VII** — a target or not touched: no domain layer is introduced (`leave-it`, D2), no third party is called,
  no long-running process.
- **V (target)** — what holds today: the acceptance scenarios enter at the command's boundary, a fixture
  repository surveyed (`buildable` / `survey` over a tree, and `adopt` end to end in the fixture harness); no mock.
- **VIII** — additive: a new value `rust` for a toolchain `kind` and `cargo` for `ecosystem` in `project.json`;
  readers tolerate unknown values (`adopted_ci.SETUP` skips a kind it has no row for — SG2).
- **IX** — nothing read or written is a secret or personal data.

## Pin

The slice changes code that was here: `ecosystems.py` (the table) and, through it, `survey.buildable`. Pinned
before the change in `delivery/survey/pinned.md`:

1. **The nine existing ecosystems survey as they do today** — `tests/test_survey.py` (every ecosystem row, mixed
   directories, ownership) and `make test-adoption` (seven fixtures adopted end to end). Both exist and are the pin.
2. **A directory holding only a `Cargo.toml` is surveyed as nothing** — observed 2026-09-28: `buildable` over a
   tree whose root holds one `[package]` `Cargo.toml` returns `()`. This is the behaviour the slice changes on
   purpose; its first RED test is this observation inverted.

## Design

`ecosystems.cargo(root, directory) -> Detected | None`:

| Field | Value |
|---|---|
| found when | `Cargo.toml` is a file in the directory — by file name alone, so a malformed manifest is still Cargo (edge case) |
| `ecosystem` / `language` | `cargo` / `rust` |
| `evidence` | `prefixed(directory, "Cargo.toml")` |
| install | `in_dir(directory, "cargo fetch --locked")` — whether or not `Cargo.lock` exists (edge case) |
| typecheck | `in_dir(directory, "cargo check --all-targets")` |
| lint | `in_dir(directory, "cargo clippy --all-targets -- -D warnings && cargo fmt --check")` — one prefix for both halves (SG3) |
| test | `in_dir(directory, "cargo test")` |
| integration, adversarial | `None` (FR-002) |
| audit, mutation | `None` in this slice; `optional-tools` proposes them where configured |
| `toolchain` | `{"kind": "rust", "version": ""}` whatever the tree pins (SG1); `toolchain-pin` reads it |
| `packaging` | `None` |

Appended to `ECOSYSTEMS` after `ruby`. `aggregates` is unchanged, so a single crate owns nothing below it (SG5).
`role_of` is unchanged (SG4). `adopted_ci.SETUP` is unchanged (SG2). The docstring on `Detected.toolchain` gains
`rust` in its list of kinds.

## Project Structure

### Documentation (this slice)

```text
specs/001-rust-cargo-adopt/slices/single-crate/
├── plan.md        # this file
├── research.md    # what Cargo's commands do, cited
└── tasks.md       # drive-tasks
```

### Source Code

```text
src/slipwai/ecosystems/                    # was ecosystems.py; split at T004 (see below)
├── __init__.py                            # the table: ECOSYSTEMS, and the names callers import, unchanged
├── common.py                              # TARGETS, EXTRA, Detected, and the helpers every row writes with
├── rows.py                                # the nine rows that were here, moved unchanged, and `aggregates`
└── cargo.py                               # the cargo row; the later Cargo slices grow this module
tests/test_survey_cargo.py                 # detection, commands, subdirectory, mixed, malformed, skipped
tests/fixtures/adopt/rust-crate/           # Cargo.toml, Cargo.lock, src/lib.rs with one test, README.md
scripts/test-adoption.py                   # ADOPTIONS["rust-crate"] = ([], "cargo")
changelog.d/rust-cargo-adopt.md            # MINOR, experimental
```

**Structure Decision**: the one deployable, `slipwai`, at the root — the only service `project.json` records, and
its purpose covers the survey. One vocabulary (the survey's: candidate, ecosystem, target, toolchain), so one
bounded context; the strategy is `leave-it` (D2, ADR 0002 at `Proposed`), so the row lands in the existing
module beside its nine siblings.

**Why a package (found at T004).** `scripts/check-structure.py` holds every module and suite to 350 lines;
`ecosystems.py` stood at exactly 350 before this slice and the row took it to 365, and `test_survey.py` to 386.
The gate is not changed. A package keeps every submodule in the `contract` tier by prefix, so `TIERS` needs no
edit, every `from .ecosystems import …` still resolves, and the Cargo slices after this one grow `cargo.py` and
`test_survey_cargo.py` rather than the two files at their budget. The nine rows moved without a change.

## Not working yet (deliberate, owned by later slices)

- Audit and mutation are always a written no — `optional-tools`.
- The toolchain version is always empty — `toolchain-pin`.
- The adopted gate's CI sets up no Rust — `ci-toolchain`.
- A workspace's members are proposed as candidates of their own — `workspace`.
- `make -f delivery/Makefile smoke` in this repository still says none is recorded until a person regenerates the
  targets (D6).

## Complexity Tracking

None.
