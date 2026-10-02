# Adversary log — Rust answers every axis and target

## http-axum · 7a706c9 · 2026-10-02

| Trigger | Status | Evidence |
|---|---|---|
| driving adapter (HTTP route, CLI command, queue consumer) | widened | `assets/backing-services/rust/http_app.rs`, `http_security.rs`, `serve_main.rs` (a generated HTTP service); `catalog.json`, `src/slipwai/prune.py` via `assets/backing-services/prune.py` (`--http axum`, `./init --http none`) |
| driven adapter or the provider types behind one | widened | `assets/backing-services/rust/http_app.rs` (the readiness probe asking the event store), `observability.rs` (the OTLP exporter) |
| authorisation decision (who can reach one that already exists) | not present | no auth answer is offered for Rust in this slice (`catalog.json`); CORS is an origin policy, attacked under the driving adapter |
| concurrency, idempotency, ordering, retention, or time | widened | `serve_main.rs` (graceful shutdown on signal), `observability.rs` (a span per request, `traceparent` continuation) |

Spawned: runtime (the generated service over HTTP, ports 3100/3101) · `drive-adversary` · claude-opus-5-5 (host) · delegated, fresh context · manifest: `assets/backing-services/rust/{http_app,http_security,observability,config,serve_main,http_openapi,driving_mod}.rs`, `openapi.yaml`, `tests/test_rust_http_{service,entry}.py`
Spawned: factory (generate, prune, locks, migrate, add-service for Rust `--http`) · `drive-adversary` · claude-opus-5-5 (host) · delegated, fresh context · manifest: `catalog.json`, `src/slipwai/{add_service,backends,prune}.py`, `src/slipwai/project/{composition,openapi,rules,run_skill,rust_entry,rust_layouts}.py`, `src/slipwai/project/languages/{cargo,rust}.py`, `assets/backing-services/prune.py`, `assets/languages/rust/{app/Cargo.toml,locks/}`, `scripts/regenerate-locks.py`, `tests/test_rust_http_{locks,none,prune,names,baseline}.py`, `tests/test_pruning.py`, `tests/test_matrix.py`
Omitted: none
Findings:

| # | Severity | Finding | Triage | State |
|---|---|---|---|---|
| F1 | HIGH | A project named after a crate the manifest declares (`axum`, `tower`, `tracing`, `opentelemetry`, `tokio`, `serde`), a keyword (`self`, `type`, `crate`) or `core` generates and does not compile (`rust.py` `crate_name` guards only `test`/`std`/`alloc`/`proc_macro`); `tokio`/`core` already failed at the base through `migrate.rs` | confirmed | open |
| R1 | MEDIUM | SIGTERM with one silent or half-open connection never ends the process: `with_graceful_shutdown` has no deadline (`serve_main.rs`); Go's `Shutdown` is bounded at 10s (`serve_main.go`) | confirmed | open |
| R2 | MEDIUM | No header-read timeout: a client sending part of a request line holds the connection indefinitely (slowloris); Go sets `ReadHeaderTimeout: 5s` | confirmed | open |
| R3 | LOW | The request method goes verbatim into the span name and `http.request.method` (`observability.rs`); a 3000-byte method is exported as written. OpenTelemetry's HTTP conventions map an unknown method to `_OTHER` | confirmed | open |
| F2 | MEDIUM | `scripts/backing-services.py --http <x>` applies one answer to every service of a mixed project, Go's included, and the next `migrate` installs a transport into a worker; present at the base for TypeScript+Go | confirmed, predates the slice (D10) | open |
| F3 | LOW | `--http none` offline with an empty cargo cache leaves a stale lock (`_relock_rust` runs `cargo metadata`); the same path at the base when a store is removed | confirmed, predates the slice (D10) | open |
| F4 | LOW | `--http none` leaves `tokio` `net`/`signal`, `serde_json` and an empty `[dev-dependencies]` | duplicate of T025, T034 | open |
