# Research — http-axum

Every statement about a crate below cites where it was read: `cargo search` / `cargo info` against crates.io from
cargo 1.98.0 on this machine, and a throwaway crate built and run under
`$HOME/.cache/slippy-cruise-tmp/axum-spike` (the *spike*), 2026-10-02. A statement with no citation says
*assumed*, and the plan does not rest on it without the task that proves it.

## The crates, and the versions current today

`cargo search --limit 1 <crate>`, 2026-10-02:

| Crate | Current | Role in the slice |
|---|---|---|
| `axum` | 0.8.9 | the router, the extractors, `axum::serve` |
| `tokio` | 1.53.1 | already pinned by the event store (`cargo.EVENT_STORE_CRATES`); the transport adds `net` and `signal` |
| `serde` / `serde_json` | 1.0.229 / 1.0.151 | already pinned by the event store; the transport needs both too |
| `tracing` | 0.1.44 | spans and log records |
| `tracing-subscriber` | 0.3.23 | the JSON and the readable formatters, the level filter |
| `opentelemetry` / `opentelemetry_sdk` / `opentelemetry-otlp` | 0.33.0 each | the SDK and the OTLP/HTTP exporter — Go's `otel` + `sdk` + `otlptracehttp` |
| `tracing-opentelemetry` | 0.34.0 | the bridge from a `tracing` span to an OpenTelemetry one |
| `serde_path_to_error` | 0.1.20 | names the field in a 400 for a wrong-type value, as Go's names `UnmarshalTypeError.Field`; adds no package to any lock — axum's `json` feature already depends on it (D9) |
| `tower` | 0.5.3 | dev-dependency: `ServiceExt::oneshot`, which dispatches through the real router with no socket |
| `http-body-util` | 0.1.5 | dev-dependency: reading a response body in an edge test |
| `tower-http` | 0.7.1 | **not taken** — see *CORS* below |

- **`tracing-opentelemetry` 0.34.0 is built against `opentelemetry` 0.33.0** — `cargo info -v
  tracing-opentelemetry@0.34.0` lists `opentelemetry@0.33.0` and `tracing-subscriber@0.3.22` among its
  dependencies. The two are released in pairs; a mismatched pair does not share trait types.
- **`opentelemetry-otlp` 0.33.0's defaults** are `http-proto, reqwest-blocking-client, trace, metrics, logs,
  internal-logs` (`cargo info opentelemetry-otlp@0.33.0`). The slice takes `default-features = false` with
  `http-proto`, `reqwest-blocking-client`, `reqwest-rustls` and `trace`: traces only, as Go exports traces only.

## What the spike proved

The spike's manifest: axum (`default-features = false`, `http1, json, tokio, query`), the OpenTelemetry trio
above, `tracing-opentelemetry` (`default-features = false`), `tracing-subscriber` (`fmt, json, registry, std,
ansi`), tokio with `macros, net, rt-multi-thread, signal, sync, time`; dev-dependencies tower (`util`) and
http-body-util.

- **It resolves and builds in about half a minute.** `cargo build --all-targets` with `CARGO_BUILD_JOBS=2`: 30 s
  with no TLS, 36 s once `reqwest-rustls` was added (registry warm, target directory cold). The lock: 136
  packages without TLS, 202 with it, no store crates in either.
- **A `GET /health` through `Router::oneshot` answers 200 with `{"status":"ok"}`** — the edge-test shape, no
  socket (`cargo test`, the spike's one test, green).
- **The exporter is built before the runtime, from a synchronous `fn main`, and nothing panics.** The blocking
  reqwest client the exporter uses must not be created or dropped inside an async context (*assumed* from the
  reqwest-blocking documentation's warning; not provoked). The spike builds the tracer provider in plain `main`,
  then builds the tokio runtime explicitly and `block_on`s the server, then drops the runtime and calls
  `provider.shutdown()`. Run with no endpoint: exit 0. Run with `OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:1`
  (nothing listening): the batch exporter retries, and the process still exits 0 — an unreachable collector is a
  warning, never a crash, which is Go's rule.
- **Dependencies log through the same subscriber.** With no filter the spike printed `hyper_util`'s connection
  attempts at TRACE/DEBUG. So `LOG_LEVEL` filters *this service's* records at the level asked and everything else
  at `warn` (a `Targets` filter), or a `LOG_LEVEL=debug` run drowns in the exporter's connection pool.
- **`https://` endpoints need TLS in the exporter's client.** Without `reqwest-rustls` the lock carries no TLS
  crate at all (the spike's lock), so an `https://` collector — which `config` accepts, as Go's does — could not be
  reached. With it, `rustls` comes with `aws-lc-rs`/`aws-lc-sys`, and `aws-lc-sys` 0.45.0 compiled here with
  `gcc` alone: there is no `cmake` and no `clang` on this machine (`which`). The `rust:1.98-bookworm` image CI
  and Compose use carries gcc (*assumed*: Debian `buildpack-deps` lineage); the matrix's native gate and
  `make demo` are the runs that prove it there.
- **Spans carry the request.** `tracing_subscriber::fmt::layer().json().with_current_span(true)` printed the
  enclosing span on a log line written inside it. The OpenTelemetry trace id is not a `tracing` field by itself;
  it is recorded onto the request span as a field (`trace_id`) once the span's parent is set from `traceparent`,
  so a log line carries it the way Go's `TracingHandler` adds `trace_id` (*assumed* shape; the observability
  rule's test is what proves it).

## Facts the plan leans on that are still *assumed*

- **An axum `Router` cannot list its routes** — no public iterator over registered paths (*assumed* from the
  shape of `Router`'s API; D4 already decided the consequence). So the OpenAPI test reads the adapter's own source
  for its route registrations the way Go's reads `mux.HandleFunc("<METHOD> <path>"`, with plain string scanning —
  no `regex` crate and no YAML crate added for a test (D4) — and asserts a minimum count so a registration
  written another way cannot slip past unseen.
- **rustc warns where Go does not.** An assignment never read (`unused_assignments`), a binding shadowed before
  it is read (`unused_variables`) and code after a `return` (`unreachable_code`) are warn-by-default, and the gate
  runs clippy with `-D warnings`. Go's entry point writes the in-memory store, then overwrites it inside the chosen
  store's region; copied literally into Rust, that fails lint in the generated file or in the pruned one. The
  entry point's store wiring has to be warning-free in both states — generated with each store, and after
  `./init --event-store memory` — and the prune test and the matrix are what prove it.
- **`EventStore` is not object-safe** (`events.rs`: its methods return `impl Future`), so the readiness probe the
  adapter declares cannot be `dyn EventStore`. The adapter declares its own object-safe probe and the entry point
  adapts the opened store to it — the Rust spelling of Go declaring `ReadinessProbe` structurally, so the
  adapter imports no port and builds on the standard profile, where there is none.

## CORS: hand-written, as Go's is

`tower-http` 0.7.1 has a `CorsLayer`, and it is not the parity answer: it answers a preflight with 200 rather
than the 204 Go's `security.go` sends, and adds `Vary` for request headers beyond `Origin` (*assumed* from the
crate's documented defaults; not run). Go's wrapper is eighteen lines whose every behaviour is a case in
`http_security_test.go`; the Rust one is the same function as an axum middleware (`axum::middleware::from_fn`),
held to the same cases, and costs no crate.

## The locks, today and after

Today `cargo.lock_variant` names four committed locks (`memory`, `memory-sqlite`, `memory-postgres`,
`memory-sqlite-postgres`) and writes an empty lock when no service depends on anything. With the transport the
variants are the union of stores × {axum, no transport}; no store and no transport stays the empty lock, so five
locks are new: `axum`, `memory-axum`, `memory-sqlite-axum`, `memory-postgres-axum`,
`memory-sqlite-postgres-axum`. `scripts/regenerate-locks.py` resolves each with `cargo generate-lockfile` in a
throwaway one-member workspace, as it does today; `make locks` needs cargo and the network, generation needs
neither.
