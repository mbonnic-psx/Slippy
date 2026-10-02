# Survey

Written by `slipwai adopt` (1.4.0.dev0) from the tree as it was; `/survey` refreshes it. Every line names the
file that said so. Experimental: see `../docs/adoption.md`.

## Builds

- `.` — cargo, rust, from `Cargo.toml`, rust 1.85; what it is for, nothing here says

Wrapped as: `ledger` (`.`).

## Continuous integration

- none found

Proposed forge: `none`, from no CI configuration in the tree.

## How a change reaches production

- none found

Proposed: `unknown` — nothing in the tree says, so it stays unrecorded until somebody does.

## Containers

- none found

## Infrastructure as code

- none found

Proposed home: `unmanaged`.

## Database

Schema tools:

- none found

Drivers in dependency manifests:

- none found

Proposed schema home: `none`.

## Also here

- root `Makefile`: no
- `README`: no

## Big issues that are quick wins

Each is a proposal, not a change the factory made; `/drive` offers them before the map's rows while any remain, a secret first, and `/survey` drops each as the tree stops showing it.

| Kind | Where | What | Fix |
|---|---|---|---|
| `no-lockfile` | `Cargo.toml` | no lockfile beside `Cargo.toml` (Cargo.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
