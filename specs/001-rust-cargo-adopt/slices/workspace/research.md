# Research: workspace — what Cargo does at a workspace root

Every row below was read off a run of `cargo 1.98.0 (797e8a9bc 2026-08-05)` and `rustfmt` from the same
toolchain on 2026-09-30, over probe trees under `$HOME/.cache/slippy-ws-tmp/probe/`, each run under
`systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`. Nothing here is *assumed*.

Probe trees:

- `cairn/` — the shape of mbonnic-psx/Cairn: a root `package.json`; `src-tauri/Cargo.toml` holding both
  `[workspace] members = ["helper"]` (with `[workspace.package]`) and `[package] name = "cairn"` with `[lib]` and a
  feature; `src-tauri/helper/Cargo.toml` a member; one `#[test]` in each crate; `Cargo.lock` generated at
  `src-tauri/`.
- `virt/` — a virtual workspace: `[workspace] members = ["crates/*"]`, `default-members = ["crates/a"]`, crates `a`
  and `b`, one test each.
- `wsdep/` — a `[package]` whose manifest also carries only `[workspace.package]`, and a crate `m/` below it.

| Question | Observed | Consequence for the proposal |
|---|---|---|
| At a root that is both `[workspace]` and `[package]`, what do `cargo check`, `cargo clippy`, `cargo test` build without a flag? | Only the root package: `Checking cairn` and `test t::root_test`; `helper` is not built and its test does not run. | Without `--workspace` the gate would never see a member. Check, clippy and test carry `--workspace` (US3 scenario 2). |
| …and with `--workspace`? | Both: `Checking helper`, and `test t::root_test` then `test t::helper_test`. | `--workspace` covers every member. |
| At a virtual root with `default-members = ["crates/a"]`, what does `cargo test` run? | `t_a` only; `cargo test --workspace` runs `t_a` and `t_b`. | A virtual workspace needs the flag too wherever it sets `default-members`, so the flag is proposed for every workspace root, not only one that is also a package. |
| Does `cargo fmt --check` at the workspace root check the members? | Yes. With the root formatted and `helper/src/lib.rs` not, `cargo fmt --check` exits 1 and prints `Diff in …/helper/src/lib.rs`. `cargo fmt --all --check` gives the same. | The fmt half of lint is unchanged. `--all` is not added: it also formats path dependencies outside the workspace (rustfmt's `--all`: "Format all packages, and their local path-based dependencies"), which are not this candidate's. |
| Does `cargo fetch --locked` at the root fetch for the whole workspace? | Exits 0 at `src-tauri/` with the one `Cargo.lock`; `cargo fetch` has no `--workspace` flag, and a member directory holds no lockfile of its own (`ls src-tauri/helper` → `Cargo.toml src`). | Install is unchanged. The one lockfile sits at the workspace root (gap WG7). |
| A member listed in `members` that itself declares `[workspace]`? | Every command at the outer root fails: `error: multiple workspace roots found in the same workspace`. | The spec's edge case is a configuration Cargo refuses; D12. |
| A crate below the root that declares its own `[workspace]` and is excluded from the outer one? | `cargo check --workspace` at the outer root does not build it (`Checking a` only once `b` is excluded); `cargo check` inside it builds it (`Checking b`). | A nested workspace root is a separate workspace the outer root's commands do not cover; D12. |
| Is a manifest with `[workspace.package]` and no `[workspace]` header a workspace root? | Yes: `cargo check` in the unlisted crate below it fails with `current package believes it's in a workspace when it's not … workspace: …/wsdep/Cargo.toml`. | A root is recognised by a `[workspace]` **or** `[workspace.<table>]` header at the start of a line (WG1). `workspace = true` inside a dependency and `package.workspace = "…"` are a member pointing *at* a root, not a root. |

What was not probed, and so is not claimed: an inline top-level `workspace = { … }` table (valid TOML, which the
text match does not read — the Assumptions accept a text match); `[patch]`/`[replace]` across workspaces.
