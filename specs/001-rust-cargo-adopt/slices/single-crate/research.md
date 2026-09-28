# Research — single-crate

Every statement about Cargo below cites where it was read: `cargo help <command>` from cargo 1.98.0
(797e8a9bc 2026-08-05), clippy 0.1.98 and rustfmt 1.9.0-stable, on this machine, 2026-09-28.

- **`cargo fetch --locked`** — "Asserts that the exact same dependencies and versions are used as when the
  existing Cargo.lock file was originally generated. Cargo will exit with an error when either" the lockfile is
  missing or would change (`cargo help fetch`, `--locked`). So with no `Cargo.lock` install fails loudly rather
  than resolving — the spec's edge case, as stated.
- **`cargo check --all-targets`** — "Check all targets. This is equivalent to specifying --lib --bins --tests
  --benches --examples" (`cargo help check`). Tests are type-checked too, which is what `typecheck` means for the
  other rows (`go build ./...`, `tsc --noEmit`).
- **`cargo clippy --all-targets -- -D warnings`** — clippy takes cargo's target selection before `--` and
  rustc's lint flags after it; `-D warnings` turns every warning into an error, so lint fails on a warning.
  *Assumed* from clippy's documented usage; the fixture's `verify` run under `make test-adoption` is the run
  that proves it here.
- **`cargo fmt --check`** — rustfmt's check mode: exits non-zero and prints the diff where a file is not
  formatted, and writes nothing. Proven by the same fixture run.
- **No `[workspace]` parse.** The spec's Assumptions say the manifest is read by file name (this slice) and text
  match for `[workspace]` (`workspace`), with no TOML parser — the other rows read their manifests the same way.

The factory's own generated Rust projects (`assets/languages/rust/`) are depth 4 below this repository's root,
past the survey's `DEPTH = 3`, so this repository's own survey does not start proposing them.
