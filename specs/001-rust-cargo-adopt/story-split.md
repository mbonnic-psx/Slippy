# Story split — Adopt recognises a Rust (Cargo) repository

The capability is *a maintainer of an existing Rust repository runs `slipwai adopt` and is offered something to
confirm*. Each slice below leaves that maintainer able to do one more thing, end to end, through the survey's
own output and the `project.json` it records — never a layer of the survey on its own. Split by
**rules** and **data variations** (SPIDR): the one-crate-at-the-root case first, as the walking skeleton, then
each variation the specification names as its own slice.

The changelog fragment (FR-009) opens with slice 1, claiming MINOR, and every later slice extends it; `VERSION`
already reads `1.4.0.dev0`, the MINOR the fragment claims, so no slice raises it.

## Slices, in order

| # | Slice | What the maintainer can do afterwards | Specification | Depends on |
|---|---|---|---|---|
| 1 | `single-crate` | Adopt a one-crate repository — at the root or in a subdirectory — and be offered a Rust / Cargo candidate with `Cargo.toml` as its evidence and a command, or a written no-answer, for all eight targets | US1 scenarios 1, 2, 5; the `audit` and `mutation` no-answers of scenarios 3 and 4; FR-001, FR-002, FR-004, FR-008 (fixture and detection tests), FR-009; SC-001, SC-004; edge cases: skipped directories, mixed-manifest directory, malformed `Cargo.toml`, no `Cargo.lock` | — |
| 2 | `optional-tools` | Be offered `cargo deny check` as audit where the crate carries a `deny.toml`, and `cargo mutants` as mutation where it carries `.cargo/mutants.toml` | US1 scenarios 3 and 4 (the configured halves); FR-003 | 1 |
| 3 | `toolchain-pin` | See the Rust toolchain the repository pins recorded — from `rust-toolchain.toml`'s `channel`, the legacy `rust-toolchain` line, or an empty version where none is pinned | US2 scenarios 1–3; FR-005 | 1 |
| 4 | `ci-toolchain` | Push the adopted repository and have its gate's CI install Rust before running the recorded commands, and see `make -f delivery/Makefile verify` pass in an adopted Rust fixture | US2 scenarios 4 and 5; FR-007; SC-003 | 3 |
| 5 | `workspace` | Adopt a Cargo workspace and be offered exactly one candidate, at its root, whose commands cover every member | US3 scenarios 1–3; FR-006; SC-002; edge case: nested workspace root | 1 |

## Slice graph

```text
single-crate ──┬── optional-tools
               ├── toolchain-pin ── ci-toolchain
               └── workspace
```

- `single-crate`: depends_on none
- `optional-tools`: depends_on single-crate
- `toolchain-pin`: depends_on single-crate
- `ci-toolchain`: depends_on toolchain-pin
- `workspace`: depends_on single-crate

Slices 2, 3 and 5 are ready together once slice 1 is done. They share one written contract — the Cargo row of the
survey's ecosystem table and the candidate it proposes, as slice 1 leaves them — and each touches a different
part of it (the audit and mutation proposals, the toolchain, the aggregation), so each is tested against that
contract rather than against its siblings.

## Why not another split

- **Not by target.** Eight slices of one command each would each be a line in one table, with no maintainer able
  to adopt anything until the last; slice 1 answers all eight at once, the two optional ones as written
  no-answers.
- **Not detection, then commands.** A candidate with no commands is the state the issue reports as the problem;
  the walking skeleton is a candidate the maintainer can confirm.
- **The workspace is last of its siblings, not first,** though most real repositories are workspaces: the
  first user's need (issue #10) is met by slice 1, and aggregation changes nothing slice 1 records for a single
  crate.
