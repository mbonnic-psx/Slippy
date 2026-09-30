PATCH

**The ratchet no longer records a Cargo build that stopped in a build script as a baseline.** A crate whose
dependency needs a system library the machine lacks (a Tauri app without libdbus's headers, say) exits 101
from `cargo clippy`, `cargo check` and `cargo test` with `error: failed to run custom build command for
`<package>`` before a line of the repository's code is checked. The first run of `make verify` recorded
that as `exit 101, no findings`, and every later run that got no further passed on the exit code. The ratchet
now reads it the way it reads a missing tool: the run fails, names the package whose build script did not
finish, and records nothing, whether or not a baseline is there already and under `make ratchet-tighten`.

**Catch-up.** A `delivery/baseline.json` entry of `{"exit": 101, "findings": []}` for a Cargo application may be
this. Once `slipwai migrate` brings the new ratchet, `make verify` fails on it and names the package: install
what its build script looks for, or record a command this machine can build (for example with
`--no-default-features`), then delete the entry and run `make verify` again to record a real one.
