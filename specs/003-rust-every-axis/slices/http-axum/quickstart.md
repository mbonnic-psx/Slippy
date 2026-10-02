# Quickstart — http-axum

What the slice lets someone do, from this checkout, with nothing installed but Git, Python, cargo (rustup reads the
generated `rust-toolchain.toml`) and, for the demo, Docker. Scratch directories go under
`$HOME/.cache/slippy-cruise-tmp/`, never `/tmp`.

```sh
out=$HOME/.cache/slippy-cruise-tmp/axum-demo
./slipwai generate ledger --language rust --profile standard --frontend none --output "$out"   # --http defaults to axum
cd "$out/ledger"
make verify                                   # fmt, clippy -D warnings, cargo llvm-cov ≥ 70%
make dev                                      # cargo run --locked --bin serve, on http://localhost:3000
curl -s localhost:3000/health                 # {"status":"ok"}
curl -s localhost:3000/ready                  # {"status":"ready"}
curl -s localhost:3000/nope                   # {"error":"notFound"}, 404
make demo                                     # the same `make dev` in rust:1.98-bookworm, healthy within the window
make demo-down
./init --http none && make verify             # the transport taken away; still green, axum gone from Cargo.lock
```

With a store: `--profile event-modelling --event-store sqlite` (or `memory`, `postgres`); `/ready` then asks the
store's head. With `--http none` the project is exactly what Rust generated before this slice.
