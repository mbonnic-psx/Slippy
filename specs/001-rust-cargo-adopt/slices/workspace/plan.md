# Implementation Plan: workspace — a Cargo workspace is proposed once, at its root

**Branch**: `001-rust-cargo-adopt-workspace` | **Date**: 2026-09-30 | **Spec**: [spec.md](../../spec.md) · slice 5 of
[story-split.md](../../story-split.md)

**Input**: US3 scenarios 1–3; FR-006; SC-002; the reworded nested-workspace edge case; the slice's gaps WG1–WG9;
decisions D11 (no feature flag), D12 (a nested workspace root is its own candidate), D13 (a member's lockfile is its
workspace's). What Cargo does is in [research.md](research.md), every row observed.

## Summary

A maintainer adopting a Cargo workspace is offered one candidate at the workspace root whose commands cover every
member, and no candidate per member. Three things change, all inside the survey: the Cargo row reads whether its
manifest declares a workspace and, where it does, proposes `--workspace` on check, clippy and test; a Cargo crate
is owned by any workspace root above it, whichever ecosystem reports that directory (D14), and a workspace root is
never itself owned (D12). The "no lockfile" quick win stops reporting members (D13). No other ecosystem's detection, commands or
ownership changes.

## Release constraint

**Held behind the pre-release, as standing decision D7 says for this feature.** A merge to `main` publishes
`1.4.0.dev<N>`, which installers pass over unless asked; only a person's `make release` makes it a release, and a
person merges the pull request. Nothing here reaches a maintainer whose repository has no `Cargo.toml` with a
workspace header in it: every other tree surveys as before (SC-004, pinned below).

## Technical Context

**Language/Version**: Python ≥ 3.11 (`requires-python`); run here on 3.14.

**Primary Dependencies**: none added — the standard library; the workspace header is a regular expression over the
manifest read with the survey's bounded `read`, as the Assumptions ask (no TOML parser).

**Storage**: files — the surveyed tree is read; `project.json` in the adopted repository is written by the existing
`adopt` path.

**Testing**: `python3 -m pytest` / `make test TESTS=…` — a new suite `tests/test_survey_cargo_workspace.py` at the
survey's boundary (`survey` / `buildable` over a tree on disk, `slipwai adopt` in a throwaway repository), the
lockfile case in `tests/test_quick_wins.py`; `scripts/test-adoption.py --only rust-workspace` for the committed
fixture end to end (WG8), which runs its `verify` where `cargo` is on the machine (`~/.cargo/bin/cargo`, 1.98.0).
Every run under `systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`, `TMPDIR` on disk (#13).

**Target Platform / Project Type**: the `slipwai` CLI, wherever it runs.

**Performance Goals / Constraints / Scale**: one bounded read of each `Cargo.toml` the walk meets (the row already
stats it); each manifest is read once per survey (T018: a memo per call, and a set for the tracked paths — the
first cut re-read every ancestor per member, adversary W4).

## Constitution Check

- **I. What a project was given keeps meaning what it meant** — holds. The nine other ecosystems' detection,
  commands and ownership are unchanged: the ownership exception is keyed on the Cargo row alone, and the lockfile
  change on `Cargo.toml` alone (pinned below; WG6's npm regression case). A single crate keeps `single-crate`'s
  commands word for word (WG3). No release carried the Cargo row, so changing what a workspace member is proposed
  as changes no released answer; the fragment `changelog.d/rust-cargo-adopt.md` is extended (still `MINOR`, still
  experimental), `VERSION` stays `1.4.0.dev0`.
- **II. Re-running is safe** — holds. No new writing command; WG8 proves `adopt --refresh` on the adopted workspace
  fixture changes nothing, and WG9 that a member recorded by an earlier snapshot is reported, never removed.
- **III. Simplicity** — holds. One regular expression, one function that says whether a manifest declares a
  workspace, one membership predicate shared by the survey and the lockfile rule, one branch in `buildable`.
- **V (as it holds today)** — every new test enters at the survey's boundary or through `slipwai adopt`; fakes
  only, no mock.
- **VIII** — no new value in `project.json`; commands are strings the record already carries.
- IV, VI, VII, IX–XI — not touched.

## Pin

The slice changes code that was here before the method: `survey.buildable`'s ownership rule and
`rows.aggregates` (both from the initial release, 8ac6145), and `quick_wins.missing_lockfiles` (same). The Cargo
row itself was written under the method and its tests are its pin.

1. **The nine existing ecosystems are owned as today** — pinned already: `delivery/survey/pinned.md` row 1
   (`tests/test_survey.py`, `make test-adoption`). A nested npm `"workspaces"` under an npm workspace root being owned
   is not asserted by that suite; T001 adds it as a characterisation, observed green before any change.
2. **`missing_lockfiles` reports every `Cargo.toml` with no `Cargo.lock` in its own directory, whatever is above it**
   — observed and pinned by T001 before the change: a member under a workspace root with a lock is reported today
   (the case D13 changes on purpose, so its assertion is inverted in T005), and a plain crate and a `package.json`
   with no lock are reported (unchanged).

`delivery/survey/pinned.md` is not this slice's to write (the shared-surface rule); the two rows to append there —
dated 2026-09-30, seams `survey.buildable` and `quick_wins.missing_lockfiles`, tests
`tests/test_survey_cargo_workspace.py` and `tests/test_quick_wins.py`, run with
`make test TESTS="test_survey_cargo_workspace test_quick_wins"` — are handed back to the delegating session.

## Design

`src/slipwai/ecosystems/cargo.py`:

| Name | What |
|---|---|
| `WORKSPACE` | `re.compile(r"(?m)^[ \t]*\[[ \t]*workspace[ \t]*[.\]]")` — a `[workspace]` or `[workspace.<x>]` table header at the start of a line (WG1). Not `workspace = true`, not `package.workspace = "…"`, not `# [workspace]`, not `[[workspace…]]`. |
| `declares_workspace(manifest: Path) -> bool` | the header is in the manifest's bounded read; `False` for a file that cannot be read |
| `cargo(root, directory)` | as today, with `flag = " --workspace" if declares_workspace(manifest) else ""` in typecheck `cargo check{flag} --all-targets`, lint `cargo clippy{flag} --all-targets --message-format=short -- -D warnings && cargo fmt --check`, test `cargo test{flag}`. Install and the fmt half unchanged (research). No feature flag (D11). |

`member_of_workspace(root, manifest, present=None) -> bool` (in `cargo.py`, added at T010 for D14) — a
`Cargo.toml` that declares no workspace itself, with an ancestor `Cargo.toml` (inside the root; `present` says which
count: the files on disk, or the ones Git tracks) that declares one. It is the one membership rule: `buildable` and
`missing_lockfiles` both ask it.

`src/slipwai/ecosystems/__init__.py` re-exports `declares_workspace` and `member_of_workspace`. `rows.py` is
unchanged: the first design here gave `aggregates` a Cargo branch and added `stands_alone`, and both were removed at
T010, because an owner recorded only for a directory *detected* as Cargo missed a workspace root in a directory
reported as Node or Python first (D14).

`src/slipwai/survey.py` `buildable`: a Cargo candidate is owned exactly when `member_of_workspace` says so, whichever
ecosystem reports the owning root's directory (D14); every other ecosystem's ownership as before. Where a directory's
first detection is owned by an outer build of its ecosystem and its `Cargo.toml` declares a workspace, the directory is
proposed as the Cargo candidate (D15, T015). The docstrings name Cargo workspaces.

`src/slipwai/quick_wins.py` `missing_lockfiles` (D13, WG7): for `Cargo.toml` only, where no `Cargo.lock` is beside it,
the manifest is skipped when it declares no workspace itself and some ancestor directory's tracked `Cargo.toml`
declares one — its lock is that root's, which is judged at its own path. Everything else as today.

## Project Structure

### Documentation (this slice)

```text
specs/001-rust-cargo-adopt/slices/workspace/
├── plan.md        # this file
├── research.md    # what Cargo does at a workspace root, observed
├── tasks.md       # drive-tasks
└── benchmark.json
```

### Source Code

```text
src/slipwai/ecosystems/cargo.py            # WORKSPACE, declares_workspace, --workspace in the row
src/slipwai/ecosystems/__init__.py         # re-exports
src/slipwai/survey.py                      # buildable: a Cargo workspace root is never owned
src/slipwai/quick_wins.py                  # a member's lockfile is its workspace root's
tests/test_survey_cargo_workspace.py       # new suite: WG1–WG6, WG9
tests/test_quick_wins.py                   # WG7 (pin first, then the change)
tests/fixtures/adopt/rust-workspace/       # virtual workspace, crates/ledger + crates/report, Cargo.lock, README.md
scripts/test-adoption.py                   # ADOPTIONS["rust-workspace"], its recorded commands (WG8)
changelog.d/rust-cargo-adopt.md            # extended: workspaces, and the Catch-up for a snapshot adoption
docs/adopting.md                           # one clause: a Cargo workspace owns its members, like npm/Maven/Gradle/.NET
```

**Structure Decision**: the one deployable, `slipwai`, at the root — the only service `project.json` records, and its
purpose covers the survey. One vocabulary (candidate, ecosystem, owner, member, workspace), so one bounded context;
strategy `leave-it` (D2), so the change lands in the existing modules. A new suite rather than growing
`tests/test_survey_cargo.py` (129 lines), so each stays readable and well inside the 350-line budget
(`scripts/check-structure.py`).

The fixtures the brief asks for are all examples of the new suite, written as trees on disk: the Tauri shape (WG5),
a pure virtual workspace (WG2, WG4, also committed as `rust-workspace` for WG8) and the nested workspace root (WG6).

## Not working yet (deliberate, owned by other slices)

- Audit and mutation are always a written no — `optional-tools`.
- The toolchain version is always empty — `toolchain-pin`.
- The adopted gate's CI sets up no Rust — `ci-toolchain`.
- `members`, `exclude` and `default-members` are not read: a crate below a workspace root that the root excludes, and
  that declares no workspace of its own, is owned and not proposed (the Assumptions' text match; spec, WG4).
- A feature matrix is not proposed; `--all-features` is one override away (D11).
- `make -f delivery/Makefile smoke` still says none is recorded until a person regenerates the targets (D6).

## Complexity Tracking

None.
