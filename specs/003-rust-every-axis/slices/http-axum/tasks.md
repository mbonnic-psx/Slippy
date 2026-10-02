---

description: "Tasks for slice http-axum: a Rust service serves HTTP through axum"
---

# Tasks: http-axum — a Rust service serves HTTP through axum

**Input**: `specs/003-rust-every-axis/slices/http-axum/plan.md` (Rules R1–R13, Pin, Design, Source Code, Not working
yet), `research.md`, `quickstart.md`, `specs/003-rust-every-axis/spec.md` (US1 scenarios 1–12; the `http-axum`
paragraph under *Gaps reviewed*; SC-003), `decisions.md` D1–D7. There is no `data-model.md` or `contracts/`: the slice
stores no entity, and its one published contract is the generated service's `openapi.yaml` (R7).

**Cycle**: one task per rule of the plan's example map, each its own RED-GREEN-REFACTOR, one local commit per task,
never pushed. Every factory-level test enters at the boundary (`write_project` / `slipwai generate` /
`slipwai migrate` into a scratch directory under `$TMPDIR`) and uses no mocking library (`AGENTS.md`): fakes are
classes or functions in the test tree. The generated service's own rules are `#[cfg(test)]` modules in the assets,
dispatched through `Router::oneshot` (no socket) and `config::load_from(lookup)` (an environment handed in), and run
by a generated project's `cargo test`.

**Format**: `[ID] [P?] [Story] Description` — `[P]` only where the task's files are disjoint from its siblings' and no
RED depends on another task's behaviour.

**Test command** (every task; from the worktree root):
`env -u CRUISE_RUNNER -u CRUISE_ITERATION make test TESTS="test_rust_http"` — narrow it to the one class the task
adds (`TESTS="test_rust_http.<Class>"` where the runner accepts it). The suite `tests/test_rust_http.py` is new and
holds the slice's factory-level rules; `scripts/check-structure.py`'s 350-line module budget applies to it, so when
it nears the budget split it by rule family (`tests/test_rust_http_<family>.py`) and keep the same prefix so
`TESTS="test_rust_http"` still selects all of them.

## HARD SAFETY RULES (every implementer reads these; follow them for every command you run)

- The session is already inside a 4G-capped systemd unit. Do **not** wrap commands in `systemd-run` again.
- `TMPDIR=$HOME/.cache/slippy-cruise-tmp` (on disk) stays set. Never use `/tmp`: it is a RAM-backed tmpfs. Probe
  projects, scratch generations and cargo target directories go under `$HOME/.cache/slippy-cruise-tmp/`, and big
  scratch directories are cleaned up when the task is done.
- Cargo builds of generated projects run with `CARGO_BUILD_JOBS=2` (and `CARGO_TARGET_DIR` under the cache
  directory, shared across a task's probes).
- Every test run and `make verify` runs under `env -u CRUISE_RUNNER -u CRUISE_ITERATION` (001's D8).
- `make verify` takes over half an hour, beyond one foreground tool call. Run it in the background with its output
  to a log under `$HOME/.cache/` (for example `env -u CRUISE_RUNNER -u CRUISE_ITERATION make verify >
  $HOME/.cache/slippy-http-axum-verify.log 2>&1`, with `run_in_background`) and poll the log; never wait in the
  foreground.
- Known, not yours: the local tags `v1.4.0`, `v1.5.0` and `v1.5.1` make
  `tests/test_changelog.py::test_every_release_this_repository_has_ever_tagged_has_an_entry` red in this checkout
  only. Never delete or move those tags. When `tests/test_changelog.py` matters (T014, T015), prove it in a scratch
  `git clone --no-tags` of the worktree under `$HOME/.cache/slippy-cruise-tmp/`. Any other failure is real.
- cargo 1.98 has network here, so `make locks` / `scripts/regenerate-locks.py` may run.
- The one sanctioned way to see a RED on code that exists, or to check a guard has teeth, is
  `delivery/docs/delegated-agent-safety.md`'s: change the production file, run the test, restore with
  `git checkout -- <exact path>`. Never `git stash`, never copy a tracked file aside, never `pkill -f`.
- No mocking library (`AGENTS.md`): no `unittest.mock`, no `mock.patch`; fakes in the test tree only.
- One task per local commit; never push, never alter branches, tags or remotes. End each commit message with the
  attribution line the session gives, and say the version level where the task touches a user-visible tree
  (`AGENTS.md`: MINOR, `VERSION` stays `1.4.0.dev0`).
- Write nothing under `delivery/` (the pin rows are already in `delivery/survey/pinned.md`), and not the root `Makefile`,
  `project.json`, `pyproject.toml`, `VERSION` or anything under `.github/`. Needing one is a stop: report it, do not
  edit it. Edit only the files a task's manifest names; if another file must change, stop and report.

## Layers this slice covers

The catalog and the pruner's tables (T002, T011), the generator's per-feature tables (T003 to T010), the generated
service end to end — driving adapter, hardening, checked environment, tracing, published contract and entry point
(T005 to T010), the manifest and the committed locks (T004), taking the transport away (T011), a browser app beside
the service (T012), the factory's own suite and docs held to the new rows (T013), the changelog (T014) and the gate
(T015). No screen, no stored entity, no new port: the slice is a transport on a generated service.

## Phase 1: Setup — the Pin

- [x] T001 **Pin — characterise today's Rust no-transport tree on both profiles, and a migrate of a record with no
  `http` key, green before any production change** (plan *Pin* rows 1 and 2; scenarios 2 and 9 as the baseline they
  will later hold; the edge case *a project generated before this feature*).
  Files: `tests/test_rust_http.py` (new). No production file is touched. Not a
  RED-GREEN increment: a characterisation is green by design.
  - First run `env -u CRUISE_RUNNER -u CRUISE_ITERATION make test TESTS="test_catalog test_axes test_readiness
    test_running"` and confirm green, so a later red is the slice's and not the tree's.
  - In the new suite, one characterisation per profile (`standard`, `event-modelling`): `--language rust` generated
    with no `--http` answer has no `src/bin/serve.rs`, no `src/adapters/driving/`, no `src/config.rs`, no
    `src/observability.rs`, no `openapi.yaml`, no `.env.example` transport keys, no Compose `service`, no `make dev`
    entry, and `project.json`'s selection has no `http` key. Record the exact file set in the test (a sorted list) so
    T003 can hold `--http none` to it.
  - One characterisation of `slipwai migrate` over a generated Rust project whose `project.json` has no `http` key
    (the record written by today's generator): the migrated tree has no transport and the record is still without an
    `http` answer that means `axum` (`Selection.option` reads an unasked axis as `absent`, `none`).
  - Observe each green. Name each test for what it pins; T003 reuses the file-set one.
  - Rows 1 and 2 of the plan's *Pin* were appended to `delivery/survey/pinned.md` with the plan's commit, naming
    this suite; confirm the test names you choose match what the rows say they pin, and do not edit the rows (the
    ledger is append-only — a correction is a new row, reported back).
  - Run: `make test TESTS="test_rust_http"`.

## Phase 2: User Story 1 — a Rust service serves HTTP through axum (P1)

**Goal**: `slipwai generate --language rust` asks the HTTP question like every backend (`none` or `axum`, default
`axum`); with `axum` the service serves `/health`, `/ready`, a JSON 404 and the browser hardening, traces each
request, reads a checked environment, publishes an `openapi.yaml` a test holds to the router, and binds its port with
`make dev` and `make demo`; `--http none` is today's tree.

**Independent test**: generate a Rust project with `--http axum`, run its `make verify`, `make dev` and request
`GET /health` and `GET /ready`; generate one with `--http none` and see the file set T001 recorded.

- [x] T002 [US1] **Rule R1 — Rust is asked the HTTP question** (scenario 1; FR-001 for `http`; D2). Depends on T001.
  Files: `tests/test_rust_http.py`, `catalog.json`, `assets/backing-services/prune.py`, and
  only those existing suites the catalog change turns red, each edited to the new answer and nothing more (their
  structural flips are T013).
  - RED, in the new suite: `axis_options("http", "rust", "none") == ["none", "axum"]`; the default http answer for
    Rust is `axum`; `--http axum` and `--http none` both generate for `--language rust`; the interactive prompt reads
    `Choose (none/axum) [axum]`; `--http net-http --language rust` is refused as every other backend's wrong
    transport is. Observe it fail on the missing option, not on an import error.
  - GREEN: `catalog.json` — `rust` under `http.options.none.backends`; the `axum` option (`capabilities:
    ["http-axum"]`, `features: ["axum"]`, `backends: ["rust"]`, `targets` as `net-http`'s, no containers, no
    migrations, no integration suite, a label saying what it is and what it does not prove); `default.http.rust =
    "axum"`. `prune.py` — `FEATURES` and the `AXES` `http` option mirrored. Then run the whole `make test` once and
    repair, in this commit, only the existing assertions that read Rust as "no transport" because of the default.
  - Guards: `test_catalog` / `test_axes` parity between `catalog.json` and `prune.py` stay green (FR-004); no branch
    on the option's name (`test_an_option_s_feature_is_never_branched_on_by_name`).
  - REFACTOR: none expected; suite green.
  - Run: `make test TESTS="test_rust_http test_catalog test_axes"`.

- [x] T003 [US1] **Rule R2 — `--http none` is today's tree** (scenarios 2 and 9; edge case *generated before*).
  Depends on T002.
  Files: `tests/test_rust_http.py`, and production files only if the RED exposes a difference (expected: none).
  - RED: on each profile, `--http none` generates exactly the file set T001 recorded and the same bytes as
    generation at the base `40dacad`, except `project.json`'s `http: none` (generate once at the base into a scratch
    worktree under the cache, diff, and record the result for the verdict; the test itself holds the file set and
    the absence of every transport file); a factory test generates `--http none` and runs its `make verify` with
    cargo; a record with no `http` key still migrates with no transport (T001's characterisation, still green).
    The rule's behaviour mostly exists already, so if the RED is green on arrival, fold it into the task that
    produced the behaviour (T002) and say so in the commit; do not leave a test that is born green and proves
    nothing. A difference is the RED: fix it in `src/slipwai/` and name the file in the commit.
  - Run: `make test TESTS="test_rust_http"` (the cargo `make verify` test is the slow one; run it alone with
    `CARGO_BUILD_JOBS=2`).

- [x] T004 [US1] **Rule R9 — the manifest and the locks** (scenarios 5, 7 and 10; FR-006). Depends on T002. Moved
  ahead of R3 on purpose: the assets of T005 to T010 cannot compile in a generated project until the manifest and
  its lock carry axum's crates (see *Dependencies & execution order*).
  Files: `tests/test_rust_http.py`, `src/slipwai/project/languages/cargo.py`,
  `scripts/regenerate-locks.py`, `assets/languages/rust/app/Cargo.toml`,
  `assets/languages/rust/locks/{axum,memory-axum,memory-sqlite-axum,memory-postgres-axum,memory-sqlite-postgres-axum}/Cargo.lock`.
  - RED: `lock_variant` returns `axum`, `memory-axum`, `memory-sqlite-axum`, `memory-postgres-axum`,
    `memory-sqlite-postgres-axum` for the five transport selections beside today's four, and the empty lock
    unchanged for no store and no transport; `tokio` is declared once with the union of the store's and the
    transport's features (`net`, `signal` added by axum), never a second key in a marked region; every variant's
    committed lock lists every crate its manifest names directly, `dependencies` and `dev-dependencies` alike (no
    cargo needed); a two-service workspace, one on axum and one not, takes the union lock with each member listing
    only its own crates. Observe the first fail on the missing variant.
  - GREEN: the transport's crates inside `# backing-service:axum:begin` / `:end` of `[dependencies]` (versions in
    `research.md`: `axum` with `http1, json, tokio, query`; `tracing`; `tracing-subscriber` with `fmt, json, registry,
    std, ansi`; the OpenTelemetry trio with `opentelemetry-otlp` at `default-features = false` and `http-proto,
    reqwest-blocking-client, reqwest-rustls, trace`; `tracing-opentelemetry`), its two dev-dependencies in an `axum`
    region of their own under a `[dev-dependencies]` placeholder; `tokio`, `serde`, `serde_json` unmarked whenever
    the store or the transport is present; `direct_crates` naming dependencies and dev-dependencies;
    `lock_variant`; `scripts/regenerate-locks.py` makes all nine; run `make locks` to write the five new locks.
  - Guards: `make check-locks` is clean; the matrix's native gate builds `--locked` with each store (run one
    combination by hand: generate `--http axum --event-store sqlite`, `cargo build --locked --all-targets` with
    `CARGO_BUILD_JOBS=2`).
  - REFACTOR: none expected; suite green. Clean up scratch generations.
  - Run: `make test TESTS="test_rust_http test_cargo test_rust_locks"` (use the names the suite actually has;
    `ls tests | grep -i "cargo\|lock"`).

- [x] T005 [US1] **Rule R3 — the adapter's routes** (scenario 3). Depends on T004.
  Files: `tests/test_rust_http.py`, `assets/backing-services/rust/driving_mod.rs`,
  `assets/backing-services/rust/http_app.rs`, `src/slipwai/project/rust_layouts.py`,
  `src/slipwai/project/languages/rust.py`.
  - RED: a factory test generates `--http axum` (standard profile) and runs the service's
    `cargo test --locked --lib adapters::driving::http` (`CARGO_BUILD_JOBS=2`); the asset's `#[cfg(test)]` module,
    written first with a stub implementation so the failure is an assertion and not a compile error, holds:
    `GET /health` → 200 `{"status":"ok"}`; `GET /ready` with no probe → 200 `{"status":"ready"}`; with a probe that
    answers → 200; with a probe that fails → 503 `{"status":"unready","reason":"eventStore"}`, `Cache-Control:
    no-store` either way, the failure logged and not in the body; any unmatched path → 404 `{"error":"notFound"}`
    saying nothing about path or method, and the same for a known path under the wrong method; a JSON-body
    extractor whose rejection is the one 400 body `{"error":"schemaValidationFailed","field":…,"message":…}` for an
    unknown field, a wrong type and not one JSON value, naming the field and the rule and never the value. Every
    test dispatches through the real router with `oneshot`, no socket.
  - GREEN: `build_app(registrars)`, the `readiness(probe)` registrar over an object-safe probe trait the adapter
    declares (it imports no port), the fallback, the extractor; `RUST_WRITE_SIDE['axum']` in `rust_layouts.py`
    lists the files (destination to asset); `rust.py` declares the modules.
  - REFACTOR: the routes' body constants in one place; `cargo clippy -D warnings` clean in the generated project.
  - Run: `make test TESTS="test_rust_http"` (factory) and, in the scratch project, `cargo test --locked --lib`.

- [x] T006 [US1] **Rule R4 — what a browser meets first** (scenario 3). Depends on T005 (same `mod.rs`, same layout
  table).
  Files: `tests/test_rust_http.py`, `assets/backing-services/rust/http_security.rs`,
  `assets/backing-services/rust/http_app.rs` (only the `pub mod security;` line),
  `src/slipwai/project/rust_layouts.py`.
  - RED: the asset's `#[cfg(test)]` cases are Go's `http_security_test.go` cases: `secure(router, allowed_origins)`
    sets `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` and `Content-
    Security-Policy: default-src 'none'; frame-ancestors 'none'` on every response, no HSTS; `Vary: Origin`
    whenever an `Origin` came; the origin echoed with `Access-Control-Allow-Credentials: true` only when allowed; a
    preflight from an allowed origin answered 204 before any route with the methods, the requested headers (or
    `content-type`) and `Max-Age: 600`; an unknown origin gets no allow header; an empty list is same-origin only.
    Observe a stubbed `secure` fail on the first header assertion.
  - GREEN: CORS hand-written as an `axum::middleware::from_fn`, no `tower-http` (research).
  - REFACTOR: none expected; clippy clean.
  - Run: `make test TESTS="test_rust_http"`; `cargo test --locked --lib adapters::driving::http::security` in the
    scratch project.

- [x] T007 [US1] **Rule R5 — the checked environment** (scenario 11). Depends on T006 (same layout table and
  `lib.rs` module region).
  Files: `tests/test_rust_http.py`, `assets/backing-services/rust/config.rs`,
  `src/slipwai/project/rust_layouts.py`, `src/slipwai/project/languages/rust.py`,
  `src/slipwai/backends.py`.
  - RED: `config::load_from(lookup)` over a fake lookup: defaults (`HOST` `0.0.0.0`, `PORT` 3000); `PORT`
    refused unless 1–65535; `PUBLIC_BASE_URL` and `OTEL_EXPORTER_OTLP_ENDPOINT` refused unless `http://` or
    `https://`; `LOG_LEVEL` and `LOG_FORMAT` read and never refused; `OTEL_SERVICE_NAME` defaults to the service's own
    name (the placeholder `rust.py` resolves, as Go's `__SERVICE_NAME__` is); `CORS_ALLOWED_ORIGINS` comma-split,
    blanks dropped; `EVENT_STORE_PATH` only in the `sqlite` region and `DATABASE_URL` (refused unless `postgres://`
    or `postgresql://`) only in the `postgres` region; every refusal names the variable; `address()` brackets an
    IPv6 host; `reported_url()` prefers `PUBLIC_BASE_URL`. Factory example: generated `.env.example` carries the
    transport's keys through the `__TRANSPORT__` region and the store's in theirs.
  - GREEN: `config.rs`, `ENV_FEATURES` gains `axum`, `__SERVICE_NAME__` resolved in `config.rs`.
  - REFACTOR: one table of keys read by both the loader and the `.env.example`, if that reads better; clippy clean.
  - Run: `make test TESTS="test_rust_http"`; `cargo test --locked --lib config` in the scratch project.

- [x] T008 [US1] **Rule R6 — one span per request** (scenario 3). Depends on T007 (same layout table and
  `lib.rs` module region).
  Files: `tests/test_rust_http.py`, `assets/backing-services/rust/observability.rs`,
  `src/slipwai/project/rust_layouts.py`, `src/slipwai/project/languages/rust.py`.
  - RED: a request with a `traceparent` yields a span whose trace id is the caller's; a request without one yields a
    fresh one; the request span is named for its method and matched route and records `trace_id` so a log line
    written inside it carries it; `LOG_LEVEL` filters this service's records and dependencies at `warn`;
    `LOG_FORMAT=pretty` for a terminal, JSON otherwise, an unknown level read as `info`; a failing flush or shutdown
    is reported and never raised. The tests use a recording exporter written in the test module (a fake), not a mock.
  - GREEN: the provider records spans always and exports only when an endpoint was named, built before the runtime;
    the per-request layer; the subscriber setup.
  - REFACTOR: none expected; clippy clean.
  - Run: `make test TESTS="test_rust_http"`; `cargo test --locked --lib observability` in the scratch project.

- [x] T009 [US1] **Rule R7 — the published contract** (scenario 12; D4). Depends on T008 (it reads the adapter's
  routes, written by T005, and shares the layout table).
  Files: `tests/test_rust_http.py`, `assets/backing-services/rust/openapi.yaml`,
  `assets/backing-services/rust/http_openapi.rs`, `src/slipwai/project/openapi.py`,
  `src/slipwai/project/rules.py`, `src/slipwai/project/rust_layouts.py`.
  - RED: the asset's `#[cfg(test)]` module reads the adapter's source for every route it registers and fails when
    `openapi.yaml` does not describe that path and method, with a minimum count so a stale pattern cannot pass
    silently; stub it so the failure is on a route the document lacks. Factory example: the generated service has
    `openapi.yaml` with `/health`, `/ready` (200 and 503) and the `Health`, `Ready`, `Unready`, `SchemaFailure`,
    `NotFound` schemas, and no `/api/flags`; `DOCUMENTS['axum'] == "openapi.yaml"` and `API_CONTRACTS['axum']`
    exist; no exporter, no `EXPORTERS` row, no `check-openapi` recipe.
  - GREEN: the hand-written document in the shape of Go's, the test module, and the two table rows. No YAML or regex
    crate (D4).
  - Guard: break the document (rename a path), run, restore with `git checkout -- <path>`, to see the test fail.
  - Run: `make test TESTS="test_rust_http"`; `cargo test --locked --lib openapi` in the scratch project.

- [x] T010 [US1] **Rule R8 — the entry point, and running it** (scenario 4; SC-003; D5). Depends on T009 (it wires
  every module written before it).
  Files: `tests/test_rust_http.py`, `assets/backing-services/rust/serve_main.rs`,
  `src/slipwai/project/rust_entry.py` (new), `src/slipwai/project/composition.py`,
  `src/slipwai/project/run_skill.py` (or the module that writes the generated run page),
  `src/slipwai/project/rust_layouts.py`.
  - RED: factory examples at generation: `src/bin/serve.rs` exists for `--http axum`; Rust's `ENTRY_STORES` row
    fills `__STORE_IMPORT__`, `__STORE_OPEN__` and `__STORE_ARGUMENT__` for each store (none, `memory`, `sqlite`,
    `postgres`, more than one) and `ENTRY_WIRING` gains no row (no flag placeholder, slice 4); the generated project
    builds `cargo build --locked --all-targets` and `cargo clippy -D warnings` with no warning for each store and
    after `./init --event-store memory` (research: `unused_assignments` and friends); the generated `skills/run-the-
    app/SKILL.md`, where a project has two Rust services with a transport, says their dev servers share one
    `target/` and run one at a time on the host, or together through `make demo` (D5); a Rust `axum` service beside a
    Go `net-http` one gets two Compose services, two ports and two `make dev` entries. `serve.rs` itself has no test
    (coverage already ignores `src/bin/`), so its RED is the build and the lint of the generated file.
  - GREEN: `serve.rs` (logging first; `config::load()` with a refusal logged and exit 1; the tracer provider built
    before the runtime; the store opened once, lazily, Postgres through `PgPoolOptions::connect_lazy` +
    `PostgresEventStore::from_pool`; adapted to the probe; the router wrapped by `observability` and `security` outside
    `build_app`; bound on `config.address()`, the reported URL logged; graceful shutdown on SIGTERM and Ctrl-C, then
    the provider's shutdown); `rust_entry.py` with Rust's `EntryStore`, merged where `composition.py` reads the table
    (`entry_stores.py` is at its line budget and is not touched); D5's sentence.
  - Proven by running, isolated and disposable (nothing already running is touched): from a scratch generated
    project, `make dev` answers `GET /health` with `{"status":"ok"}` and `GET /ready` with `{"status":"ready"}`
    (`HEALTH_BODIES`, `READY_PATHS` already say so for Rust); stop the PID this task started, by recorded PID. The
    `make demo` run is the final gate's (T015).
  - REFACTOR: none expected; suite green, clippy clean.
  - Run: `make test TESTS="test_rust_http test_running"`.

- [x] T011 [US1] **Rule R10 — taking the transport away** (scenario 8). Depends on T010.
  Files: `tests/test_rust_http.py`, `assets/backing-services/prune.py`,
  `src/slipwai/project/languages/rust.py`, `src/slipwai/project/languages/cargo.py`,
  `assets/backing-services/rust/adapters_mod.rs` (or its replacement by `rust.py`'s computation).
  - RED: on each profile, generate with `axum`, run `./init --http none` in the generated project: no transport
    file (`src/adapters/driving`, `src/config.rs`, `src/observability.rs`, `src/bin/serve.rs`, `openapi.yaml`), no
    axum crate in `Cargo.toml` or `Cargo.lock`, no Compose `service`, `src/lib.rs` and `src/adapters/mod.rs` declare
    only what is left (possibly nothing; `pub mod adapters;` stays unmarked because the store shares it), and `make
    verify` (cargo) is green; the pruner refuses nothing new; the `OWNED_FILES_PER_WEB_APP` rows for `axum` exist.
    Observe it fail on the leftover files.
  - GREEN: `OWNED_FILES['axum']['rust']` with `any: packages/api-client`, `OWNED_FILES_PER_WEB_APP['axum']`
    (`src/routes`, `tests/routes`), `APP_SERVICE_FEATURES` gains `axum`, `PACKAGE_EDITS['rust']['axum']` names the
    transport's crates so `_relock_rust` drops them offline through `cargo metadata`,
    `MARKED_FILES_BY_LANGUAGE['rust']` gains `src/lib.rs`, `src/adapters/mod.rs`, `src/bin/serve.rs`,
    `src/config.rs`; `declare_modules` writes `config` and `observability` inside an `axum` region of `src/lib.rs`
    and `src/adapters/mod.rs` from the files present.
  - REFACTOR: none expected; suite green.
  - Run: `make test TESTS="test_rust_http test_prune"` (cargo verify per profile is slow; `CARGO_BUILD_JOBS=2`).

- [x] T012 [US1] **Rule R11 — a browser app beside a Rust service** (scenario 6). Depends on T009 and T011.
  Files: `tests/test_rust_http.py`, and `src/slipwai/project/openapi.py`, `src/slipwai/project/rules.py`, or
  `assets/backing-services/prune.py` only if the RED shows a table row is missing.
  - RED: with `--frontend react-vite --http axum`, `packages/api-client` is generated from the Rust service's
    `openapi.yaml`, `apps/web/vite.config.ts` proxies `/api` inside the `axum` region, the route that shows the API
    answering arrives, and the generated `make verify` passes. The behaviour is the generic machinery reached by
    `DOCUMENTS['axum']` and the per-web-app rows; if the RED is green on arrival, fold the example into T009 or T011
    (whichever produced the behaviour) and say so, rather than ship a test born green. The matrix's
    `event-modelling` / `react-vite` row is the native proof (T013).
  - Run: `make test TESTS="test_rust_http"`.

- [x] T013 [US1] **Rule R12 — the factory holds Rust to the transport rows** (FR-007 for this slice).
  Depends on T010, T011 and T012.
  Files: `tests/test_rust_http.py`, `tests/test_catalog.py`, `tests/test_matrix.py`, `tests/test_readiness.py`,
  `tests/test_running.py` (plus whatever the sweep below finds, named in the commit), `docs/axes.md`, `README.md`,
  `docs/backend-obligations.md` (only if a table row has to be named).
  - RED: `TRANSPORTS['rust'] = 'axum'`; `PARTIAL['rust'] = {"event-store", "http"}` and every test reading it
    follows; the matrix's maximal row gives each backend only the `auth` and `users` answers it is offered (Rust:
    `none`); `test_catalog`'s default-project checks take Rust's `auth` and `users` as `none`; `test_readiness`
    reads `.rs` sources; `test_running` has a Rust row (`axum`, `apps/service/src/bin/serve.rs`, `cargo run --locked
    --bin serve`); the multi-service case (Rust `axum` beside Go `net-http`: two Compose services, two ports, two
    `make dev` entries) is asserted at generation. Observe each fail before its flip.
  - GREEN: the test edits above, plus the docs: `docs/axes.md`'s `--http` row (with `axum` among the per-backend
    defaults, D2), its "every backend but Rust" sentence, Rust's coverage row and the "what arrives" row with its
    dependency column; `README.md`'s `--http` list. Sweep `grep -rn "rust" tests/*.py | grep -i "partial\|transport"`
    for any other test that reads Rust as having no transport.
  - Guard: the `make starters` before/after diff (every non-Rust tree identical) is recorded in T015, not here.
  - Run: `make test TESTS="test_rust_http test_catalog test_matrix test_readiness test_running"`.

- [x] T014 [P] [US1] **Rule R13 — the changelog** (FR-008; D2; D6). Depends on nothing from the other tasks; its
  wording is checked once T013 is done.
  Files: `changelog.d/rust-http-axum.md` (new), `changelog.d/rust-backend.md`, `changelog.d/rust-event-store.md`.
  Not a RED-GREEN increment: a fragment is a document and `tests/test_changelog.py` is its check.
  - The new fragment's first line is `MINOR`: what `axum` gives a Rust service (routes, hardening, tracing, checked
    environment, `openapi.yaml`, `serve` binary, `make dev`, `make demo`), that a new Rust project now gets an HTTP
    service by default while one generated before keeps what it recorded (no `http` key reads as `none`), `--http
    none` as the way back, and the **Catch-up.** (nothing for an existing project). `VERSION` stays `1.4.0.dev0`.
  - D6: correct in place the sentences in `rust-backend.md` ("no transport") and `rust-event-store.md` ("Rust still
    answers no transport …; axum comes next") that would be false at release.
  - Run: prove `tests/test_changelog.py` in a scratch `git clone --no-tags` of the worktree under
    `$HOME/.cache/slippy-cruise-tmp/`, not in this checkout (its tag failure here is environmental).

## Design review

No screen in this slice.

## Model mockups

No white box in this slice: no event model, no screen states to write back; `check-model` has nothing to refuse.

## Phase 3: Polish

- [ ] T015 [US1] **Final gate.** Depends on T001–T014. No file is written except what a failure hands back.
  Run each, report each outcome, and do not commit red or touch a file no manifest names to make it pass:
  - `make check-locks` clean (nine committed locks, no diff after regeneration).
  - `make starters` before and after: materialise at the base `40dacad` (a scratch `git worktree`-free export or a
    `git clone --no-tags` under the cache) and at HEAD; every non-Rust tree is identical; the Rust trees differ only
    by what this slice adds. Record the diff summary.
  - The `--http none` byte comparison against `40dacad` on both profiles: identical except `project.json`'s
    `http: none` (T003's recorded result, re-run on final HEAD).
  - A generated Rust `axum` project's `make dev` answers `GET /health` (`{"status":"ok"}`) and `GET /ready`
    (`{"status":"ready"}`); stop the PID started; a scratch `./init --http none && make verify` is green.
  - `make demo` reports the service healthy within the Compose healthcheck's window (`start_period` 20s, 60 retries
    at 5s) on a first container build (SC-003), then `make demo-down`. Report the elapsed time. If Docker is not
    available, say so and hand it back; do not mark SC-003 proven.
  - The full `make verify` green, run in the background with its log under `$HOME/.cache/` and polled; the one
    environmental `test_changelog` tag failure is noted and proved green in the `--no-tags` clone; any other failure is
    real.
  - Clean up every scratch generation and cargo target directory under `$HOME/.cache/slippy-cruise-tmp/`.

## Dependencies & execution order

- T001 first: it creates the suite and pins both seams green before anything changes.
- T002 needs T001. T003 needs T002 (its RED is `--http none` after the option exists).
- T004 (R9) needs T002 and comes **before** the asset rules: T005–T010 write Rust that a generated project compiles
  with `--locked`, which needs axum's crates in the manifest and a committed lock that holds them. The plan lists R9
  after R8; this order is the dependency order and the rules themselves are unchanged.
- T005 → T006 → T007 → T008 → T009 → T010 in that order: each adds a module declared in `rust.py`, a row in
  `rust_layouts.py`, and (T006) a line in T005's `http/mod.rs`; T009's RED reads T005's routes; T010 wires every
  module before it.
- T011 needs T010 (the serve binary and marked regions it prunes). T012 needs T009 and T011. T013 needs T010–T012.
- T014 needs nothing, but check its wording once T013 is done.
- T015 last.
- One task per commit; the fragment (T014) and the docs (T013) ride in the same pull request.

## Parallel opportunities

- **May run alongside:** T014 with any task. Its files, `changelog.d/rust-http-axum.md`, `changelog.d/rust-backend.md`
  and `changelog.d/rust-event-store.md`, appear in no other task and no RED reads anything another task writes.
- **May not:** everything else is sequential. T002 to T013 all write `tests/test_rust_http.py`, and most share
  `catalog.json`, `prune.py`, `rust_layouts.py`, `rust.py` or `cargo.py` (T002/T011 in `prune.py`; T004/T011 in
  `cargo.py`; T005–T010 in `rust_layouts.py`; T005, T007, T008, T011 in `rust.py`). T004 precedes T005–T010 because
  their RED builds a generated project against its lock. T009 reads T005's routes. T012 needs T009's document and
  T011's rows. T013 sweeps files the earlier tasks touch. T015 waits for all.
- With one delegate for the story, take T001 to T013 in order, T014 beside any of them if a second agent is wanted
  (it saves little), then T015. No two concurrent tasks write the same file.

## Not working yet (owned by later slices or out of scope)

- No `--auth` or `--users` answer but `none` for Rust: slices `auth` and `users`; `PARTIAL['rust']` still names them.
- No `/api/flags`, no `ENTRY_WIRING['axum']`, no flagged document, no image, no `aws` or `azure` target: slices `aws`
  and `azure`. The entry point carries no flag placeholder.
- Two Rust services' `serve` binaries share a name and a `target/` (D5): run their dev servers one at a time on the
  host, or together through `make demo`.
- After `./init --http none` on the standard profile, `tokio`, `serde` and `serde_json` stay declared and
  `src/adapters/mod.rs` may declare nothing.
- No exporter, no `EXPORTERS` row, no `check-openapi` recipe for Rust (D4); no `tower-http`, no YAML or regex crate.

## Convergence

(Written by the convergence pass. It also records the T003 and T015 byte comparisons and the `make starters` diff.)

## Phase 4: Convergence pass 1 — what the slice still owes

Judged at `115e5a7` against `7e4291e..HEAD` (pass 1 of 2). Evidence is from a scratch generation under
`$HOME/.cache/slippy-cruise-tmp/converge1/` (outside the repository) unless a repository path is named; no
production file in the worktree was changed. Only `CRITICAL` and `HIGH` re-open the loop.

- [x] T016 [US1] **HIGH — the 400 body quotes the caller's value when the value contains `, expected `** (R3:
  "names the field and the rule and never the value"; scenario 3). `schema_failure_for` in
  `assets/backing-services/rust/http_app.rs:216` rebuilds the rule by splitting serde's *message* on its first
  `, expected `, and serde's message for `invalid type` quotes the value first. Observed through `Router::oneshot` in a
  generated `--http axum` project: body `{"quantity":"x, expected SECRET-TOKEN"}` into a `u32` field →
  `400 {"field":"quantity","message":"must be SECRET-TOKEN\", expected u32"}`. Go's `schemaFailureFor` reads the
  structured `UnmarshalTypeError.Type` and cannot leak this way.
  RED: one `#[cfg(test)]` case per serde message shape that carries the caller's input — `invalid type`,
  `invalid value`, `invalid length`, `unknown variant`, and a custom `de::Error::custom` — each with a value holding
  `, expected ` and a token, asserting the token is absent from the body.
  GREEN: the message is never parsed for the value side — the rule comes from what does not depend on input (the
  text *after the last* `, expected ` is still serde's prose, so prefer the classification plus the field, or the
  tail after `rsplit_once`, held by the cases). Sweep: **every branch of `schema_failure_for`** is exercised by a case
  whose value contains each delimiter the branch parses on (`` ` ``, `, expected `, ` at line `).

- [x] T017 [US1] **HIGH — the entry point's own log lines are filtered out at every level but `warn`**
  (R8: "bound on `config.address()`, the reported URL logged"; scenario 4). `subscriber` in
  `assets/backing-services/rust/observability.rs:154-161` lets through only the *library* crate's target
  (`module_path!()`'s first segment, e.g. `ax_std`) at `LOG_LEVEL`; `src/bin/serve.rs` is a separate crate whose target
  is `serve`, so its `tracing::info!(…"service listening")` (`serve_main.rs:94`) is dropped. Observed: the generated
  binary run with `PORT=38517` answered `GET /health` and printed nothing at all; a refusal (`PORT=0`) printed
  because it is `ERROR`. `make dev` therefore never says where the service is.
  RED: a case in `observability.rs` writing an `info` record with `target: "serve"` (and the package's other binary,
  `migrate`, where the store region has it) through `subscriber("info", …)` and asserting it is written; a factory
  example that starts the generated `serve` on a free port (as `test_make_dev_answers_health_and_ready…` does) and
  reads the reported URL from its output.
  GREEN: the filter admits every target this package owns — the library and each of its binaries — at the asked
  level, dependencies still at `warn`. Sweep: **every `tracing::` call in every generated `src/bin/*.rs`** is at a
  target the filter admits.

- [x] T018 [US1] **HIGH — no trace-to-event correlation, which every other backend's transport gives** (spec: "the
  bar is the other backends"; US1 "everything a transport brings with it elsewhere"). Go's `tracing.go:135`
  `TraceIDs`, TypeScript's `tracing.ts:145` `traceIds`, and Python's `tracing.py` `trace_ids` turn the request span
  into the event's correlation and causation ids, each with tests (`TestAnEventIsCorrelatedByTheTraceTheCallerSentIn`,
  `TestNothingInventsAnIDOutsideARequest`). `assets/backing-services/rust/observability.rs` has no equivalent, while the
  generated `.env.example` (transport region) tells the reader spans "give every log line and every event a trace
  id". RED: the two Go cases, ported — the caller's trace id re-punctuated as a correlation UUID, the request span id
  in the low half of a causation UUID, and nothing outside a request. GREEN: a `trace_ids()` in `observability.rs`
  returning ids `events.rs` accepts (`CorrelationId`/`CausationId` parse them), documented as Go's is. Sweep: **every
  helper `tracing.go`, `tracing.ts` and `tracing.py` export to a slice** has a Rust counterpart or a written reason in
  `observability.rs`'s module note.

- [ ] T019 [US1] **HIGH — `--http none` is not byte-identical to today's tree** (scenario 9; R2; and the fragment's
  "`--http none` generates exactly what Rust generated before", `changelog.d/rust-http-axum.md:21`). Generated at the
  base `40dacad` with no `--http` and at HEAD with `--http none`, both profiles, `diff -r -x .git`:
  `project.json` (expected), plus — avoidable — `apps/service/Cargo.toml` gains an empty `[dev-dependencies]` table
  with a two-line comment (`assets/languages/rust/app/Cargo.toml:15-18`), and on event-modelling
  `src/adapters/mod.rs`'s doc comment changed wording (`src/slipwai/project/languages/rust.py:72-73` vs the base
  asset); and — inherent to recording `http: none` and to a catalog change — `README.md` gains `- HTTP transport:
  `none`` and `scripts/backing-services.py` (the shipped pruner) gains the `axum` rows. T003's test holds the file
  set, not the bytes, so none of this was seen.
  RED: a factory test comparing `--http none` bytes, per profile, to a committed or reconstructed baseline for
  every file except the ones the decision below names. GREEN: no `[dev-dependencies]` table (nor its comment) when
  nothing goes in it, and the base's `adapters/mod.rs` wording when there is no `driving`. **Product question,
  returned rather than decided here:** scenario 9 names only `project.json`; whether the README selection line and
  the shipped pruner are acceptable differences is a spec amendment for the drive-skipper to record in
  `decisions.md` (an agent does not edit the spec to pass, Principle XIV). Then correct the fragment's sentence to
  match. Sweep: **every file under both `--http none` trees**, not only those T001 listed.

- [x] T020 [US1] **HIGH — a slice route mounted with `Router::route` still tells the caller which verbs exist**
  (R3: "the same for a known path under the wrong method"; scenario 3). `build_app` strips `Allow` only for routes
  mounted through the adapter's `route` helper (`http_app.rs:97-99`); axum adds `Allow` after every layer for a plain
  `Router::route`, and the adapter's own examples teach that form (`http_app.rs:408`, `:426`). Observed: a registrar
  `router.route("/orders", post(…))`, `GET /orders` → `404 {"error":"notFound"}` **with `allow: POST`**. Go's mux
  fallback cannot leak this, so the Rust guarantee is by convention where Go's is by construction, and the next
  slices (`auth`, `users`) add routes.
  RED: a case mounting a registrar with plain `Router::route`, requesting the wrong verb (and a preflight from an
  allowed origin to it) through the stack `serve.rs` builds — `instrument(secure(build_app(…)))` — and asserting no
  `Allow`. GREEN: the header is removed outside the router as a whole (a layer around the finished `Router` as one
  service, in `build_app` or the entry point), so no mounting style can bring it back; the examples in the tests use
  whichever form a slice should copy. Sweep: **every way a registrar can mount a route** (`route`, `route_service`,
  `nest`, `merge`) under the wrong verb.

- [ ] T021 [US1] **MEDIUM — a new direct dependency nobody was asked about** (Principle XIV: "a new dependency" is a
  stop-and-ask). `serde_path_to_error = "0.1.20"` is in the `axum` region (`src/slipwai/project/languages/cargo.py`,
  generated `Cargo.toml`) and in `PACKAGE_EDITS['rust']['axum']`, but in neither `research.md`'s crate table nor
  `decisions.md` (only commit `0741263`'s message). It adds no package to any lock — axum's `json` feature already
  pulls it (`assets/languages/rust/locks/axum/Cargo.lock`, axum's dependency list). GREEN: the drive-skipper records
  the decision (keep, or derive the path another way) in `decisions.md`, and `research.md` gains its row with the
  citation. Sweep: **every crate in both `axum` regions** of the manifest is in `research.md`'s table.

- [x] T022 [US1] **MEDIUM — four committed locks are stale, so `make check-locks` is not clean** (R9 guard; T015).
  Re-resolved here with `scripts/regenerate-locks.py`'s own `rust_locks()`: the five `*-axum` locks match; `memory`,
  `memory-sqlite`, `memory-postgres` and `memory-sqlite-postgres` differ by transitive patch releases (e.g. `js-sys`
  0.3.105→0.3.106, `wasm-bindgen` 0.2.128→0.2.129, `1.16.1`→`1.16.2`, `1.4.7`→`1.5.1`). GREEN: `make locks` for the
  Rust variants, committed with the slice (user-visible, covered by the MINOR fragment). Sweep: **every lock variant**
  `make check-locks` resolves, all ecosystems, clean.

- [x] T023 [US1] **LOW — the D5 sentence is written for two Rust services whether or not they serve.**
  `src/slipwai/project/run_skill.py:201` counts `service.language == "rust"`, where R8 says "two Rust services with a
  transport". GREEN: count Rust services whose selection has the transport; a case with one Rust `axum` and one Rust
  `none` service gets no sentence. Sweep: **every condition in `run_skill.py`** that names a language also asks
  whether the service serves, where serving is what it describes.

T015 (the final gate) is still open and is not repeated here: the `make starters` before/after diff, `make demo`
inside SC-003's window (Docker 29.6.1 is available on this machine), the full `make verify`, and the scratch clean-up.

### Draft verdict — not converged

**Not converged: five HIGH findings (T016–T020) re-open the loop; T021–T022 are MEDIUM, T023 LOW, and T015 is
still open.** By level:

- **Domain:** none generated or touched. The slice adds no entity, port or use case.
- **Use case (`/ready` through the probe):** holds. `serve.rs` adapts the opened store to the adapter's own
  `ReadinessProbe` through `head()` (`src/slipwai/project/rust_entry.py`, `StoreProbe`), the adapter imports no port,
  and `None` on the standard profile answers ready (`http_app.rs:133-160`; cases at `:343-384`, green in a generated
  project, 46/46 lib tests). Postgres opens lazily. Not proven: the probe against a live failing store is the
  integration suite's, which this transport does not add (catalog `integration-suite: false`).
- **Delivery adapter:** routes, the JSON 404, the 503 body, `Cache-Control: no-store`, security headers on every
  answer including 404s, a 204 preflight with `Max-Age: 600`, `Vary: Origin`, and span names `GET /ready`,
  `GET unmatched`, `OPTIONS /ready` were all observed through the stack `serve.rs` builds. The config refusals match
  Go's one for one (`config.rs:86-113` vs `config.go:106-132`). Not holding: the 400 can quote a value (T016), the
  served process logs no URL (T017), there is no trace-to-event correlation (T018), and a plainly mounted route leaks
  `Allow` (T020).
- **Published contract:** holds. `openapi.yaml` describes `/health` and `/ready` (200/503); its test fails when a
  path is renamed (seen: `GET /ready is served and openapi.yaml does not describe it`); `DOCUMENTS['axum']` and
  `API_CONTRACTS['axum']` exist with no exporter. With `--frontend react-vite --http axum --event-store memory`, on
  event-modelling, `packages/api-client` builds from `apps/service/openapi.yaml`, `vite.config.ts` proxies `/api`
  inside the `axum` region, and the generated `make install && make verify` passed: "verify: all gates passed", in 2
  minutes.
- **Factory:** the catalog and `prune.py` agree (`catalog.json` `axum` option and `default.http.rust`;
  `prune.py` `FEATURES`, `AXES`, `OWNED_FILES`, `OWNED_FILES_PER_WEB_APP`, `APP_SERVICE_FEATURES`, `PACKAGE_EDITS`,
  `MARKED_FILES_BY_LANGUAGE`). Nothing branches on the option's name; `cargo.served()` asks the feature. The suite
  rows FR-007 names for this slice are in place: `TRANSPORTS`, `PARTIAL['rust'] = {event-store, http}`, the matrix
  rows' `offered(...)`, `test_readiness` `.rs`, and `test_running`'s Rust row. The docs and README name `axum`, and
  the D6 amendments read true. `--http none` against the base is not what scenario 9 says (T019), and four locks are
  stale (T022). The factory suites for this slice (`test_rust_http*`, `test_catalog`, `test_axes`, `test_readiness`,
  `test_running`, `test_pruning`): 72 tests, OK, in 16 minutes at `115e5a7` — green, and none of them reaches T016–T020. `make starters` was not run here (T015).

**Constitution, principle by principle:**
- **I — answers keep meaning:** holds for the record. A Rust record with no `http` key migrates to no transport
  (`tests/test_rust_http.py:112`). The catalog change is additive (`catalog.json:16`, the `axum` option), and the
  fragment claims MINOR (`changelog.d/rust-http-axum.md:1`) against `VERSION` `1.4.0.dev0`.
  `tests/test_changelog.py` passed, 13 tests with 1 skipped, in a `--no-tags` clone at `115e5a7`. "Every combination
  passes its own gate" is proven here for the event-modelling/react-vite/memory row and the per-store builds
  (`tests/test_rust_http_entry.py:95`), not for the whole matrix (T015). The `--http none` "today's tree" claim is
  not met (T019).
- **II — re-running is safe:** `./init --http none` only subtracts (`tests/test_rust_http_prune.py:34`, `:59`). The
  slice adds no new writing command, so no second-run test is owed.
- **III — simplicity:** holds. There is no `tower-http`, YAML or regex crate (`http_security.rs:4-9`,
  `http_openapi.rs:3-11`), and `project/rust_entry.py` exists only because of the line budget. The one crate beyond
  research is T021.
- **VII — observability (applied to the long-running process the slice generates):** structured JSON logs
  (`observability.rs:165-175`) and a trace id on each line inside a request (`observability.rs:201-214`) hold. The
  process's own start-up line is lost (T017), and events are not correlated by trace (T018).
- **VIII — versioning:** MINOR fragment, `VERSION` unchanged, `catalog.json` additive, and no `schemaVersion` move.
- **IX — security:** holds for the factory itself: no credential is logged or written by slipwai. In the generated
  service, the 400 leak (T016) breaks the plan's own security line. The `DATABASE_URL` refusal echoing the value
  (`config.rs:143-146`) and `/ready` logging the driver's error (`http_app.rs:151`) match Go's `config.go:131-132`
  and `http_app.go:147` exactly. That is the parity bar, so neither is owed here; both are a cross-backend question.
- **X — trunk:** one PR for the slice. Commit `f9eed3b` deliberately left three suites red until later tasks, inside
  the same PR. This is recorded, not re-opened: history is not rewritten.
- **XIV — agent change, same bar:** the spec and constitution were not edited by the implementer (`git diff
  7e4291e..HEAD` touches no `spec.md`). The unasked dependency is T021, and scenario 9's wording is returned as a
  product question (T019).

Recorded for the verdict, as T003 asked: the `--http none` byte comparison is above (T019). The `make starters`
diff is still T015's.

## Phase 4: Convergence pass 2 — what is still open, graded, none re-opening

Judged at `97fcaa4` against `7e4291e..HEAD` (pass 2 of 2, the confirming pass). Each pass-1 finding was re-run
rather than read. The probes ran in scratch generations under `$HOME/.cache/slippy-cruise-tmp/converge2/`, and no
production file in the worktree was changed.

- [ ] T024 **MEDIUM — `make check-locks` cannot be clean from this slice: the four uv locks have drifted
  upstream** (R9: "`make check-locks` is clean before the PR"). `scripts/regenerate-locks.py`'s own resolvers were
  run read-only. `rust_locks()` gave 9 variants, none stale, and `go_module_files()` gave 6, none stale. But
  `python_locks()` reports `assets/languages/python/locks/uv.lock`, `uv-fastapi.lock`, `uv-postgres.lock` and
  `uv-fastapi-postgres.lock` stale. This slice has not touched them (`git diff 7e4291e..HEAD --
  assets/languages/python` is empty, and the last change was `8ac6145`, 2026-09-17). The npm leg is the host's
  re-run at the base. Handed back, not graded HIGH: refreshing Python locks inside a Rust slice is another
  ecosystem's change in this PR (`AGENTS.md`: one PR per slice). Either they are refreshed on `main` in a PR of
  their own, or R9's guard is recorded as the Rust variants'.
- [ ] T025 **LOW — pruning the transport leaves an empty `[dev-dependencies]` table under a comment that is no
  longer true.** Since `2229f57`, the table's header and comment are written by
  `src/slipwai/project/languages/cargo.py:127-138` outside the `axum` region. So `./init --http none` on an
  `axum` project, standard profile, leaves `# … ./init --http none takes them out …` and `[dev-dependencies]` with
  nothing under it. A `--http none` generation has neither. The project still builds, and R10 asks for no byte
  equality between the two routes. GREEN: the header goes inside the region, or the pruner drops a table it
  emptied. The case is an `axum` → `none` prune with no `[dev-dependencies]` line left.
- [ ] T026 **LOW — T023's guard does not pin the condition it is named for.**
  `tests/test_rust_http_entry.py:196` uses two services, so the `several` gate masks
  `src/slipwai/project/run_skill.py:201`. A Rust `axum` + Rust `none` + Go `net-http` project is the unmasked
  case. Run here, it gave a *Several services* section and no shared-target sentence, because of the filter at
  `run_skill.py:89`. GREEN: that three-service case added to the test, so a mutation of line 89 or 201 is caught.

## Convergence

**Verdict at `97fcaa4`: converged at the loop's bound.** No CRITICAL or HIGH finding is open. T016, T017, T018,
T020, T022 and T023 were each re-run and are closed. T019's first half (the avoidable `Cargo.toml` and
`adapters/mod.rs` bytes) is closed too. Two things stay open: T019's second half and T021, handed back as product
questions (below), and T024 (MEDIUM), T025 and T026 (LOW), appended above. None of the open items is CRITICAL,
so none re-opens the loop past its bound. T015, the after-acceptance gate, is still open as before: the
`make starters` diff, `make demo`, the full `make verify` and the scratch clean-up. This pass did not run it.

**Each pass-1 finding, re-run:**
- **T016 closed.** A scratch `tests/probe.rs` in a generated event-modelling `--http axum` project sent 16
  hostile bodies through `sealed(instrument(secure(build_app(…))))` with `oneshot`. The fields were nested
  (`line.quantity`), sequence (`lines[0].quantity`), map value (`counts.1`), unit enum, internally tagged enum,
  tuple too short and too long, integer out of range, root of the wrong type, a trailing second value, and a
  truncated body. Each value held `, expected `, a backtick or ` at line `. Every one was a 400 with no
  `SECRET-TOKEN` in the body. Example: `{"quantity":"x, expected SECRET-TOKEN"}` gave `must be u32`. Sweep, every
  branch of `schema_failure_for` (`assets/backing-services/rust/http_app.rs:230-278`): unknown field, missing
  field, the four `wanted_by` prefixes, and the `Error::custom` fallback. The generated project's own cases
  (`http_app.rs:574` onwards) pass in its `cargo test`.
- **T017 closed.** The generated `serve` was run on port 38617. At `LOG_LEVEL=info` and at `debug` it printed
  `{"level":"INFO","fields":{"message":"service listening","url":"http://localhost:38617",…},"target":"serve"}`.
  At `warn` it printed nothing, which is correct. `GET /health` answered 200. Sweep: every `tracing::` call in
  `src/bin/*.rs` across the standard, memory and postgres generations is in `serve.rs`. `migrate.rs` (postgres)
  only uses `println!`/`eprintln!`. Both binaries are in `BINARIES` (`observability.rs:154`). The factory test
  that sweeps this (`tests/test_rust_http_entry.py:125`) and the `make dev` test (`:143`) are green.
- **T018 closed.** Through the full stack, a handler called `observability::trace_ids()` with `traceparent:
  00-4bf92f35…-00f067aa0ba902b7-01` and got correlation `4bf92f35-77b3-4da6-a3ce-929d0e0e4736` and causation
  `00000000-0000-0000-129c-902567c302a6`, which is this service's span and not the caller's. Both parsed through
  the generated `CorrelationId::parse` and `CausationId::parse`. Outside the request the call returned `None`.
  Sweep: `observability.rs:28` onwards names a counterpart for every helper exported by `tracing.go`,
  `tracing.ts` and `tracing.py`. The log-line handlers have none, with the reason given there, and
  `tests/test_rust_http_service.py:125` holds that.
- **T019 first half closed; the rest is handed back.** At `40dacad` (`git archive`) and at HEAD with
  `--http none`, I generated both profiles and ran `diff -rq` over every file. **Standard** differs only in
  `project.json` and `README.md`. **Event-modelling** (default store `postgres`) differs in `project.json`,
  `README.md`, `scripts/backing-services.py` and `Cargo.lock` (`js-sys` 0.3.105→0.3.106 and the like, from
  T022). `apps/service/Cargo.toml` and `src/adapters/mod.rs` are now byte-identical. This matches
  `tests/test_rust_http_none.py`'s `NAMED_AND_PENDING` and `RE_RESOLVED` exactly.
- **T020 closed.** Through the stack `serve.rs` builds (`serve_main.rs:90`), I tried six mounting forms:
  `route`, `route_service`, `nest`, `nest_service`, `merge`, and a method router whose own fallback answers 405.
  Each was sent `GET`, `DELETE`, `PUT` and `HEAD`, and each answer was `404 {"error":"notFound"}` with the four
  security headers and no `Allow`. A preflight from an allowed origin, and an `OPTIONS` with no `Origin`, carried
  no `Allow` either. `POST /health` and `POST /ready` got 404 with no `Allow`, and the right verbs still answered.
  On the running binary, `curl -X POST /health` gave `404` with no `Allow`. The code is `http::sealed`
  (`http_app.rs:124`) and `without_hints` (`:94`).
- **T022 closed for what this slice owns.** `rust_locks()` resolved read-only and all 9 committed Rust locks
  equal it. `go_module_files()` gave 6, all equal. uv is T024, and npm is the host's re-run. I did not run
  `make check-locks`, as briefed.
- **T023 closed (it never reproduced).** `run_skill.py:89` filters `services` to those with a transport, so
  line 201 counts only serving services. The guard is green, and the unmasked three-service case (T026) shows the
  same.

**New since pass 1, judged:** nothing the fixes added is HIGH or CRITICAL. `without_hints` turns *every* 405,
including one a handler writes, into the contract's 404. That is what R3 promises. `sealed` wraps the span, so
span names are unchanged (the `observability` tests are green). The `span_id` on each line carries no input.
`test_rust_http_baseline.py` is data generated from `git archive 40dacad`, as its docstring says. The one
regression a fix brought is T025, and it is LOW. Commit `d6cd7c8` lint-fixes test lines from the commits before
it. That is recorded under X/XIV and not re-opened, the same as pass 1's `f9eed3b`.

**Suites:** `test_rust_http`, `test_rust_http_locks`, `test_rust_http_service`, `test_rust_http_entry`,
`test_rust_http_prune`, `test_rust_http_none`, `test_catalog`, `test_axes`, `test_readiness`, `test_running` and
`test_pruning` ran 78 tests, OK, in 1114 s at `97fcaa4`.

**By level:**
- **Domain:** none generated or touched.
- **Use case (`/ready` through the probe):** unchanged since pass 1 and holds (`http_app.rs:162`). The suites
  are green.
- **Delivery adapter:** this level now holds where pass 1 found it did not. The 400 quotes no value (T016), a
  wrong verb leaks nothing for any mounting form (T020), the process says where it listens (T017), and events
  can be correlated by trace (T018). Not proven: a live failing store behind `/ready`, as in pass 1.
- **Screen:** none. The react-vite proxy and api-client row is as pass 1 recorded, and nothing since touches it.
- **Published contract:** `openapi.yaml` is unchanged. A wrong verb now always answers the `NotFound` schema the
  document already describes.
- **Factory:** the catalog and pruner tables are unchanged since pass 1. The generator writes `--http none` as
  before, apart from the files named and pending (T019). Locks: Rust and Go are clean, uv has drifted (T024).

**Constitution, principle by principle, at the final code:**
- **I — answers keep meaning:** `--http none` is byte-equal to `40dacad` except the named files. Evidence:
  `tests/test_rust_http_none.py` with `test_rust_http_baseline.py`, plus my independent `diff -rq` above.
  Inputs: `src/slipwai/project/languages/rust.py:77,99` and `languages/cargo.py:127-138`. A record with no `http`
  key migrates with no transport (`tests/test_rust_http.py:112`, green). Whether the README line and the shipped
  pruner are acceptable differences is still a question (handed back, 1).
- **II — re-running is safe:** `./init --http none` still only subtracts, and the gate is green after it
  (`tests/test_rust_http_prune.py:34`, green). T025 is what that subtraction leaves behind.
- **III — simplicity:** the fixes add no crate. `sealed` is axum's `Router` and `middleware::map_response` only
  (`http_app.rs:124`), and `trace_ids` is a re-punctuation (`observability.rs:233`). The only crate beyond
  research is still `serde_path_to_error` (handed back, 2).
- **VII — observability (the long-running process the slice generates):** logs are structured JSON with
  `trace_id` and `span_id` on every line inside a request (`observability.rs:284,292`). The entry point's own
  lines are admitted (`:154`). The correlation identifier events take is `trace_ids()` (`:233`). The heartbeat is
  `/health` and `/ready` (`http_app.rs:162`). Alerting on it belongs to a production target, and this slice adds
  none.
- **VIII — versioning:** MINOR (`changelog.d/rust-http-axum.md:1`), `VERSION` `1.4.0.dev0`, and the catalog
  change is additive (`catalog.json:16`, `:408`). No `schemaVersion` move. The fragment's
  "`--http none` generates exactly what Rust generated before" (`:21`) is still untrue until question 1 is
  decided.
- **IX — security:** a 400 never quotes a value (`http_app.rs:230-278`, cases from `:574`, probe above). The
  `DATABASE_URL` refusal and the `/ready` log keep Go's parity, as pass 1 recorded (`config.rs:112`). The
  factory logs and writes no credential.
- **X — trunk:** one PR for the slice. `d6cd7c8` restores lint for the commits before it, in the same PR.
- **XIV — agent change, same bar:** `git diff 7e4291e..HEAD` touches no `spec.md`, `decisions.md` or
  constitution. The two decisions outside the implementer's constraints are handed back, not taken.

**Handed back, open (`plan.md` `## Blocked`, line 307).** Both are stated accurately in substance. Two
corrections for whoever decides:
1. *Scenario 9's exceptions (T019).* The pruner difference is **event-modelling only**: a standard project ships
   no `scripts/backing-services.py`. The block's "on both profiles … and also in" reads as if it applied to both.
   Standard differs in `project.json` and `README.md` alone. Options (a) and (b) are otherwise as stated, and
   the fragment sentence (`changelog.d/rust-http-axum.md:21`) follows the choice.
2. *`serde_path_to_error` (T021).* "Adds no package to any lock" is true. `serde_path_to_error` is a dependency
   of axum in `assets/languages/rust/locks/axum/Cargo.lock` and appears only in the five `*-axum` locks. Option
   (b) costs more than it says. Without the path, an unknown field and a missing field lose their parent too
   (`line.x` becomes `x`), and a wrong type in a nested field becomes `(root)`. So it is a loss of the field
   path for every nested failure, not only for a wrong type.

## Owed after the post-converge gaps pass (2026-10-02, `57f04d3`)

`drive-gaps` over `7e4291e..57f04d3` found six gaps no earlier task held. The HIGH and the MEDIUMs are implemented
here; the LOW rides in Phase 4.

- [x] T027 [US1] **HIGH — `make dev` on the default event-modelling project (Postgres) exits at start when
  `DATABASE_URL` is unset** (scenario 4; R8's "nothing connects while the process starts"). `connect_lazy("")` fails
  to parse (`rust_entry.py`), where Go's `pgxpool.New(ctx, "")` falls back to libpq defaults and the process starts,
  answering `/health` 200 and `/ready` 503. GREEN: an empty URL opens lazily from libpq-style defaults
  (`PgConnectOptions::new()`), a malformed one still stops the process naming `DATABASE_URL`. Sweep: **every store
  answer's open with its variable unset and malformed** (sqlite, postgres), each starting or refusing as Go's does;
  a factory case starts `serve` on a Postgres project with no `DATABASE_URL` and sees `/health` 200, `/ready` 503.
- [x] T028 [US1] **MEDIUM — `add-service` into a Rust project recorded before this slice gives the new service
  `axum`** (edge case *generated before*; D2, whose premise is that an unrecorded axis reads as its `absent`).
  `add_service.py` reads the first service's answer with `choices.get(axis)` and falls back to the catalog default.
  The artifacts settle the answer (D2's premise, the spec's edge case "only a new project gets `axum` by default",
  and `add_service`'s own docstring that a new service inherits the first service's answer): an axis the record
  never asked is inherited as its `absent`. GREEN: that reading in `add_service`, pinned by a case adding a service
  to a record with no `http` key. Sweep: **every place a recorded selection is replayed** (`add-service`, `migrate`,
  `replay.py`) reads an unasked axis as its `absent`.
- [x] T029 [US1] **MEDIUM — scenario 9's promised factory test that runs `make verify` on `--http none` does not
  exist**, and since defaults became `axum` nothing builds a no-transport Rust project. GREEN: one test generating
  `--http none` on standard and on event-modelling with a store, running its `make verify`.
- [x] T030 [US1] **MEDIUM — a Rust project named `test` or `std` fails its own gate** (the new `serve` binary, and
  Postgres's `migrate` before it, collide with the sysroot crates). GREEN: `crate_name` prefixes every reserved
  sysroot crate name (`test`, `std`, `alloc`, `proc_macro` — `core` builds and is left alone) as it prefixes a leading digit, with a case per
  name.
- [x] T031 [US1] **LOW–MEDIUM — the fragment's Catch-up says nothing is asked of an existing project**, but
  `slipwai migrate` on a Rust project generated before moves its `Cargo.lock`'s transitive patch versions (T022).
  GREEN: one Catch-up sentence in `changelog.d/rust-http-axum.md` saying so and that a conflicting lock is resolved by
  re-locking.
- [ ] T032 [US1] **LOW (Phase 4)** — after `./init --http none`, `.env.example` keeps `NODE_ENV`, `LOG_LEVEL`,
  `LOG_FORMAT` that nothing reads, and `src/adapters/mod.rs`'s comment still names `driving`; a fresh `--http none`
  has no `.env.example`. Cosmetic; R10 asks no byte equality.

## Gate (2026-10-02, `7547d2d`)

T015, run by the slice's host session:

- **`make verify` green**: 941 tests, OK (8 skipped), `verify: all gates passed`, in a `git clone --no-tags` of the
  worktree at `7547d2d` (so `test_changelog` ran green there; this checkout's local `v1.4.0`/`v1.5.0`/`v1.5.1` tags
  make it red here only). The baseline before the slice was green the same way at `7e4291e` (888 tests).
- **SC-003**: a generated standard Rust project (`--http` defaulted to `axum`), `make demo` on a first container
  build reported `ledger-service-1 Healthy` after 215 s, inside the healthcheck's 320 s window; `/health` →
  `{"status":"ok"}`, `/ready` → `{"status":"ready"}`, an unknown path → 404; `make demo-down` left no container.
  `make dev` on the host is held by `tests/test_rust_http_entry.py` (both bodies, SIGTERM stop by PID).
- **`make check-locks` could not run clean**: it stops in npm (`Cannot read properties of null (reading
  'edgesOut')`) before uv and Go — and does the same at the base `7e4291e`, so the failure is this machine's npm, not
  the slice. The nine Rust and six Go locks match their resolution (convergence pass 2, resolvers called directly);
  the four uv locks are stale from upstream drift (T024).
- **Not run**: the `make starters` before/after diff over every backend. The Rust `--http none` trees are held byte
  for byte by `tests/test_rust_http_none.py`; every other backend's event-modelling tree differs from the base in the
  shipped `scripts/backing-services.py`, which carries the `axum` rows (the shared pruner — Blocked question 1).

T015 stays open for the `make starters` diff and a clean `make check-locks` on a machine whose npm resolves.
