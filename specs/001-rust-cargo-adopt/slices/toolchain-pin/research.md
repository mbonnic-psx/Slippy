# Research: toolchain-pin — what rustup reads, observed

Every row below was observed on 2026-10-01 with rustup 1.29.0 (28d1352db 2026-03-05), toolchains `stable`
(rustc 1.98.0) and `1.98.1` installed, `RUSTUP_AUTO_INSTALL=0`, by `rustup show active-toolchain` run in a probe
directory under `$HOME/.cache/slippy-toolchain-pin-tmp/` (the two probe scripts, `probe.sh` and `probe2.sh`, are
there). The page rustup itself publishes on the subject is https://rust-lang.github.io/rustup/overrides.html, which
the gaps review read (spec, *Slice `toolchain-pin` — Gaps reviewed*); nothing here rests on it alone.

| # | Tree | rustup said | What it means for the survey |
|---|---|---|---|
| R1 | `rust-toolchain` (`1.98.1`) and `rust-toolchain.toml` (`channel = "stable"`) in one directory | `warn: both … exist; using contents of …/rust-toolchain`, then `1.98.1` | TG2: the legacy file wins within a directory. |
| R2 | `rust-toolchain` holding `[toolchain]\nchannel = "1.98.1"` | `1.98.1 (overridden by …/rust-toolchain)` | TG3: the legacy file may be TOML (D21's "would reverse if" does not hold). |
| R3 | `rust-toolchain` (`1.98.1`) at the root, run from `crates/a` | `1.98.1 (overridden by …/up/rust-toolchain)` | TG1: rustup walks up from the directory the gate `cd`s into. |
| R4 | `rust-toolchain` (`1.98.1`) at the root, `sub/rust-toolchain.toml` with `components = ["clippy"]` and no channel, run from `sub` | `stable (overridden by …/sub/rust-toolchain.toml)` — the default, not the root's `1.98.1` | TG1: the nearest file decides and the search stops there even when it names no channel; recording empty is what rustup builds with (its default). |
| R5 | `rust-toolchain` holding `v1.98.1` | `error: custom toolchain 'v1.98.1' … is not installed` | TG3/TG4: no leading `v` is stripped — rustup does not strip it, and `first_line` (which does) is not used. |
| R6 | `rust-toolchain` holding `  1.98.1  \n` (one padded line) | `1.98.1` | TG3: one line is stripped. |
| R7 | `rust-toolchain` holding `1.98.1` with no newline | `1.98.1` | as R6. |
| R8 | `rust-toolchain` holding `\n  1.98.1  \n` | `error: … TOML parse error at line 2` | rustup's rule is *one line is a channel, anything else is TOML* — not "starts with `[`". See R11. |
| R9 | `rust-toolchain` holding `1.98.1\n\n` | `error: … TOML parse error at line 1` | as R8: a blank line after the channel makes it two lines, so TOML. |
| R10 | `rust-toolchain` empty | `error: … empty toolchain override file detected` | TG5: empty version (rustup refuses the file; the survey records no pin, never an error). |
| R11 | `rust-toolchain` holding TOML with `channel = 3` | `error: … invalid type: integer 3, expected a string` | TG5: a non-string channel is no pin. |
| R12 | `rust-toolchain.toml` holding `path = "/nonexistent"` under `[toolchain]` | `error: invalid toolchain: the path '/nonexistent' has no bin/ directory` | TG5: a `path` is a place on someone's machine; never recorded. |
| R13 | `rust-toolchain.toml` holding `[toolchain\nchannel = ` | `error: … TOML parse error … unclosed table` | TG5: invalid TOML is no pin. |
| R14 | `rust-toolchain.toml` with `channel = "stable"` | `stable (overridden by …)` | TG4: a named channel is a pin, recorded as written. |

## R8/R9 against TG3's wording

TG3 says a `rust-toolchain` is TOML "where it starts with `[`, otherwise its first non-blank line". rustup's rule,
observed in R6–R9, is narrower and different at the edge: **exactly one line is the channel, stripped; more than one
line is TOML**. The two agree on every case TG3 and TG4 name (a one-line channel; a TOML file starting with
`[toolchain]`). They disagree on a file of more than one line that does not start with `[`:

- `# pinned for the MSRV\n[toolchain]\nchannel = "1.85"\n` — rustup reads `1.85`; "first non-blank line" would record
  `# pinned for the MSRV` as the version, which `ci-toolchain` would hand to the setup action.
- `1.85\n\n` or `\n1.85\n` — rustup refuses the file (R8, R9); "first non-blank line" would record `1.85`, a pin the
  build cannot use.

D21's own reason is that "the recorded value is what rustup and the setup action will consume, so it is rustup's
reading or nothing". The plan therefore implements rustup's rule (decision D22), which keeps every case TG3 and TG4
name and records nothing where rustup would read nothing.

## Where this lives in the code today

- `src/slipwai/ecosystems/cargo.py` `cargo()` returns `{"kind": "rust", "version": ""}` unconditionally (SG1).
- `src/slipwai/ecosystems/common.py` `first_line` strips a leading `v` — right for `.nvmrc`, wrong for R5 — so the
  Cargo row does not use it.
- `src/slipwai/bounded_read.py` `read` returns `""` for a missing, non-regular or oversize file, which is TG5's empty
  version for free.
- `tomllib` is in the standard library from Python 3.11 (`requires-python >= 3.11`); `tomllib.loads` raises
  `TOMLDecodeError` on invalid TOML, and on a leading U+FEFF (Python 3.14, observed by `python3 -c` in this
  worktree), which the row catches as no pin.
- `src/slipwai/resurvey.py` `reconciled_app` already refreshes a toolchain whose provenance is `detected` and reports
  a field the tree newly reads differently where it is `confirmed` or `overridden` (lines 89–107); TG8 needs no change
  there, only a test.
- `src/slipwai/adopt_report.py` `survey_page` (line 52) and `src/slipwai/project/adopted.py` (line 112) print
  `<kind> <version>` only where the version is not empty; TG7's page needs no change there.
- `src/slipwai/platform.py` `dated` returns `None` for a product `support.json` does not know; it has no `rust`, so a
  pin adds nothing to the Platform record (D21, out of scope).
