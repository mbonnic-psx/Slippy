# Adversary log — 001-rust-cargo-adopt

## single-crate · 667f513 · 2026-09-29

| Trigger | Status | Evidence |
|---|---|---|
| driving adapter (HTTP route, CLI command, queue consumer) | widened | `src/slipwai/ecosystems/cargo.py` — `slipwai adopt` now turns a `Cargo.toml` anywhere within three levels of an untrusted tree into shell commands recorded in `project.json` and run by `delivery/Makefile`; `src/slipwai/cli_adopt.py` refusal text |
| driven adapter or the provider types behind one | widened | `assets/adoption/scripts/ratchet.py` (Phase 4, T012) — the adopted gate reads cargo's output and exit codes to decide pass, baseline or refusal |
| authorisation decision (who can reach one that already exists) | not present | no identity or permission in the diff |
| concurrency, idempotency, ordering, retention, or time | already covered | re-run safety (`adopt --refresh` a no-op) is proved for the Rust fixture by `scripts/test-adoption.py`; the diff makes no ordering or time claim beyond the ecosystem order, pinned in `tests/test_survey_cargo.py` |

Resumed 2026-09-29 at 726baf7 (main merged, with #13's bounded read) after the first attempt's probe took the host
down (#13). Every probe ran under `systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`, `TMPDIR` on disk;
no OOM kill, no timeout.

Spawned: cargo survey row → shell commands · `drive-adversary` · opus-5.5 (host) · delegated, fresh context ·
manifest: `src/slipwai/ecosystems/{cargo,common,__init__,rows}.py`, `src/slipwai/survey.py`, `src/slipwai/cli_adopt.py`,
`src/slipwai/bounded_read.py`, `src/slipwai/wrappers.py`, `tests/test_survey_cargo.py`, `tests/test_refresh_cargo.py`,
`tests/test_adopted_rust_ci.py`, `tests/test_adopt.py`, `tests/fixtures/adopt/rust-crate/`, `scripts/test-adoption.py`
Spawned: ratchet reading cargo output and exit codes · `drive-adversary` · opus-5.5 (host) · delegated, fresh context ·
manifest: `assets/adoption/scripts/ratchet.py`, `tests/test_ratchet_cargo.py`, `tests/test_ratchet*.py`, the adopted
`delivery/Makefile` of `tests/fixtures/adopt/rust-crate/`
Omitted: authorisation · not present. Concurrency/time · already covered (row above).

Findings:

| # | Severity | Triage | State | Finding |
|---|---|---|---|---|
| S1 | CRITICAL | confirmed — reproduced by the host | open — fixing on `fix-unsafe-candidate-paths` (GHSA-3fpx-wg55-c4qj) | A directory name with a newline (`a\n$(shell touch X)\n#`) holding a `Cargo.toml` is written unescaped into `delivery/Makefile` (recipe, `?=` and `@echo` lines); make evaluates `$(shell …)` while parsing, so `make -f delivery/Makefile help` — or any target of the root `Makefile`, which `-include`s it — runs code nobody confirmed. Not Cargo-specific: every row's path reaches the Makefile the same way. |
| S2 | HIGH | confirmed — Makefile line seen by the host | open — same branch as S1 | `in_dir` (`ecosystems/common.py`) writes `cd {directory} && …` unquoted: `x;touch X;y/Cargo.toml` gives `cd x;touch X;y && cargo fetch --locked`; `$(…)` in a name reaches `sh -c` through the ratchet. Predates the slice (a `go.mod` does the same); the Cargo row widens the reach. |
| S3 | MEDIUM | confirmed — agent | open | A `cd` that fails (a name with a space, a leading `-`) exits 2, which the ratchet baselines as a known failure: the crate is never built and the gate is green. |
| S4 | LOW | confirmed — agent | open | `survey.directories` follows directory symlinks: a crate outside the repository is proposed, and `ln -s .. up` proposes one crate several times. Directory sibling of #15. |
| R1 | HIGH | confirmed — reproduced by the host | fixed (T014, commit to follow) | `ratchet.py` takes the first word after `cd` as the tool, so `cd ledger && cargo clippy …` — what the survey proposes for any crate one level down — never matches `tool == "cargo"`: a missing clippy is baselined (exit 101, no findings) and passes from then on. `RUSTFLAGS=… cargo` and `env … cargo` do the same. |
| R2 | HIGH | confirmed — host, 2026-09-29: a linked toolchain with only `cargo` and `rustc`, rustup 1.29.0, `cargo clippy` / `cargo fmt --check` exit 1 with `'cargo-clippy' is not installed …` | fixed (T015, commit to follow) | Under rustup, a missing clippy/rustfmt component is reported by the proxy as `error: 'cargo-clippy' is not installed …` with exit 1, not `no such command` with 101, and would be baselined. To confirm with `rustup component remove clippy` on a scratch toolchain before a fix relies on it. |
| R3 | HIGH | confirmed — agent, real cargo 1.98 | fixed (T016, commit to follow) | libtest's panic line carries the OS thread id (`thread 'tests::known_red' (520735) panicked at src/lib.rs:7:22:`), which becomes the key: a quarantined red suite fails on every later run with the same code. |
| R4 | HIGH | confirmed — agent, real cargo 1.98 | fixed (T017, commit to follow) | A test binary killed by a signal prints no location and exits 101, which the exit-code comparison accepts against a quarantined 101: a crashing test passes a quarantined suite. |
| R5 | MEDIUM | confirmed — agent, bytes read with `od` | fixed (T018, commit to follow) | With `CARGO_TERM_COLOR=always`, `no such command` is wrapped in ANSI codes and misses the `^error:` anchor, so it is baselined locally and passes on CI against an existing baseline. |
| R6 | MEDIUM | confirmed — agent | open | A failing test whose captured stdout prints `error: no such command: …` makes the whole run "could not run", false red, and cannot be quarantined. |
| R7 | LOW | confirmed — agent | fixed (with T016) | libtest's `test x ... FAILED` / `failures:` names match no `TEST_FAILURES` pattern: an `Err`-returning failure has no key. Masked by R3 today. |
| R8 | LOW | confirmed — agent | open | Output is buffered whole and findings are de-duplicated in a list (quadratic): 80k findings 16.8 s, ~140 MB output 498 MB RSS. Held under the cap. |

## workspace · 44de220 · 2026-09-30

| Trigger | Status | Evidence |
|---|---|---|
| driving adapter (HTTP route, CLI command, queue consumer) | widened | `src/slipwai/ecosystems/cargo.py` (`WORKSPACE`, `declares_workspace`, `member_of_workspace`) and `src/slipwai/survey.py` `buildable` — `slipwai adopt` now reads the *content* of an untrusted tree's `Cargo.toml` files, and of their ancestors, to decide which directories become recorded commands and which are hidden; `src/slipwai/quick_wins.py` `missing_lockfiles` |
| driven adapter or the provider types behind one | not present | the ratchet and the adopted Makefile are unchanged; the commands gain `--workspace` only |
| authorisation decision (who can reach one that already exists) | not present | no identity or permission in the diff |
| concurrency, idempotency, ordering, retention, or time | already covered | `adopt --refresh` a no-op on the new shapes is proved by `scripts/test-adoption.py` (`rust-workspace`) and was re-checked by the pass on the Tauri and nested shapes |

Every probe ran under `systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`, `TMPDIR` on disk, trees under
`$HOME/.cache/slippy-ws-tmp/adv/`; no OOM kill, no timeout. Real cargo 1.98.0 only as `cargo metadata --no-deps --offline`.

Spawned: survey reading untrusted `Cargo.toml` content for ownership and commands · `drive-adversary` · opus-5.5 (host) ·
delegated, fresh context · manifest: `src/slipwai/ecosystems/cargo.py`, `src/slipwai/survey.py`, `src/slipwai/quick_wins.py`,
`src/slipwai/bounded_read.py`, `tests/test_survey_cargo_workspace.py`, `tests/test_quick_wins.py`,
`tests/fixtures/adopt/rust-workspace/`, `scripts/test-adoption.py`
Omitted: driven adapter · not present. Authorisation · not present. Concurrency/time · already covered (row above).

Findings:

| # | Severity | Triage | State | Finding |
|---|---|---|---|---|
| W1 | HIGH | confirmed — reproduced by the host (HEAD `[('.', 'node')]`, `0e3bab3` `[('.', 'node'), ('packages/native/a', 'cargo')]`) | fixed (T015, `f01a9ec`) | A Cargo workspace whose directory's first detection is owned by an outer build of that ecosystem (a napi-rs package with its own `[workspace]` inside an npm workspace) vanishes, root and members: `member_of_workspace` counts the ancestor as owner though that directory is never proposed. A regression against `0e3bab3`, and D14's override cannot reach it. |
| W2 | MEDIUM | confirmed — agent, `cargo metadata` reads it as a workspace | fixed (T016, `570143d`) | A UTF-8 BOM before `[workspace]` on line 1 hides the header: members proposed each, no `--workspace`, false `no-lockfile` findings. |
| W3 | LOW | confirmed — agent, real cargo | open — named in the fragment (T017, `13a5f1c`) | The header text match disagrees with Cargo both ways: `[workspace]` inside a multi-line string makes a plain crate an owner and hides a crate below it; a quoted `["workspace"]` header or a top-level `workspace.members = […]` key is a workspace Cargo reads and the survey does not. The Assumptions' text match; named in the fragment by T017. |
| W4 | LOW | confirmed — agent (16 MB root, 500 members: 34.6 s against 0.06 s) | fixed (T018, `0f20431`) | `member_of_workspace` re-reads every ancestor manifest for every member, up to `MAX_READ` each, and `missing_lockfiles` tests ancestors against a list; each read is bounded, the total is not. |

## optional-tools · e524890 · 2026-10-01

| Trigger | Status | Evidence |
|---|---|---|
| driving adapter (HTTP route, CLI command, queue consumer) | widened | `src/slipwai/ecosystems/cargo.py` — `slipwai adopt` now reads four configuration names in an untrusted tree to decide whether `cargo deny … check advisories` and `cargo mutants` become recorded commands run by `delivery/Makefile`; `src/slipwai/project/gitignore.py`, `src/slipwai/adopt.py` — the adopted `.gitignore` block gains two lines |
| driven adapter or the provider types behind one | not present | the ratchet is unchanged; audit and mutation are not ratcheted (`project/native_commands.py` `RATCHETED`) |
| authorisation decision (who can reach one that already exists) | not present | no identity or permission in the diff |
| concurrency, idempotency, ordering, retention, or time | already covered | `adopt --refresh` idempotence over the new records is in `tests/test_survey_cargo_tools_refresh.py` and was re-checked by the pass |

Every probe ran under `systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`, `TMPDIR` on disk; cargo-deny
0.20.2 and cargo-mutants 27.1.0 installed.

Spawned: cargo tool configuration → recorded commands and ignore block · `drive-adversary` · opus-5.5 (host) · delegated,
fresh context · manifest: `src/slipwai/ecosystems/cargo.py`, `src/slipwai/project/gitignore.py`, `src/slipwai/adopt.py`,
`src/slipwai/resurvey.py`, `src/slipwai/project/{makefile,native_commands}.py`, `src/slipwai/convergence.py`,
`tests/test_survey_cargo_tools*.py`, the slice's `research.md` and `plan.md`
## toolchain-pin · 40be4a2 · 2026-10-01

| Trigger | Status | Evidence |
|---|---|---|
| driving adapter (HTTP route, CLI command, queue consumer) | widened | `src/slipwai/ecosystems/cargo.py` — `slipwai adopt` now reads `rust-toolchain` / `rust-toolchain.toml` from an untrusted tree, walking up to the root, and writes the value into `project.json`, the survey pages, `delivery/commands/ground.md` |
| driven adapter or the provider types behind one | not present | `project/adopted_ci.py` is unchanged (SG2) |
| authorisation decision (who can reach one that already exists) | not present | no identity or permission in the diff |
| concurrency, idempotency, ordering, retention, or time | widened | refresh re-reads the pin (TG8); the pass checked it stays a no-op |

Every probe ran under `systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`, `TMPDIR` on disk; compared with
rustup 1.29.0 (`rustup show active-toolchain`, `RUSTUP_AUTO_INSTALL=0`).

Spawned: toolchain file reading → recorded pin and pages · `drive-adversary` · opus-5.5 (host) · delegated, fresh context ·
manifest: `src/slipwai/ecosystems/{cargo,common}.py`, `src/slipwai/bounded_read.py`, `src/slipwai/resurvey.py`,
`src/slipwai/adopt_report.py`, `src/slipwai/project/adopted_ci.py`, `tests/test_survey_cargo_toolchain*.py`,
`tests/test_adopt_cargo_toolchain.py`, the slice's `research.md`
Omitted: authorisation · not present. Driven adapter · not present (row above).

Findings:

| # | Severity | Triage | State | Finding |
|---|---|---|---|---|
| OT1 | LOW | confirmed — agent, cargo-deny 0.20.2 in probe trees | open | A directory named `deny.toml` beside a real `.deny.toml` proposes audit, since the survey counts regular files and cargo-deny takes the first name that *exists*; `make audit` then fails `Is a directory` until the tree is fixed. Loud, never a false green. |
| OT2 | LOW | confirmed — agent, `od -c` | open — predates the slice | `adopt.append_block` reads and rewrites `.gitignore` in text mode, so a CRLF file comes back LF throughout and the adoption commit touches every user line. |
| OT3 | LOW | confirmed — agent | open — predates the slice | A user's `!mutants.out.old/` before the factory's block is overridden by the block's `mutants.out.old/`, which comes later. |
| AV1 | MEDIUM | confirmed — reproduced by the host (`rust-toolchain -> /etc/hostname` records the hostname) | fixed (COMMIT) | `readable_text` opens a toolchain file wherever its link resolves, so a tree can write a file of the adopting machine (hostname, machine id, a one-token credential file) into committed pages, and a link to a changing file makes every `adopt --refresh` rewrite four files. D19's "inside the checked-out tree" does not hold. |
| AV2 | LOW | confirmed — agent, rustup 1.29.0 | open | TOML 1.1 forms rustup's parser accepts (multi-line inline table, trailing comma, `\x` / `\e` escapes, a time without seconds) are invalid to `tomllib` (TOML 1.0): an empty version where rustup builds with a pin. |
| AV3 | LOW | confirmed — agent, rustup 1.29.0 | open | A file rustup refuses still records a pin: `\x1c`–`\x1f` around a legacy line are stripped by `str.strip()` and not by rustup; an invalid `profile` or a non-array `components` beside a good `channel`. The build fails either way. |
| AV4 | LOW | confirmed — agent, rustup 1.29.0 | open — the pin side of S4 | A candidate reached through a directory symlink walks the link's path while rustup walks the resolved one, so the nearest pin differs. |
