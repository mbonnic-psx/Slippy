# Demo log — slice `single-crate`

## 2026-09-28T23:27:14Z — accepted · iteration 2 · drive-hand (claude-opus-5-5)
- **Started with:** `cp -r tests/fixtures/adopt/rust-crate /tmp/carin && cd /tmp/carin && git init -q && git add -A && git -c user.name=carin -c user.email=carin@local commit -qm "the crate" && /home/mbonnic/Slippy/slipwai adopt --yes`, then `env -u CRUISE_RUNNER -u CRUISE_ITERATION make -C /tmp/carin -f delivery/Makefile verify`. The same steps with the crate at `crates/ledger/` of `/tmp/carin-sub`. `slipwai adopt` with no `--yes`, under a pty, in `/tmp/carin-noyes`. · **Seeded:** the committed fixture `tests/fixtures/adopt/rust-crate`, nothing else. SG1/SG4 used one throwaway copy (`/tmp/carin-sg`) that added `rust-toolchain.toml` (`channel = "1.80.0"`) and a `Dockerfile`. All `/tmp/carin*` directories deleted afterwards.
- **Driven through:** CLI. `.specify/cruise.json` names `hand: browser`, but this slice has no screen and no HTTP surface: `slipwai adopt` is a command-line verb and the gate is `make`. So the CLI is the demo.
- **Examples:**
  - US1-1: passed. There is one candidate at `.`: language `rust`, ecosystem `cargo`, evidence `Cargo.toml`. The pty prompt shows `.  rust  from Cargo.toml  (4 of 8 targets have a command)` and `survey.md` shows `` `.` — cargo, rust, from `Cargo.toml` ``.
  - US1-2: passed. `project.json` has install `cargo fetch --locked`, typecheck `cargo check --all-targets`, lint `cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt --check`, test `cargo test`, and integration and adversarial `null`. `adoption.md` writes each `null` as "a written no".
  - US1-3 (no-answer half): passed. There is no `deny.toml`, and audit is `null` (a written no).
  - US1-4 (no-answer half): passed. There is no `.cargo/mutants.toml`, and mutation is `null` (a written no).
  - US1-5: passed. With the crate at `crates/ledger`, the candidate is `crates/ledger` with evidence `crates/ledger/Cargo.toml`. All four commands are prefixed once with `cd crates/ledger && `. `verify` is green there as well.
  - SG1: passed. The toolchain is `{kind: rust, version: "", ecosystem: cargo}` in both runs. It stays empty even when a `rust-toolchain.toml` pinning `1.80.0` is present, and the survey page shows no version.
  - SG3: passed. The subdirectory lint command is exactly `cd crates/ledger && cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt --check`. The `cd` holds for the fmt half too:
    - `cargo fmt --check` fails at the repo root (no `Cargo.toml` there).
    - The recorded lint command fails with a fmt diff on a misindented `crates/ledger/src/lib.rs`.
    - It exits 0 once that file is restored.
  - SG4: passed. With no `Dockerfile`, kind is `application` with provenance kind `unrecorded`, and the survey says "what it is for, nothing here says". With a `Dockerfile` beside the crate, kind is `service` with provenance `detected`: "a service by `Dockerfile`".
  - Gate (board): passed. `verify` exits 0 and prints "verify: all gates passed". It ran lint, typecheck and test through `ratchet.py carin …` with the Cargo commands, and 1 test passed. `verify` was also green in the `crates/ledger` case.
- **Evidence:** everything is under `specs/001-rust-cargo-adopt/slices/single-crate/demo/`:
  - `adopt-yes.txt`, `project.json`, `survey.md`, `adoption.md` and `verify.txt`: the root case.
  - `adopt-sub-yes.txt`, `project-sub.json`, `verify-sub.txt` and `sub-fmt-teeth.txt`: the subdirectory case.
  - `adopt-without-yes.txt` and `adopt-interactive.txt`: `adopt` without `--yes`.
  - `sg1-sg4-toolchain-dockerfile.txt`: the SG1/SG4 copy.
- **Feedback:** nothing re-enters the ladder. Notes for the next slices:
  - Without `--yes`, `adopt` only shows the count "4 of 8 targets have a command" before it writes anything. The eight commands first appear in `delivery/docs/adoption.md`, after the commit. Every ecosystem works this way; it is not specific to Rust.
  - Without a tty, `slipwai adopt` exits 2 with "nothing to ask on; pass --yes".
  - Ctrl-C at `./delivery/init`'s extension menu prints two Python tracebacks rather than a clean stop. This affects every ecosystem.
  - The fixture has no `.gitignore`, so `verify` leaves `target/` untracked. A real crate made by `cargo new` ignores `/target`, so this is not a product fault.
  - `adoption.md`'s heading "an application whose role is not recorded in rust (cargo)" reads awkwardly. The sentence is shared across ecosystems.
