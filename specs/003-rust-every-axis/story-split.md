# Story split — Rust answers every axis and target

The capability is *someone generating a Rust service is offered every answer the other backends are offered,
and each answer works*. Each slice below leaves that person able to choose one more answer for Rust, end to end
— the question asked, the files generated, the generated project's own `make verify` green — never a layer of
one answer on its own. Split by **rules** (SPIDR): one axis or one target per slice, in the order the issue
gives and `requires` imposes, one pull request each.

`VERSION` already reads `1.4.0.dev0`, a MINOR over the last release; each slice's fragment claims MINOR (a new
option, a new target for an existing backend), so no slice raises it.

## Slices, in order

| # | Slice | What the actor can do afterwards | Specification | Depends on |
|---|---|---|---|---|
| 1 | `http-axum` | Generate a Rust service with `--http axum` (or `none`), run `make dev`, hit `/health` and `/ready`, and `make demo` it | US1; FR-001 (http), FR-002, FR-006, FR-007 (`TRANSPORTS`, README), FR-008; SC-003; edge cases: mixed-backend project, `--http none` beside a transport, committed locks, a project generated before this feature | — |
| 2 | `auth` | Generate a Rust `axum` service with `--auth keycloak` and get the realm, the role mapping and the OIDC adapter; the cloud answers listed for Rust | US2; FR-001 (auth), FR-003 (auth), FR-004 | 1 |
| 3 | `users` | Generate a Rust `axum` service with `--users keycloak` (and with `--auth keycloak` beside it) and get the customers realm and the customer adapter | US3; FR-001 (users), FR-003 (users), FR-004; FR-007 (Rust out of `PARTIAL`, the matrix's maximal row); SC-001 | 1, 2 |
| 4 | `aws` | Generate a Rust project under `--target aws`, build its production image and smoke it, and have the deploy workflow ship it | US4; FR-005 (aws), FR-007 (`targeting("aws")`, the matrix's `aws` row); SC-002, SC-004 (aws) | 1, 2, 3 |
| 5 | `azure` | The same under `--target azure`, `entra` included | US5; FR-005 (azure), FR-007 (`targeting("azure")`); SC-004 (azure) | 4 |

## Slice graph

```text
http-axum ── auth ── users ── aws ── azure
```

- `http-axum`: depends_on none
- `auth`: depends_on http-axum
- `users`: depends_on http-axum, auth
- `aws`: depends_on http-axum, auth, users
- `azure`: depends_on aws

## Why a chain, not a fan-out

- **`auth` and `users` could run side by side on paper** — each needs only `http` — but both write the same rows
  of `prune.py`'s `OWNED_FILES` and `PACKAGE_EDITS` for Rust, the same marked regions of the Rust `Cargo.toml`,
  and the same committed locks, and the matrix's maximal row needs both at once. Two branches each regenerating
  `Cargo.lock` is a conflict in a generated file; in order, `users` lands on the lock `auth` left. The issue
  orders them the same way.
- **`aws` after both identity axes** because its maximal row is `cognito` for both, and because `PARTIAL` is
  emptied by `users`, so the target slice is held to every axis at once.
- **`azure` after `aws`** because it reuses the image builder, the migration and TLS answers and the flag reader
  `aws` adds; it is the same tables read by a second cloud.

## Why not another split

- **Not `none` before `axum`.** Listing Rust under `http: none` alone changes nothing a user sees — Rust already
  resolves to its no-transport answer unasked — so it rides in slice 1.
- **Not the image apart from the target.** An image nothing deploys is not something the actor can choose; the
  image builder lands with the first target that ships it.
- **Not per identity provider.** `keycloak`, `cognito`, `entra` and `auth0` share one adapter (feature `keycloak`);
  four slices would be one adapter and three catalog lines.
