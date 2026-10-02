# Research: optional-tools — which files cargo-deny and cargo-mutants read, and what they write

Every row below was read off a run of `cargo 1.98.0 (797e8a9bc 2026-08-05)`, `cargo-deny 0.20.2` and
`cargo-mutants 27.1.0` on 2026-10-01, over probe trees under `$HOME/.cache/slippy-optional-tools-tmp/probe1/`, each
run under `systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`. A config file was made invalid TOML
(`this is = = not toml`) so the tool's own parse error names the file it read; `cargo deny check bans` was used for
discovery because it needs no network, and discovery is the same for every check (one `--config` for the run).
Nothing here is *assumed*, except where a row says so.

| Question | Observed | Consequence for the proposal |
|---|---|---|
| Does cargo-deny read `deny.toml` in the directory it runs in? | `failed to parse config from '…/deny-denytoml/deny.toml'`, exit 1. | A trigger (OG1). |
| …`.deny.toml`? | `failed to parse config from '…/dotdeny/.deny.toml'`, exit 1. | A trigger (OG1, D20). |
| …`.cargo/deny.toml`? | `failed to parse config from '…/deny-cargo_denytoml/.cargo/deny.toml'`, exit 1. | A trigger (OG1, D20). |
| …with none of them? | `[WARN] unable to find a config path, falling back to default config`, `bans ok`, exit 0. | No file, no proposal: audit stays a written no (FR-003). |
| …with a `deny.toml` only in the parent of the crate? | `failed to parse config from '…/above/deny.toml'`: cargo-deny walks up. | cargo-deny honours a file above; the survey does not count it (D20, the reversal it names; *Out of scope*). |
| Where does `--workspace` go? | `cargo deny --help` lists `--workspace` among the top-level options ("all workspace packages are used as roots for the crate graph … Automatically assumed if the manifest path points to a virtual manifest"); `cargo deny check --help` does not. | `cargo deny --workspace check advisories` at a workspace root (OG2). At a virtual root it is redundant and harmless; at a root that is also a `[package]` it is what brings the members in. |
| Does cargo-mutants read `.cargo/mutants.toml` at the directory it runs in? | With `exclude_globs = ["**"]` there, `cargo mutants --list --workspace` lists nothing; made invalid, `Error: parse toml from …/ws/.cargo/mutants.toml`, exit 1. | The trigger (OG3). |
| …a bare `mutants.toml`? | With `exclude_globs = ["**"]` in `ws/mutants.toml`, `cargo mutants --list` still lists `src/lib.rs` mutants. | Not a trigger (OG3). |
| …a member's `.cargo/mutants.toml`? | With it in `ws/m/.cargo/`, every `m/src/lib.rs` mutant is still listed, whether run at the root with `--workspace` or inside `m/`. | Not a trigger; the workspace root's is the one read (OG3). |
| At a root that is both `[workspace]` and `[package]`, what does `cargo mutants` cover? | `--list` lists only `src/lib.rs`; `--list --workspace` lists `m/src/lib.rs` and `src/lib.rs`. | `cargo mutants --workspace` at a workspace root (OG3, D20), as WG2 does for check, clippy and test. |
| What does a run write? | `cargo mutants` in crate `one/` leaves `mutants.out/` beside `Cargo.toml`; a second run leaves `mutants.out/` and `mutants.out.old/`. | The adopted ignore block carries both where a Cargo app is recorded (OG5). Unanchored lines cover a crate in a subdirectory too. |
| A cargo subcommand that is not installed? | `cargo frobnicate-not-a-tool` → `error: no such command: \`frobnicate-not-a-tool\``, exit 101. | A machine without `cargo-deny` or `cargo-mutants` fails `make audit` / `make mutation` loudly (OG4); no guard is written (D20). |

Read off this repository rather than a tool:

- `src/slipwai/convergence.py` `platform_row` (lines 180–204): the Platform row is `audited` only where every product
  read is in support **and** every wrapped application records an `audit`. Rust has no `support.json` row (D21), so a
  repository whose only product is a Rust crate has no product to date and its row stays `unknown` with or without an
  audit. OG5's "as for any ecosystem" is that shared rule; the slice proves it in a repository that also has a product
  the table dates.
- `src/slipwai/project/native_commands.py` line 42: `RATCHETED = ("lint", "typecheck", "test")` — audit and mutation
  are never baselined (OG4). `delivery/Makefile`'s `verify` runs neither; the adopted CI runs `verify`.
- `src/slipwai/adopt.py` line 265–266 and `src/slipwai/converge.py` line 171–172 are the only writers of the adopted
  `.gitignore` block (`append_block`, `respell`); `migrate` and `adopt --refresh` never rewrite it. A repository
  adopted before this slice therefore gains no ignore line by itself: the Catch-up says so.
- `src/slipwai/project/gitignore.py` `build_artifacts`: every per-language line comes from `services_of(apps)`, the
  *generated* services; a wrapped application contributes nothing today, so the adopted block of every existing
  adoption carries no language line.

What was not probed, and so is not claimed: `cargo deny check advisories` itself (it fetches the advisory database
over the network; the subcommand and the discovery are the ones above); cargo-mutants' `--in-diff`, which this slice
does not propose.
