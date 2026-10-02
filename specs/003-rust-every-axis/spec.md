# Feature Specification: Rust answers every axis and target

**Feature Branch**: `003-rust-every-axis`

**Created**: 2026-10-01

**Status**: Draft

**Input**: Fork issue mbonnic-psx/Slippy#11 — "Rust answers every axis: http (axum), auth, users, and the aws/azure
targets", so the Rust work can go upstream to slipwai complete. Taken as the second half of the run started as
`/cruise Rust issues` (`specs/001-rust-cargo-adopt/decisions.md`, D16).

## User Scenarios & Testing *(mandatory)*

The actor is **someone generating a new project with `slipwai generate` who chose Rust** for a service. Today
Rust is the one backend that answers only the event store: it is never asked for an HTTP transport, a staff or
customer identity provider, or a cloud, and the only targets it can be given are `none` and `existing`. Such a
project can be verified but not demonstrated — no `make dev`, no Compose service, no `make demo` — and cannot be
deployed by the factory at all.

**The bar is the other backends, and no more.** Every answer below is "what every other backend already gets for
that answer, in Rust's idiom" — the obligations `docs/backend-obligations.md` lists per axis and per target, the
menus `docs/axes.md` documents — never a richer or a different behaviour. Where another backend that owns no
startup (Go, TypeScript, Python) leaves a part deliberately unimplemented, Rust leaves the same part unimplemented
in the same way.

### User Story 1 - A Rust service serves HTTP through axum (Priority: P1)

The actor generates a Rust service and is asked the HTTP question, as for every other backend: `none` or
`axum`. With `axum` the service has an HTTP driving adapter, the entry point that binds a port, and everything a
transport brings with it elsewhere, so the actor can run `make dev`, open the service, and `make demo` it.

**Why this priority**: auth, users and both clouds require `http` (`catalog.json` `requires`); nothing else in
this feature can be offered before it.

**Independent Test**: Generate a Rust project with `--http axum`, run its `make verify`, run `make dev` and
request `GET /health` and `GET /ready`; generate one with `--http none` and see the same files Rust gets today.

**Acceptance Scenarios**:

1. **Given** the actor chose Rust, **When** the HTTP question is asked, **Then** the offered answers are `none`
   and `axum`, interactively and through `--http`, and the default is `axum`, as every backend's default is its own transport (D2).
2. **Given** `--http none`, **When** the project is generated, **Then** the Rust service is exactly what Rust
   generates today with no transport: no adapter, no `serve` binary, no Compose service.
3. **Given** `--http axum`, **When** the project is generated, **Then** the service has the HTTP driving adapter
   with the same routes the other transports serve — `GET /health`, `GET /ready`, a JSON 404 for any unmatched
   path, and `GET /api/flags` under a managed target only — the same CORS and preflight hardening, one tracing
   span per request, a committed OpenAPI document a test holds to the routes, edge tests that dispatch through
   the real router without opening a socket, and one entry point binary that binds the configured port.
4. **Given** that project, **When** the actor runs `make dev`, **Then** the service starts and answers
   `GET /health` with the body the probe table records for Rust; **When** they run `make demo`, **Then** it is
   demonstrable as every other transport is (Compose `service`, `.env.example` keys, `make dev`).
5. **Given** `--http axum` with each event store (`memory`, `sqlite`, `postgres`, and more than one), **When**
   the project is generated, **Then** its committed `Cargo.lock` builds `--locked` and offline, as the event-store
   locks do today, and its `make verify` passes.
6. **Given** a browser frontend beside a Rust `axum` service, **When** the project is generated, **Then** the
   frontend reaches it the way it reaches any other transport (the generated API client and the routes the
   frontend proxies), and `make verify` passes.
7. **Given** `--profile standard --http axum` (no event store — the default Rust project after D2), **When** the
   project is generated, **Then** its committed `Cargo.lock` accepts `--locked` and `make verify` passes: the lock
   variants are {no store, `memory`, `memory`+`sqlite`, `memory`+`postgres`, `memory`+`sqlite`+`postgres`} ×
   {axum, no transport}, and `scripts/regenerate-locks.py` makes every one of them.
8. **Given** a generated Rust `axum` project, **When** the actor runs `./init --http none`, **Then** the project
   that remains passes `make verify` and its `Cargo.lock` is re-locked without network, axum's crates gone from it:
   every module the transport adds is declared inside a marked region, and the entry point, the config module,
   `src/lib.rs` and `src/adapters/mod.rs` are listed for Rust in `MARKED_FILES_BY_LANGUAGE`, as Go's are.
9. **Given** `--http none` on either profile, **When** the project is generated, **Then** its files are
   byte-identical to what Rust generates today except where the answer is recorded or the factory's shared parts
   learn the option (D8): `project.json`'s selection and the generated `README.md`'s selection line, which now
   record `http: none`; the shipped `scripts/backing-services.py`, which every backend's copy shares and which gains
   the `axum` rows; and, on event-modelling, `Cargo.lock`, re-resolved with the locks. A factory test generates it,
   holds every other byte, and runs `make verify`.
10. **Given** any Rust selection, **When** its manifest is written, **Then** `tokio` is declared once, with the
    features axum and the store need together, never as a second key in a marked region.
11. **Given** `--http axum`, **When** the service reads its environment, **Then** it reads the keys Go's transport
    reads, with the same refusals — `HOST`, `PORT` (3000 by default), `PUBLIC_BASE_URL`, `LOG_LEVEL`, `LOG_FORMAT`,
    `OTEL_SERVICE_NAME`, `OTEL_EXPORTER_OTLP_ENDPOINT`, `CORS_ALLOWED_ORIGINS` — the store's keys staying in their
    marked regions of `.env.example`; on the Standard profile `/ready` answers ready with no dependency to ask, as
    Go's does.
12. **Given** `--http axum`, **When** the OpenAPI document is checked, **Then** it is a committed, hand-written
    `openapi.yaml` that a Rust test holds to the routes the router registers, as Go's is — no exporter, no
    `check-openapi` recipe (D4).

---

### User Story 2 - A Rust service is given a staff identity provider (Priority: P2)

With `axum` answered, the actor is asked the staff identity question for Rust, and is offered exactly the answers
every other backend is offered under the same target: `keycloak` under `none` and `existing`, `cognito` under
`aws`, `entra` under `azure`, `auth0` under either cloud, or `none`. The cloud answers share the one adapter
`keycloak` brings (feature `keycloak` in `catalog.json`), so they are listed for Rust in this story and become
reachable when US4 and US5 give Rust the clouds.

**Why this priority**: it is the first thing `http` unlocks, and the issue orders it next.

**Independent Test**: Generate Rust with `--http axum --auth keycloak`, run `make verify`, start the Keycloak
container and see the realm imported; ask for `--auth keycloak` with `--http none` and see the refusal.

**Acceptance Scenarios**:

1. **Given** Rust with `axum`, **When** the auth question is asked, **Then** it offers the same answers, per
   target, as it offers every other backend.
2. **Given** Rust with `--http none`, **When** the actor asks for any auth answer but `none`, **Then** generation
   refuses with the sentence every backend's refusal prints (`REQUIRES_BECAUSE`).
3. **Given** `--auth keycloak`, **When** the project is generated, **Then** the realm, the container and the
   group-to-role mapping arrive as for every other backend, and the service has the OIDC adapter with the role
   mapping implemented and tested — including the `cognito:groups` claim — and the protocol flow left
   unimplemented exactly as Go, TypeScript and Python leave it.
4. **Given** each auth answer, **When** the project is generated, **Then** its `make verify` passes.

---

### User Story 3 - A Rust service is given a customer identity provider (Priority: P3)

With `axum` answered, the actor is asked the customer identity question for Rust and is offered what every other
backend is: `keycloak` under `none` and `existing`, `cognito` under `aws`, `auth0` under either cloud, or `none`
— the cloud answers reachable once US4 and US5 land, as in US2.

**Why this priority**: it needs `http` and nothing from US2, and the issue orders it after auth.

**Independent Test**: Generate Rust with `--http axum --users keycloak`, run `make verify`, and read the
customer adapter's tests.

**Acceptance Scenarios**:

1. **Given** Rust with `axum`, **When** the users question is asked, **Then** it offers the same answers, per
   target, as it offers every other backend; with `--http none` any answer but `none` is refused as for every
   backend.
2. **Given** `--users keycloak`, **When** the project is generated, **Then** the `customers` realm arrives in the
   same container as for every other backend, and the service has the customer adapter: a validated token becomes
   a customer only from that realm's issuer and only with a verified email, tested; bearer-token validation is
   left unimplemented exactly as Go, TypeScript and Python leave it.
3. **Given** `--auth keycloak --users keycloak` together, **When** the project is generated, **Then** both
   adapters coexist in one service and `make verify` passes — the maximal selection the factory's matrix runs for
   every backend.

---

### User Story 4 - A Rust project deploys to AWS (Priority: P4)

The actor chooses the `aws` target and can choose Rust for a service. The project gets what every other backend
gets under `aws`: a production image built for the service, its migrations run in production the way the
backend's table says, the deploy workflow, and the `build`, `push`, `smoke-image`, `smoke`, `deploy`,
`rollback`, `url` (and `migrate-remote` where a store is applied by a task) targets.

**Why this priority**: it needs `http` (`aws` requires it) and an image, and the issue orders it after the
identity axes.

**Independent Test**: Generate Rust under `--target aws` with `--http axum --event-store postgres --auth cognito
--users cognito`, run `make verify`, then `make build smoke-image` and see the image answer `/ready`.

**Acceptance Scenarios**:

1. **Given** `--target aws`, **When** the backend question is asked, **Then** Rust is offered, with the menus
   `docs/axes.md` documents for every backend under `aws` (`memory`/`postgres`, `none`/`axum`,
   `none`/`cognito`, `none`/`cognito`).
2. **Given** that project, **When** `make build smoke-image` runs, **Then** a production image of the Rust service
   is built and answers its readiness probe at the path and body the probe tables record for Rust.
3. **Given** `--event-store postgres` under `aws`, **When** the project is generated, **Then** its migrations
   run in production the way `MIGRATIONS_IN_PRODUCTION` records for Rust, and the database's TLS policy is told to
   the driver the way `POSTGRES_SSLMODE` records for Rust.
4. **Given** that project, **When** the deploy workflow is read, **Then** it installs whatever Rust's image
   builder needs, the way every other backend's does, and the flag reader is committed where `FLAG_READERS`
   records for Rust.
5. **Given** that project, **When** `make verify` runs, **Then** it passes.

---

### User Story 5 - A Rust project deploys to Azure (Priority: P5)

The actor chooses the `azure` target and can choose Rust for a service, with what every other backend gets under
`azure`, `entra` as the staff identity answer among them.

**Why this priority**: it reuses the image US4 builds; the issue orders it last.

**Independent Test**: Generate Rust under `--target azure` with `--http axum --event-store postgres --auth
entra`, run `make verify`, then `make build smoke-image`.

**Acceptance Scenarios**:

1. **Given** `--target azure`, **When** the backend question is asked, **Then** Rust is offered with the same
   menus every backend has under `azure`.
2. **Given** that project, **When** `make verify` and `make build smoke-image` run, **Then** both pass, as in US4.

---

### Edge Cases

- **A multi-service project mixing Rust with another backend** gets each service's answers in its own idiom;
  a Rust `axum` service beside a Go `net-http` one gets its own Compose service, port and `make dev` entry.
- **A Rust service with `--http none` in a project whose other service has a transport** stays a library or
  worker, as for every backend.
- **A project already generated with Rust before this feature** records `http: none` (or no transport) in its
  `project.json`; `slipwai migrate` must carry it forward with the answers it gave meaning what they meant. Its record
  has no `http` key, which reads back as the axis's `absent`, `none`, never the catalog default — so it keeps
  meaning "no inbound HTTP", and only a new project gets `axum` by default (D2).
- **The `PARTIAL` exemption.** Rust leaves the factory's "answers only some axes" list once it answers every
  axis; `docs/axes.md`'s row of dashes for Rust becomes its answers, and the "every backend but Rust" sentences
  go.
- **The committed locks.** Rust's `Cargo.lock` is chosen per union of stores today; once the transport and the
  identity adapters bring crates of their own, every combination a generated project can be given still builds
  `--locked` and offline.

### Gaps reviewed

Before planning (iteration 3): the parity bar and what it leaves unimplemented (answered by the issue and
`docs/axes.md`); which identity answers each target offers (from `catalog.json`: `cognito`, `entra`, `auth0`
are cloud-only, so they become reachable with US4 and US5); Rust's default transport and what it does to a
project generated before (D2); what "every Rust combination" in the issue's *done when* is held by (FR-007,
SC-002: the starters plus the matrix rows every backend has, not a full cross product no backend is run
against). Each slice's own gaps pass runs at its own stage.

`http-axum` (iteration 4): fourteen findings, scenarios 7–12 above and the lines below. Already answered in the
tree, so the slice proves rather than adds them: `/health` answers `{"status":"ok"}` and readiness is `/ready`
(`probes.py`), `dev_command` is `cargo run --locked --bin serve`, `COMPOSE_CACHES` holds cargo's registry, git and
target directories (`backends.py`). Decided: D4 (the OpenAPI document), D5 (two Rust services' `serve` binaries),
D6 (the unreleased Rust fragments). No control file changes; `scripts/regenerate-locks.py` does, and `make locks`
needs cargo and network on whoever runs it.

- **`/api/flags` and the flag wiring arrive with `aws`**: Rust's targets in this slice are `none` and `existing`,
  neither managed, so the flag route, `ENTRY_WIRING['axum']` and the flagged document are slice 4's; the entry
  point here carries no flag placeholder.
- **The transport-keyed tables** this slice gives an `axum` (or `rust`) row, beside section 3's per-backend ones,
  with Go's `net-http` rows as the pattern: `ENTRY_STORES`, `ENTRY_WIRING` (no flag, above), `DOCUMENTS`,
  `API_CONTRACTS`, `ENV_FEATURES`, and in `prune.py` `FEATURES`, the `AXES` http option and its capability
  `http-axum`, `OWNED_FILES` (with the `any: packages/api-client` entry), `OWNED_FILES_PER_WEB_APP`,
  `APP_SERVICE_FEATURES` and `PACKAGE_EDITS['rust']`. With a browser app, `packages/api-client` is generated from
  the Rust service's document (scenario 6).
- **The factory suite this slice must move with it**: `TRANSPORTS['rust'] = 'axum'`; `PARTIAL['rust']` becomes
  `{event-store, http}`; the matrix's maximal row gives Rust only the auth and users answers it is offered until
  `users` lands; `test_catalog`'s default-project checks take Rust's `auth`/`users` as `none`; `test_readiness`
  reads `.rs`; `test_running` has a Rust row.
- **Locks, verifiably**: generation needs no network; a factory test holds each lock variant to contain every crate
  its manifest names directly; the variants are refreshed with `make check-locks` before the PR, since no CI job
  runs it.
- **SC-003's window** is the Compose healthcheck's (`start_period` 20s, 60 retries at 5s): `make demo` reports the
  service healthy within it on a first container build.
- **FR-007's documents**: `docs/axes.md`'s `--http` row, its "every backend but Rust" sentence, Rust's coverage
  row and the "what arrives" row, and `README.md`'s `--http` list name `axum`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `rust` backend MUST be listed under the http axis's `none` and under a new `axum` option, and
  under every auth and users option, with the per-target menus every other backend has.
- **FR-002**: `axum` MUST deliver every obligation `docs/backend-obligations.md` lists for the `http` axis —
  `dev_command`, `COMPOSE_CACHES`, the driving adapter and its routes, the entry point, the OpenAPI document and
  its test, the frontend's API client — and every per-backend table that axis makes owed.
- **FR-003**: Rust's `auth` and `users` answers MUST deliver what the obligations page lists for those axes, with
  the same parts left unimplemented that Go, TypeScript and Python leave unimplemented.
- **FR-004**: `catalog.json` and `prune.py` MUST agree on what requires `http` for Rust as for every backend, and
  a refused combination MUST print the same sentence it prints for every backend.
- **FR-005**: `backends.rust.targets` MUST include `aws` and `azure`, and every table `docs/backend-obligations.md`
  says a production target reads (`IMAGE_BUILDERS`, `MIGRATIONS_IN_PRODUCTION`, `POSTGRES_SSLMODE`,
  `FLAG_READERS`, the deploy workflow's tool setup) MUST carry a Rust entry.
- **FR-006**: Every combination of Rust answers a project can be given MUST build from its committed lock with
  `--locked`, offline.
- **FR-007**: The factory's own suite MUST hold Rust to the same rows it holds every other backend to: Rust in
  `TRANSPORTS`, out of `PARTIAL`, in the matrix's maximal-selection row and its `aws` row, in `targeting("aws")`
  and `targeting("azure")`; and `docs/axes.md`'s coverage tables and the README's `--http` list MUST name `axum`.
- **FR-008**: Each slice MUST add a changelog fragment claiming MINOR (a new axis option and new targets for an
  existing backend), and `VERSION` MUST carry the number those fragments justify.

### Key Entities

- **Rust service**: a Cargo package in the generated workspace; gains a transport, adapters and an image.
- **`axum` option**: the new http answer for Rust, its crates in the committed locks.
- **Per-backend tables**: the inventory `docs/backend-obligations.md` section 3 lists; each gains a Rust entry
  where the axis or target is answered.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `docs/axes.md`'s coverage table has no dash in Rust's row, and
  `tests/test_catalog.py::test_the_documented_axis_coverage_is_the_catalog_s` passes against it.
- **SC-002**: `make starters` materialises every Rust starter and each passes its own `make verify`; the matrix's
  Rust rows (two profile/frontend rows, the maximal selection, the `aws` row) pass native `make verify` in CI's
  `matrix` job for `rust`.
- **SC-003**: A Rust `axum` project's `make dev` answers `GET /health` and `GET /ready` within the time the other
  backends' dev servers are given.
- **SC-004**: `make build smoke-image` succeeds for a Rust service under `aws` and under `azure`, run by hand on
  this machine as `docs/backend-obligations.md` requires before a backend is called done for a target.

## Assumptions

- The parity bar is the other backends as they are on `main` today; improving what they leave unimplemented is
  not this feature.
- axum is the transport the issue names; no second Rust transport is offered.
- Each user story is one pull request, in the issue's order, through the `add-backing-service` and `add-target`
  skills; the adoption path (`layout.delivery`) is untouched, so none of this is under the experimental exemption.
