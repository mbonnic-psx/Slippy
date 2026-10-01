# Implementation Plan: optional-tools — audit and mutation where the crate configures them

**Branch**: `001-rust-cargo-adopt-optional-tools` (claimed as `slice/optional-tools`, D17) | **Date**: 2026-10-01 |
**Spec**: [spec.md](../../spec.md) · slice 2 of [story-split.md](../../story-split.md)

**Input**: US1 scenarios 3 and 4 (the configured halves); FR-003; the slice's gaps OG1–OG7; decisions D18 (audit is
`cargo deny check advisories`) and D20 (which files count, `--workspace`, no guard). What the two tools read and write
is in [research.md](research.md), every row observed.

## Summary

A maintainer adopting a Cargo repository whose candidate directory carries a cargo-deny configuration is offered
`cargo deny check advisories` as audit, and one whose candidate directory carries `.cargo/mutants.toml` is offered
`cargo mutants` as mutation — each with `--workspace` at a workspace root and prefixed once in a subdirectory. Where
neither file is there, both stay a written no, exactly as `single-crate` left them. The adopted `.gitignore` block
gains `mutants.out/` and `mutants.out.old/` where a Cargo application is recorded, because that is what
`cargo mutants` writes beside the crate. No other ecosystem's detection, commands or ignore lines change.

## Release constraint

**Held behind the pre-release, as standing decision D7 says for this feature.** A merge to `main` publishes
`1.4.0.dev<N>`, which installers pass over unless asked; only a person's `make release` makes it a release, and a
person merges the pull request. No flag is opened. Nothing here reaches a maintainer whose repository has no
`Cargo.toml`: every other tree surveys, and every other adoption's ignore block reads, as before (SC-004, pinned below).

## Technical Context

**Language/Version**: Python ≥ 3.11 (`requires-python`); run here on 3.14.

**Primary Dependencies**: none added — the standard library; the triggers are `Path.is_file()` checks in the
candidate's directory, as `rows.py` reads its own trigger files (`requirements.txt`, the Maven plugin).

**Storage**: files — the surveyed tree is read; `project.json` and `.gitignore` in the adopted repository are written
by the existing `adopt` path.

**Testing**: `python3 -m pytest` / `make test TESTS=…` — a new suite `tests/test_survey_cargo_tools.py` at the survey's
boundary (`survey` over a tree on disk) and at the adopt boundary (`slipwai adopt --yes` in a throwaway repository,
through `tests/test_adopt.py`'s `repository` / `slipwai` helpers). The committed fixtures `rust-crate` and
`rust-workspace` are **not** changed (OG7), so `make test-adoption` keeps proving the no-answer half end to end.
Every run under `systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`, `TMPDIR` on disk, with
`CRUISE_RUNNER` and `CRUISE_ITERATION` unset (D8).

**Target Platform / Project Type**: the `slipwai` CLI, wherever it runs.

**Performance Goals / Constraints / Scale**: at most four `stat` calls per Cargo candidate; no file is read.

## Constitution Check

- **I. What a project was given keeps meaning what it meant** — holds. The nine other ecosystems' rows are untouched;
  the ignore line is keyed on a wrapped application whose toolchain records `ecosystem: cargo`, so a generated
  project's `.gitignore` (whose apps are all generated) and every non-Cargo adoption's block are byte-identical
  (pinned below). No release carried the Cargo row, so the new answers change no released one; the fragment
  `changelog.d/rust-cargo-adopt.md` is amended (still `MINOR`, still experimental), `VERSION` stays `1.4.0.dev0`.
- **II. Re-running is safe** — holds. No new writing command. `adopt --refresh` proposes the new commands to a snapshot
  adoption the way WG9 reconciles any detected command (OG6); the ignore block is written once, by `adopt`, as today.
- **III. Simplicity** — holds. Two constants and one small function in `cargo.py`; one keyed table and one line in
  `build_artifacts`.
- **V (as it holds today)** — every new test enters at the survey's or adopt's boundary; fakes only, no mock.
- **VIII** — no new value in `project.json`; the commands are strings the record already carries.
- **IX (dependency scanning, an obligation once an audit is recorded, D3)** — the proposal is a vulnerability scan
  (D18), so a recorded audit is the same kind of claim every other ecosystem's is; `platform_row` lifts the Platform
  row to `audited` by the shared rule (research).
- IV, VI, VII, X, XI — not touched.

## Pin

The Cargo row was written under the method and its tests are its pin (`tests/test_survey_cargo*.py`). The slice also
changes `project/gitignore.build_artifacts`, which was here before the method (initial release, 8ac6145):

1. **An adopted repository's ignore block carries no language line for a wrapped application** — observed and pinned
   by T001 before any change: a Node repository adopted with `--yes` has a block with neither `node_modules/` nor
   `mutants.out/` added by the factory, and a Cargo crate adopted with `--yes` has no `mutants.out/` (the case this
   slice changes on purpose, so that one assertion is inverted at T004). `build_artifacts` for a generated Rust
   service is already pinned by `tests/test_mutation.py`.

`delivery/survey/pinned.md` is not this slice's to write (the shared-surface rule). The row to append there — dated
2026-10-01, seam `project/gitignore.build_artifacts` through `slipwai adopt`, tests `tests/test_survey_cargo_tools.py`,
run with `make test TESTS=test_survey_cargo_tools` — is handed back to the delegating session.

## Design

`src/slipwai/ecosystems/cargo.py` — kept to its own small block, so the sibling `toolchain-pin` (which changes the
toolchain line) merges mechanically:

| Name | What |
|---|---|
| `DENY = ("deny.toml", ".deny.toml", ".cargo/deny.toml")` | what cargo-deny reads in the directory it runs in (OG1, research) |
| `MUTANTS = ".cargo/mutants.toml"` | what cargo-mutants reads at the directory it runs in, the workspace root for a workspace (OG3, research) |
| `optional_tools(root, directory, flag) -> dict[str, str \| None]` | `{"audit": in_dir(directory, f"cargo deny{flag} check advisories") or None, "mutation": in_dir(directory, f"cargo mutants{flag}") or None}` — each present only where its file is a regular file in `root / directory`; `flag` is the row's existing `" --workspace"` or `""` |
| `cargo(root, directory)` | as today, `complete(…, **optional_tools(root, directory, flag))` |

No `command -v` guard and no Make variable (D20, OG4): a missing tool exits 101 with `no such command`.

`src/slipwai/project/gitignore.py`:

| Name | What |
|---|---|
| `WRAPPED_ARTIFACTS = {"cargo": "mutants.out/\nmutants.out.old/\n"}` | what a wrapped application's own optional tool writes beside it when the delivery material runs it, keyed by the toolchain's `ecosystem` |
| `build_artifacts(...)` | appends `WRAPPED_ARTIFACTS[ecosystem]` once per ecosystem across `wrapped_of(apps)`, in first-appearance order; a generated project has no wrapped app, so its `.gitignore` is unchanged |

Unanchored lines, so a crate in a subdirectory is covered. Keyed on a recorded Cargo application, not on a recorded
mutation command, so a maintainer who adds `mutation` later needs no second edit (OG5).

## Project Structure

### Documentation (this slice)

```text
specs/001-rust-cargo-adopt/slices/optional-tools/
├── plan.md        # this file
├── research.md    # what cargo-deny and cargo-mutants read and write, observed
├── tasks.md       # drive-tasks
└── benchmark.json
```

### Source Code

```text
src/slipwai/ecosystems/cargo.py        # DENY, MUTANTS, optional_tools, two targets in the row
src/slipwai/project/gitignore.py       # WRAPPED_ARTIFACTS, one line in build_artifacts
tests/test_survey_cargo_tools.py       # new suite: OG1–OG5 at the survey and adopt boundaries
tests/test_survey_cargo.py             # unchanged; its no-answer assertions are the unconfigured half
changelog.d/rust-cargo-adopt.md        # amended: audit and mutation when configured; the ignore lines; Catch-up
docs/adopting.md                       # one sentence: the two trigger files for a Cargo candidate
```

**Structure Decision**: the one deployable, `slipwai`, at the root — the only service `project.json` records, and its
purpose covers the survey and adopt. One vocabulary (candidate, target, audit, mutation, configuration file), so one
bounded context; strategy `leave-it` (D2), so the change lands in the existing modules. A new suite rather than
growing `tests/test_survey_cargo.py`, which the sibling `toolchain-pin` edits, so the two merge without touching the
same lines; it stays inside the 350-line budget (`scripts/check-structure.py`).

## Not working yet (deliberate, owned elsewhere)

- A `deny.toml` above the candidate (cargo-deny honours it) proposes nothing (D20's reversal; spec *Out of scope*).
- A pure Rust repository's Platform row stays `unknown` even with an audit recorded: Rust has no `support.json`
  product to date, which `toolchain-pin` puts out of scope; the `audited` rung needs `supported` first.
- A repository adopted before this slice keeps its ignore block as written: `migrate` and `adopt --refresh` never
  rewrite it; the Catch-up names the two lines to add by hand.
- No offline advisories run and no `--in-diff` mutation scope (spec *Out of scope*).
- The toolchain version is always empty — `toolchain-pin`; the adopted CI sets up no Rust — `ci-toolchain`.
- `story-split.md` row 2 still names `cargo deny check`; D18 superseded it, and the split is not this slice's to
  reword (it is the feature's, not in this slice's manifest).
- `make -f delivery/Makefile smoke` still says none is recorded until a person regenerates the targets (D6).

## Complexity Tracking

None.
