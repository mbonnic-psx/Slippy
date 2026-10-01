# Implementation Plan: toolchain-pin — the Rust toolchain a repository pins is recorded

**Branch**: `001-rust-cargo-adopt-toolchain-pin` (claim `slice/toolchain-pin`, D17) | **Date**: 2026-10-01 |
**Spec**: [spec.md](../../spec.md) · slice 3 of [story-split.md](../../story-split.md)

**Input**: US2 scenarios 1–3; FR-005; the slice's gaps TG1–TG10 (SG1 replaced); decisions D19 (where the pin is
looked for, and the hand-over to `ci-toolchain`), D21 (which file wins, what is recorded, `rust-version` is not a
pin) and D22 (a multi-line legacy file is TOML, as rustup reads it). What rustup does is in
[research.md](research.md), every row observed.

## Summary

A maintainer adopting a Rust repository sees the toolchain it pins recorded in `project.json` and on the survey
page, read the way rustup reads it: from the candidate's directory up to the repository root, the nearest directory
holding a toolchain file deciding, `rust-toolchain` over `rust-toolchain.toml` within it, the channel recorded as
written — or an empty version, never an error, where nothing usable is pinned. One function changes, the Cargo row's
toolchain; the record's shape, every other ecosystem, the survey page and `adopt --refresh` are unchanged in code.

## Release constraint

**Held behind the pre-release, as standing decision D7 says for this feature.** A merge to `main` publishes
`1.4.0.dev<N>`, which installers pass over unless asked; only a person's `make release` makes it a release, and a
person merges the pull request. No flag is opened. Nothing here reaches a repository with no `Cargo.toml`: every other
tree surveys as before (SC-004).

## Technical Context

**Language/Version**: Python ≥ 3.11 (`requires-python`); run here on 3.14.

**Primary Dependencies**: none added — `tomllib` from the standard library (3.11+), and the survey's bounded `read`.

**Storage**: files — the surveyed tree is read; `project.json` in the adopted repository is written by the existing
`adopt` path.

**Testing**: `python3 -m pytest` — a new suite `tests/test_survey_cargo_toolchain.py` at the survey's boundary
(`survey` over a tree on disk; `slipwai adopt` and `adopt --refresh` in a throwaway repository through
`tests/test_adopt.py`'s `repository` / `slipwai` helpers); the one flipped assertion in `tests/test_survey_cargo.py`;
`scripts/test-adoption.py --only rust-crate` for the committed fixture end to end (TG7). Every run under
`systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`, `TMPDIR=$HOME/.cache/slippy-toolchain-pin-tmp`,
`env -u CRUISE_RUNNER -u CRUISE_ITERATION` (D8).

**Target Platform / Project Type**: the `slipwai` CLI, wherever it runs.

**Performance Goals / Constraints / Scale**: at most two `lexists` per directory between the candidate and the root
(the survey's depth bounds it), and one bounded read of the one file that decides.

## Constitution Check

- **I. What a project was given keeps meaning what it meant** — holds. No release carried the Cargo row (it is in
  `changelog.d/`, not `CHANGELOG.md`), so a version where there was an empty one changes no released answer; the
  fragment is amended (still `MINOR`, still experimental), `VERSION` stays `1.4.0.dev0`. Other ecosystems keep
  reading their own directory (D19, SC-004): the new helper is called by the Cargo row alone.
- **II. Re-running is safe** — holds. No new writing command; a refresh of an adopted repository with a pin it already
  recorded changes nothing (the fixture's `adopt --refresh` no-op, `make test-adoption`), and TG8's refresh of an
  empty detected version is the existing `reconciled_app` path, tested, not changed.
- **III. Simplicity** — holds. One function reading the pin, beside the row that calls it; no new field (D19).
- **V (as it holds today)** — every new test enters at the survey's boundary or through `slipwai adopt`; trees on
  disk, no mock.
- **VIII** — no new key in `project.json`; `toolchain.version` is a string it already carries.
- IV, VI, VII, IX–XI — not touched.

## Pin

The slice changes no code that was here before the method: the Cargo row (`src/slipwai/ecosystems/cargo.py`) was
written under it, in `single-crate`, and its tests are its pin (`tests/test_survey_cargo.py`,
`tests/test_survey_cargo_workspace.py`). `resurvey.reconciled_app` — code that was here — is not changed; TG8 is a
test of what it already does, observed green before and after. `delivery/survey/pinned.md` therefore gains no row.
D6's smoke (`./slipwai --version && ./slipwai adopt --next`) is recorded and is run at convergence.

## Design

`src/slipwai/ecosystems/cargo.py`:

| Name | What |
|---|---|
| `TOOLCHAIN_FILES` | `("rust-toolchain", "rust-toolchain.toml")` — in the order rustup prefers them within a directory (TG2, R1) |
| `pinned_channel(text: str) -> str` | `toolchain.channel` of a TOML text by `tomllib`; `""` for invalid TOML, a `toolchain` that is not a table, a `channel` that is not a string, or a `path` in the table (TG3, TG5, R11–R13) |
| `legacy_channel(text: str) -> str` | rustup's reading of a `rust-toolchain` (D22): its lines split as Rust's `str::lines` does (on `\n`, a trailing empty piece not counted); exactly one line → that line stripped, nothing else removed (R5–R7); more than one → `pinned_channel(text)` (R2, R8, R9); none → `""` |
| `rust_toolchain(root: Path, directory: str, reader=read) -> str` | from `root / directory` up to and including `root`, never above (TG1, D19): the first directory where either name exists (`os.path.lexists`, so a FIFO, a directory or a dangling link there still decides) chooses the first name present, reads it with the bounded `read` (empty for a non-regular, oversize or unreadable file — TG5) and returns `legacy_channel` or `pinned_channel` of it; no file on the way up → `""`. `rust-version` in `Cargo.toml` is never read (TG6). |
| `cargo(root, directory)` | as today, with `{"kind": "rust", "version": rust_toolchain(root, directory)}` (FR-005, TG9: the record's shape unchanged) |

The candidate's evidence stays `Cargo.toml` (TG7). `common.py` needs nothing: the walk is Cargo's alone (D19 keeps
every other row in its own directory), so it lives beside the row; `first_line` is not used, since it strips a `v`.

What a maintainer sees (TG7) needs no code: `adopt_report.survey_page` already prints `, rust <version>` where the
version is not empty, and nothing where it is. `platform.inventory` dates nothing for `rust` (`support.json` has no
such product), so the Platform record is unchanged (D21).

`adopt --refresh` (TG8) needs no code: `resurvey.reconciled_app` replaces a toolchain whose provenance is `detected`
with the fresh reading and reports `toolchain.version was confirmed as "" … now says "1.85"` where it is `confirmed`
or `overridden`. The plan adds the test.

## Project Structure

### Documentation (this slice)

```text
specs/001-rust-cargo-adopt/slices/toolchain-pin/
├── plan.md        # this file
├── research.md    # what rustup reads, observed
├── tasks.md       # drive-tasks
└── benchmark.json
```

### Source Code

```text
src/slipwai/ecosystems/cargo.py            # TOOLCHAIN_FILES, pinned_channel, legacy_channel, rust_toolchain; the row calls it
tests/test_survey_cargo_toolchain.py       # new suite: TG1–TG6, TG7 (survey page), TG8 (refresh)
tests/test_survey_cargo.py                 # the one empty-version assertion flipped to the pin its tree carries (TG7)
tests/fixtures/adopt/rust-crate/rust-toolchain.toml   # channel = "stable" (TG7)
scripts/test-adoption.py                   # rust-crate's expected version is "stable"; rust-workspace's stays ""
changelog.d/rust-cargo-adopt.md            # amended (TG10): the pin is recorded; What stays out keeps only CI; Catch-up names TG8
docs/adopting.md                           # one clause: where the Rust pin is read from
```

**Structure Decision**: the one deployable, `slipwai`, at the root — the only service `project.json` records, and its
purpose covers the survey. One vocabulary (candidate, toolchain, pin, channel), so one bounded context; strategy
`leave-it` (D2), so the change lands in the existing module. A new test suite rather than growing
`tests/test_survey_cargo.py`, which the sibling `optional-tools` also edits: the two merge mechanically.

**The fixture's pin is `stable`.** It is one of TG4's shapes, read from `rust-toolchain.toml` the way US2 scenario 1
reads `1.85`, and it is installed wherever the factory's CI and this machine have Rust, so `make test-adoption`'s
`verify` of the fixture never asks rustup to download a toolchain. A numbered pin (`1.85`) would make every run of the
factory's adoption job install that release, or fail where auto-install is off. The numbered shapes are each a
survey-level test (TG4).

## Not working yet (deliberate, owned by other slices)

- The adopted gate's CI sets up no Rust — `ci-toolchain` (SG2 stands); this slice hands it the version string only
  (TG9, D19).
- Components and targets a toolchain file lists are not read, and a Rust version has no support status
  (`support.json` has no `rust`) — out of scope (spec).
- `make -f delivery/Makefile smoke` still says none is recorded until a person regenerates the targets (D6).

## Questions returned

- **D22** was decided here, on D21's stated reason, and is returned for review: TG3's wording and rustup differ on a
  multi-line `rust-toolchain` that does not start with `[`; the plan follows rustup. If the sibling `optional-tools`
  also numbered a decision D22, one of the two is renumbered at merge.
- **D23** was decided at converge (T011, HIGH), on the same reason, and is returned for review: a channel that is not
  a toolchain name (`^[A-Za-z0-9][A-Za-z0-9._-]*$`) records an empty version, so a newline in a pin cannot break a
  generated file. Constitution XIV asks that the owner (or the skipper) accept the TG5 clause it added before merge.
- **Proposed D24, open — the owner's to decide** (T012, MEDIUM, Phase 4): where rustup cannot read a toolchain file
  (a dangling link, a directory of that name, an unreadable or non-UTF-8 file) it passes over it, to the other name
  and then upward, and it reads a `rust-toolchain.toml` starting with a byte order mark; TG5 as worded records an
  empty version and stops. Follow rustup (recommended, D21's reason), or keep TG5 as worded (T012 closes with that
  decision and the two tests stay)?
- **Not this slice's, returned:** the class T011 closed for Rust is open for every other ecosystem's pin that reaches
  `project/adopted_ci.py`'s `setup_steps` (a `.nvmrc` holding a quote, single-quoted at line 63) — code that was here
  before the method. Whether it is a slice of its own is the delegating session's decision.

## Complexity Tracking

None.
