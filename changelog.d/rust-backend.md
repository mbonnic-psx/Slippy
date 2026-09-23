MINOR

**A new backend language: Rust, as `--language rust`.** A Rust service is one package in a Cargo workspace at
the repository root. The release is pinned in `rust-toolchain.toml` (1.98.1, with clippy, rustfmt and
llvm-tools), and `make verify` runs `cargo fmt --check`, `cargo clippy -- -D warnings` and
`cargo llvm-cov` with a 70% line-coverage floor. `make audit` runs cargo-deny's advisory check, and
`make mutation` runs cargo-mutants. It is the walking skeleton only: Rust answers no axis yet, so it gets no
event-store adapters, no transport and no identity provider. Under the event profile it gets the event-store
port with its version-conflict-is-a-value contract and nothing behind it. It is offered for the `none` and
`existing` targets; the AWS and Azure images arrive with its deploy work. Every skill's example snippets now
have a Rust version, and `make check-imports` reads Rust's `crate::` paths.

**A web app generated beside a service with no transport now passes its own lint.** Pruning the transport left
`App.tsx` importing a `Route` it no longer used, with an empty route table, and left a blank line in its test.
The route table now ends with a catch-all route saying there is nothing here yet, which a project with no API
shows on every path. This affected every backend with `--http none` and a browser app.

**Catch-up.** Nothing for an existing project unless it generated a web app with `--http none`: `slipwai
migrate` brings the fixed `App.tsx` and its test.
