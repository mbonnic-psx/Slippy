MINOR

**Rust answers the HTTP axis: `--http axum`, and a Rust service now serves.** A Rust service gets what a Go one
gets from `net-http`, written in Rust's idiom. `/health` is liveness and `/ready` asks the event store through
the port, and anything else is a JSON 404 that says nothing about the path or the method. A body that does not
parse is one 400 that names the field and the rule and never the value. The browser hardening headers and a
hand-written CORS wrapper sit in front of every route, with `Vary: Origin` whenever an `Origin` came and a
preflight answered before any route. One OpenTelemetry span is opened per request, continuing a `traceparent`,
and its trace id is on every log line written inside it. Spans are exported only when
`OTEL_EXPORTER_OTLP_ENDPOINT` names somewhere, so a collector that is down is a warning and never a crash.
`HOST`, `PORT`, `PUBLIC_BASE_URL`, `CORS_ALLOWED_ORIGINS`, `LOG_LEVEL`, `LOG_FORMAT` and the OpenTelemetry
variables are read once and checked before anything binds, and a value that cannot be used stops the process
with the variable named. `openapi.yaml` is hand-written beside the service and a test holds it to the routes
the router registers. `src/bin/serve.rs` binds the port, so `make dev`, the Compose `service` and `make demo`
now exist for Rust, and the generated run page says two Rust services' dev servers share one `target/`.
Each router rule is a `#[cfg(test)]` module dispatched through `Router::oneshot`, with no socket.

**A new Rust project is now generated with this by default; one generated before keeps what it recorded.**
`axum` is Rust's default answer, as every backend's own transport is its default. A project generated before
this release recorded no `http` answer, and an unasked axis reads as `none`, so `slipwai migrate` carries it
forward with no transport rather than adding one. `--http none` generates what Rust generated before — only the recorded answer, the shared pruner script and a re-resolved lock differ —
and `./init --http none` takes the transport away again — its files, its crates and the Compose service —
leaving the lock to follow the manifest. The manifest's `tokio`, `serde` and `serde_json` stay declared after
that, because the event store needs them too. Five more `Cargo.lock` files are committed for the union of
stores and transport across a workspace, so `--locked` builds offline.

**Catch-up.** Nothing is asked of an existing project's code: this only adds what a new Rust service is generated
with. `slipwai migrate` does move a Rust project's `Cargo.lock`, to the newer transitive patch versions the
committed locks were re-resolved to; where that conflicts with a lock you changed, re-lock (`cargo update`, then
commit `Cargo.lock`) and take the result. To give an existing Rust service a transport, answer `--http axum` on a
fresh generation and carry its files over.
