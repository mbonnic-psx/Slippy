# Implementation Plan: http-axum — a Rust service serves HTTP through axum

**Branch**: `003-rust-every-axis-http-axum` (the claim is `slice/http-axum`; the work lives on the feature-named
branch for the reason `specs/001-rust-cargo-adopt/decisions.md` D17 gives) | **Date**: 2026-10-02 |
**Spec**: [spec.md](../../spec.md) · slice 1 of [story-split.md](../../story-split.md)

**Input**: US1 scenarios 1–12; the `http-axum` paragraph under *Gaps reviewed*; FR-001 (http only), FR-002,
FR-006, FR-007 (the parts that paragraph assigns this slice), FR-008; SC-003; the edge cases the split names
(mixed-backend project, `--http none` beside a transport, the committed locks, a project generated before this
feature). Standing decisions D1–D7 in [decisions.md](../../decisions.md). What the crates do is in
[research.md](research.md), each fact cited to a run or marked *assumed*. The parity bar is Go's `net-http`
(`assets/backing-services/go/http_*.go`, `config.go`, `tracing.go`, `serve_main.go`, `openapi.yaml`) and
[docs/backend-obligations.md](../../../../docs/backend-obligations.md); the procedure is
`.claude/skills/add-backing-service/`.

## Summary

Rust is asked the HTTP question like every other backend: `none` or `axum`, default `axum` (D2). With `axum` a
Rust service gets what `net-http` gives a Go one, in Rust's idiom: the driving adapter with `/health`, `/ready`
and a JSON 404, the 400 body for a request that does not parse, the browser hardening, one span per request, a
checked environment, a hand-written `openapi.yaml` a test holds to the router (D4), and `src/bin/serve.rs`, which
binds the port — so `make dev`, the Compose `service` and `make demo` exist for Rust. `--http none` generates
exactly what Rust generates today. The manifest gains the transport's crates in a marked region, `tokio` stays one
key, and the committed locks double to cover {no store, memory, memory+sqlite, memory+postgres,
memory+sqlite+postgres} × {axum, none}. Nothing about the other five backends changes.

## Release constraint

**Held behind the pre-release, as standing decision D7 says for this slice.** A merge to `main` publishes a
`1.4.0.dev<N>` snapshot, which installers pass over unless asked; a person merges the pull request and a person
runs `make release`. There is no flag file: someone generating with a released slipwai never meets the Rust
transport until that release, and the four later slices land behind the same gate. Inside a snapshot the change
is coherent the moment it lands: every combination it offers builds, verifies and runs, and `--http none` keeps
today's tree.

## Technical Context

**Language/Version**: the factory is Python ≥ 3.11 (run here on 3.14); the generated service is Rust 1.98.1
(`RUST_TOOLCHAIN`), edition 2024.

**Primary Dependencies**: none added to the factory. In a generated Rust service: `axum` 0.8.9, `tracing` 0.1.44,
`tracing-subscriber` 0.3.23, `opentelemetry`/`opentelemetry_sdk`/`opentelemetry-otlp` 0.33.0,
`tracing-opentelemetry` 0.34.0, with `tokio`, `serde`, `serde_json` at the versions the event store already pins;
dev-dependencies `tower` 0.5.3 and `http-body-util` 0.1.5 (research).

**Storage**: none new. The event store's three answers are opened by the entry point (R8).

**Testing**: the factory's suite (`make test TESTS=…`, unittest under pytest), with fakes only — no mocking
library (`AGENTS.md`). A new suite `tests/test_rust_http.py` holds the slice's factory-level rules; the generated
service's own rules are `#[cfg(test)]` modules in the assets, run by its `make verify` (`cargo llvm-cov`, floor
70%, `src/bin/` excluded as it is today). Every run inside the 4G unit with `TMPDIR=$HOME/.cache/slippy-cruise-tmp`,
cargo with `CARGO_BUILD_JOBS=2`, the suite with `env -u CRUISE_RUNNER -u CRUISE_ITERATION` (001's D8).

**Target Platform / Project Type**: the `slipwai` CLI generating a repository; the generated service runs on
Linux and macOS hosts and in `rust:1.98-bookworm` (Compose, CI).

**Performance Goals / Constraints**: SC-003 — `make demo` reports the service healthy inside the Compose
healthcheck's window (`start_period` 20s, 60 retries at 5s) on a first container build. Generation needs no
network (committed locks). `src/slipwai/project/entry_stores.py` is at the 350-line module budget
(`scripts/check-structure.py`), so Rust's entry-store row lives in a module of its own.

## Constitution Check

- **I. What a project was given keeps meaning what it meant** — holds. A Rust project generated before this slice
  records no `http` key, and `Selection.option` reads an unasked axis as its `absent` (`none`), never the catalog
  default (D2's premise; R2 pins it through `slipwai migrate`). `--http none` generates today's tree byte for
  byte, `project.json`'s `http: none` aside (R2). No other backend's tree changes (`make starters` diff, R12).
- **II. Re-running is safe** — holds. `./init --http none` subtracts only (R10); generation stays offline.
- **III. Simplicity** — holds. One new feature, `axum`, answered with rows in tables keyed by feature or backend —
  no branch on the option's name (`test_an_option_s_feature_is_never_branched_on_by_name`). No crate for CORS, for
  YAML or for regex.
- **V. Tests at the boundary, fakes only** — the factory's rules enter through `write_project`/`slipwai generate`;
  the service's through `Router::oneshot` (no socket) and `config::load_from(lookup)` (an environment handed in).
- **VIII. Versioning** — a new axis option is MINOR; `VERSION` is already `1.4.0.dev0`, which a MINOR over 1.3.x
  needs, so it does not move; the fragment claims MINOR (R13).
- Security (the parts the constitution names): no path, method or value in a 404 or 400 body; a store's error is
  logged and never sent; no CORS wildcard; `Vary: Origin` always where an `Origin` came (R3, R4).

## Pin

The Rust backend was added on 2026-09-23, before the method was adopted (2026-09-28), so its generator is code
that was here. The slice changes two of its behaviours and must not change a third:

1. **A Rust service generated with no `--http` answer gets no transport today** — no `src/bin/serve.rs`, no
   `src/adapters/driving/`, no `.env.example`, no Compose `service`, no `make dev`, and a `project.json`
   selection with no `http` key (observed by hand 2026-10-02: `slipwai generate rust-now --language rust
   --profile standard` → `selection: {}`). D2 changes this on purpose: afterwards no answer means `axum`, and
   `--http none` is this tree. T001 characterises it green before any change, on both profiles; R2 then holds
   `--http none` to it.
2. **A Rust deployable whose record has no `http` key is carried forward with no transport** — characterised by
   T001 through `slipwai migrate` on a generated Rust project, observed green before any change; it must stay
   green (the edge case; D2's *would reverse if*).
3. **Every other backend generates as today** — pinned already by the suite (`tests/test_matrix.py`,
   `tests/test_monorepos.py`); the `make starters` diff in R12 is the proof for the whole tree.

`delivery/survey/pinned.md` gains rows 1 and 2 in the commit that records this plan, with the tests T001 names and
`make test TESTS=test_rust_http` to run them.

## Rules (the example map — each one RED-GREEN-REFACTOR cycle)

There is no event model, so the rules are numbered here and the tasks implement them in order. Each names the
scenarios it closes and the examples that drive it.

- **R1 — Rust is asked the HTTP question** (scenario 1; FR-001 http; D2). `catalog.json`: `rust` under
  `http.options.none.backends`; a new option `axum` (`capabilities: ["http-axum"]`, `features: ["axum"]`,
  `backends: ["rust"]`, `targets` as `net-http`'s, no containers, no migrations, no integration suite, a label
  saying what it is and what it does not prove); `default.http.rust = "axum"`. `prune.py`: `FEATURES`, the `AXES`
  `http` option, mirrored. Examples: `axis_options("http", "rust", "none") == ["none", "axum"]`; the default for
  Rust is `axum`; `--http axum` and `--http none` both generate; the interactive prompt reads
  `Choose (none/axum) [axum]`; `--http net-http --language rust` is refused as every other backend's wrong
  transport is.
- **R2 — `--http none` is today's tree** (scenarios 2, 9; edge case *generated before*). Examples: on each profile,
  `--http none` generates the file set T001 recorded and the same bytes as generation at `40dacad`
  (`make starters`/a scratch generation at the base, diffed once and recorded in the convergence verdict), except
  `project.json`'s `http: none`; a factory test generates it and runs its `make verify` (cargo); a record with no
  `http` key migrates with no transport (T001's second characterisation, still green).
- **R3 — The adapter's routes** (scenario 3). `src/adapters/driving/http/mod.rs`: `build_app(registrars)` returns
  the `Router`; `GET /health` → 200 `{"status":"ok"}` (`HEALTH_BODIES['rust']`); a `readiness(probe)` registrar
  for `GET /ready` → 200 `{"status":"ready"}` with no probe, 200 when the probe answers, 503
  `{"status":"unready","reason":"eventStore"}` when it fails, `Cache-Control: no-store` either way, the failure
  logged and never in the body; the fallback → 404 `{"error":"notFound"}` saying nothing about path or method, and
  the same for a known path under the wrong method; a JSON-body extractor whose rejection is the one 400 body
  `{"error":"schemaValidationFailed","field":…,"message":…}` naming the field and the rule and never the value
  (unknown field, wrong type, not one JSON value). The probe is an object-safe trait the adapter declares; the
  adapter imports no port (research). Edge tests dispatch through the real router with `oneshot`, no socket.
- **R4 — What a browser meets first** (scenario 3). `src/adapters/driving/http/security.rs`: `secure(router,
  allowed_origins)` sets `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy:
  no-referrer`, `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'` on every response, no HSTS;
  CORS hand-written as Go's (research): `Vary: Origin` whenever an `Origin` came, the origin echoed with
  `Access-Control-Allow-Credentials: true` only when allowed, a preflight from an allowed origin answered 204 before
  any route with the methods, the requested headers (or `content-type`) and `Max-Age: 600`; an unknown origin gets
  no allow header; an empty list is same-origin only. Examples are Go's `http_security_test.go` cases.
- **R5 — The checked environment** (scenario 11). `src/config.rs`: one `Config` read by `load_from(lookup)` and
  `load()` before anything binds — `HOST` (default `0.0.0.0`), `PORT` (3000; refused unless 1–65535),
  `PUBLIC_BASE_URL` (refused unless `http://`/`https://`), `LOG_LEVEL`, `LOG_FORMAT` (read, never refused),
  `OTEL_SERVICE_NAME` (defaults to the service's own name, the way Go's `__SERVICE_NAME__` does),
  `OTEL_EXPORTER_OTLP_ENDPOINT` (refused unless `http://`/`https://`), `CORS_ALLOWED_ORIGINS` (comma-split, blanks
  dropped); `EVENT_STORE_PATH` in the `sqlite` region and `DATABASE_URL` (refused unless `postgres://` or
  `postgresql://`) in the `postgres` region; `address()` brackets an IPv6 host; `reported_url()` prefers
  `PUBLIC_BASE_URL`. The refusal names the variable. `.env.example` gets the transport's keys through the existing
  `__TRANSPORT__` region (`ENV_FEATURES` gains `axum`), the store's in theirs.
- **R6 — One span per request** (scenario 3). `src/observability.rs`: the provider records spans always and
  exports only when an endpoint was named, built before the runtime (research); a layer that opens one span per
  request named for its method and matched route, continuing an incoming `traceparent` and recording `trace_id` on
  the span so every log line written inside it carries it; `LOG_LEVEL` filters this service's records and
  dependencies at `warn`; `LOG_FORMAT=pretty` for a terminal, JSON otherwise, an unknown level read as `info`; a
  failing shutdown is reported and never fatal. Examples: a request with a `traceparent` yields a span whose
  trace id is the caller's; a request without one yields a fresh one; a failing flush is reported, not raised.
- **R7 — The published contract** (scenario 12; D4). `openapi.yaml` beside the service's `Cargo.toml`,
  hand-written in the shape of Go's: `/health`, `/ready` (200 and 503), the `Health`, `Ready`, `Unready`,
  `SchemaFailure`, `NotFound` schemas — and no `/api/flags`, which arrives with `aws` (*Gaps reviewed*). A
  `#[cfg(test)]` module reads the adapter's source for every route it registers and fails when the document does
  not describe that path and method, with a minimum count so a stale pattern cannot pass silently. No exporter, no
  `EXPORTERS` row, no `check-openapi` (D4). `DOCUMENTS['axum'] = "openapi.yaml"`, `API_CONTRACTS['axum']`.
- **R8 — The entry point, and running it** (scenario 4; SC-003; D5). `src/bin/serve.rs`, the only file with no
  test (coverage already ignores `src/bin/`): logging first, `config::load()` (a refusal is logged and exits 1),
  the tracer provider, then the runtime; the event store opened once, lazily (Postgres through
  `PgPoolOptions::connect_lazy` + `PostgresEventStore::from_pool`, so nothing connects while the process starts),
  adapted to the probe and handed to `readiness`; the router wrapped by `observability` and `security` outside
  `build_app`, as Go's are outside `BuildApp`; bound on `config.address()`, the reported URL logged; graceful
  shutdown on SIGTERM and Ctrl-C, then the provider's shutdown. The store wiring is a new `rust` row of
  `ENTRY_STORES` (in `project/rust_entry.py`, merged where `composition.py` reads the table), with the
  `__STORE_IMPORT__`, `__STORE_OPEN__` and `__STORE_ARGUMENT__` placeholders; no flag placeholder (slice 4), so
  `ENTRY_WIRING` gains no row. Warning-free generated and pruned (research). Proven by running: `make dev` on the
  host answers `GET /health` and `GET /ready` (`HEALTH_BODIES`, `READY_PATHS` already say `/ready` and
  `{"status":"ok"}` for Rust), and `make demo` reports the service healthy inside the window. The generated
  `skills/run-the-app/SKILL.md` says, where a project has two Rust services with a transport, that their dev
  servers share one `target/` and run one at a time on the host, or together through `make demo` (D5).
- **R9 — The manifest and the locks** (scenarios 5, 7, 10; FR-006). `cargo.py`: the crates the transport alone
  needs inside `# backing-service:axum:begin`/`:end` of `[dependencies]`, its two dev-dependencies in an `axum`
  region of their own under `[dev-dependencies]`; `tokio`, `serde` and `serde_json` unmarked whenever the store or
  the transport is present, `tokio` once with the union of their features (`net`, `signal` added by axum), never a
  second key. `direct_crates` names dependencies and dev-dependencies, so the member entry is what Cargo writes.
  `lock_variant` returns `axum`, `memory-axum`, `memory-sqlite-axum`, `memory-postgres-axum`,
  `memory-sqlite-postgres-axum` beside today's four, the empty lock unchanged; `scripts/regenerate-locks.py` makes
  all nine (`make locks`), and `make check-locks` is clean before the PR (no CI job runs it). Examples: every
  variant's lock lists every crate its manifest names directly (a factory test, no cargo); a two-service workspace,
  one on axum and one not, takes the union lock with each member listing only its own crates; the matrix's native
  gate builds `--locked` with each store.
- **R10 — Taking the transport away** (scenario 8). `prune.py`: `OWNED_FILES['axum']['rust']` =
  `src/adapters/driving`, `src/config.rs`, `src/observability.rs`, `src/bin/serve.rs`, `openapi.yaml`, with
  `any: packages/api-client` as every transport has; `OWNED_FILES_PER_WEB_APP['axum']` = `src/routes`,
  `tests/routes`; `APP_SERVICE_FEATURES` gains `axum`; `PACKAGE_EDITS['rust']['axum']` names the transport's crates
  so `_relock_rust` runs `cargo metadata` and drops them offline; `MARKED_FILES_BY_LANGUAGE['rust']` gains
  `src/lib.rs`, `src/adapters/mod.rs`, `src/bin/serve.rs` and `src/config.rs`. `declare_modules` writes the modules
  only the transport adds (`config`, `observability`) inside an `axum` region of `src/lib.rs`; `src/adapters/mod.rs`
  is written from the files present, `pub mod driven;` where the store put it there and `pub mod driving;` inside
  an `axum` region — so after the prune it declares what is left, possibly nothing, which compiles, and `pub mod
  adapters;` stays unmarked because the store shares it. Examples: on each profile, `./init --http none` leaves a
  project with no transport file, no axum crate in `Cargo.toml` or `Cargo.lock`, no Compose `service`, and a green
  `make verify` (cargo); the pruner refuses nothing new (auth and users are not offered to Rust yet).
- **R11 — A browser app beside a Rust service** (scenario 6). With `react-vite`, `packages/api-client` is generated
  from the Rust service's `openapi.yaml`, `apps/web/vite.config.ts` proxies `/api` inside the `axum` region, the
  route that shows the API answering arrives, and the generated `make verify` passes — the generic machinery,
  reached by `DOCUMENTS['axum']` and the per-web-app rows; the matrix's `event-modelling`/`react-vite` row is the
  native proof.
- **R12 — The factory holds Rust to the transport rows** (FR-007 for this slice; *Gaps reviewed*). `tests/`:
  `TRANSPORTS['rust'] = 'axum'`; `PARTIAL['rust'] = {"event-store", "http"}` — *answers* those two now — and every
  test reading it follows; the matrix's maximal row gives each backend only the `auth`/`users` answers it is
  offered (Rust: `none`, until `users` lands); `test_catalog`'s default-project checks take Rust's `auth`/`users`
  as `none`; `test_readiness` reads `.rs` sources; `test_running` has a Rust row (`axum`,
  `apps/service/src/bin/serve.rs`, `cargo run --locked --bin serve`); the multi-service case (a Rust `axum` service
  beside a Go `net-http` one: two Compose services, two ports, two `make dev` entries) is asserted at generation.
  Docs: `docs/axes.md`'s `--http` row (with `axum` among the per-backend defaults, D2), its "every backend but Rust"
  sentence, Rust's coverage row and the "what arrives" row with its dependency column; `README.md`'s `--http` list;
  `docs/backend-obligations.md` only if a table row has to be named. The proof that nothing else moved: `make
  starters` before and after, every non-Rust tree identical, recorded in the verdict.
- **R13 — The changelog** (FR-008; D2; D6). A new fragment `changelog.d/rust-http-axum.md`, `MINOR`: what `axum`
  gives a Rust service, that a new Rust project now gets an HTTP service by default and one generated before keeps
  what it recorded, and the Catch-up (nothing for an existing project). D6: the sentences in
  `changelog.d/rust-backend.md` ("no transport") and `rust-event-store.md` ("Rust still answers no transport …;
  axum comes next") that would be false at release are corrected in place.

## Design

`src/slipwai/`:

| Where | What changes |
|---|---|
| `catalog.json` | R1 |
| `assets/backing-services/prune.py` | R1 (`FEATURES`, `AXES`), R10 (`OWNED_FILES`, `OWNED_FILES_PER_WEB_APP`, `APP_SERVICE_FEATURES`, `PACKAGE_EDITS['rust']`, `MARKED_FILES_BY_LANGUAGE['rust']`) |
| `backends.py` | `ENV_FEATURES` gains `axum` (`dev_command`, `COMPOSE_CACHES`, `BACKEND_TOOLING` already answer Rust) |
| `project/rust_layouts.py` | `RUST_WRITE_SIDE['axum']`: the files below, destination → asset |
| `project/rust_entry.py` (new) | Rust's `EntryStore`; `ENTRY_STORES` is extended with it where it is read |
| `project/languages/cargo.py` | R9 |
| `project/languages/rust.py` | `declare_modules` marks transport-only modules; `src/adapters/mod.rs` written from the files present; `__SERVICE_NAME__` resolved in `config.rs` |
| `project/openapi.py`, `project/rules.py` | `DOCUMENTS['axum']`, `API_CONTRACTS['axum']` |
| `project/run_skill.py` (or wherever the generated run page says how to run two services) | D5's sentence |
| `scripts/regenerate-locks.py` | R9 |

`assets/backing-services/rust/` (new), and where each lands under the service:

| Asset | Destination |
|---|---|
| `driving_mod.rs` | `src/adapters/driving/mod.rs` |
| `http_app.rs` | `src/adapters/driving/http/mod.rs` |
| `http_security.rs` | `src/adapters/driving/http/security.rs` |
| `http_openapi.rs` | `src/adapters/driving/http/openapi.rs` (`#[cfg(test)]`) |
| `config.rs` | `src/config.rs` |
| `observability.rs` | `src/observability.rs` |
| `serve_main.rs` | `src/bin/serve.rs` |
| `openapi.yaml` | `openapi.yaml` |

`assets/languages/rust/app/Cargo.toml` gains the `[dev-dependencies]` placeholder; `assets/backing-services/rust/
adapters_mod.rs` becomes the file `rust.py` fills from what is present (or is replaced by that computation).
`assets/languages/rust/locks/` gains the five `*-axum` directories.

## Project Structure

### Documentation (this slice)

```text
specs/003-rust-every-axis/slices/http-axum/
├── plan.md        # this file
├── research.md    # the crates and the spike, each fact cited
├── quickstart.md  # how to see it run
├── tasks.md       # drive-tasks
└── benchmark.json
```

There is no `data-model.md` or `contracts/`: the slice adds no stored entity, and the one published contract is the
generated service's `openapi.yaml`, which R7 writes and tests.

### Source Code

```text
catalog.json
assets/backing-services/prune.py
assets/backing-services/rust/{driving_mod,http_app,http_security,http_openapi,config,observability,serve_main}.rs
assets/backing-services/rust/openapi.yaml
assets/backing-services/rust/adapters_mod.rs
assets/languages/rust/app/Cargo.toml
assets/languages/rust/locks/{axum,memory-axum,memory-sqlite-axum,memory-postgres-axum,memory-sqlite-postgres-axum}/Cargo.lock
src/slipwai/backends.py
src/slipwai/project/{rust_layouts,rust_entry,composition,openapi,rules,run_skill}.py
src/slipwai/project/languages/{rust,cargo}.py
scripts/regenerate-locks.py
tests/test_rust_http.py      # new: T001's pin, R1, R2, R9, R10, R12's generation checks
tests/test_catalog.py, tests/test_matrix.py, tests/test_readiness.py, tests/test_running.py (+ whatever the sweep finds)
docs/axes.md, README.md, docs/backend-obligations.md (only if a row must be named)
changelog.d/rust-http-axum.md (new), changelog.d/rust-backend.md, changelog.d/rust-event-store.md
delivery/survey/pinned.md    # rows 1 and 2 of the Pin, appended
```

**Structure Decision**: the one deployable, `slipwai`, at the root — the only service `project.json` records, and
its purpose ("generates and adopts repositories") covers what a generated Rust service is given. One vocabulary
(axis, option, feature, transport, backend, service, region, lock), so one bounded context; strategy `leave-it`
(D1), so the change lands in the existing modules, with `project/rust_entry.py` new only because
`entry_stores.py` is at its line budget. A new suite `tests/test_rust_http.py` for the slice's factory-level rules,
so the existing suites stay inside the 350-line budget.

## Not working yet (deliberate, owned by later slices)

- No `--auth` or `--users` answer but `none` for Rust — slices `auth` and `users`; `PARTIAL['rust']` still names
  them.
- No `/api/flags`, no `ENTRY_WIRING['axum']`, no flagged document, no image, no `aws`/`azure` target — slice
  `aws` (and `azure`).
- Two Rust services' `serve` binaries share a name and a `target/` (D5): run their dev servers one at a time on
  the host, or together through `make demo`.
- After `./init --http none` on the standard profile, `tokio`, `serde` and `serde_json` stay declared (they are not
  the transport's alone) and `src/adapters/mod.rs` may declare nothing.

## Complexity Tracking

None. The one new module exists for the line budget, not for a new abstraction.

## Blocked

None. The two questions convergence pass 1 handed back (T019, T021) were decided by `/cruise` as D8 and D9 in
`specs/003-rust-every-axis/decisions.md`: scenario 9 amended to name its three differences; `serde_path_to_error` kept.
