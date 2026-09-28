# Architecture view

Written by `slipwai adopt` (1.4.0.dev0) from the tree, its Git history and — where `.codegraph/` holds an index —
CodeGraph's graph; `/survey` rewrites it. Nothing here is inferred: a file is an entry point because a manifest or
its own name says so, a hotspot because commits touched it, a dependency because the graph holds the edge.
Experimental: see `../docs/adoption.md`.

## `slipwai` — `.`

### Where anything starts

- **console script `slipwai`** — `pyproject.toml`: `slipwai.cli:main`
- **runs when executed** — `src/slipwai/__main__.py`

### What it is made of

| Directory | Files | Mostly |
|---|---|---|
| `assets/` | 1300 | markdown, python |
| `changelog.d/` | 6 | markdown |
| `docs/` | 30 | markdown |
| `scripts/` | 21 | python |
| `src/` | 148 | python |
| `tests/` | 177 | python, markdown |

### What it declares it depends on

`pyproject.toml` declares no runtime dependencies.

### What it runs on

Dated 2026-09-28 against the factory's support table of 2026-09-08 (endoflife.date; JUnit and the servlet API kept by hand).

| Product | Version | Support | Until | Read from |
|---|---|---|---|---|
| Python | 3.11 | `supported` | 2027-10-31 | `toolchain.version` (detected) |

### Where change happens

110 of the 150 commits before the method arrived (since 2026-09-17; 6 author(s)) touched this application. The files they touched most:

| File | Commits |
|---|---|
| `README.md` | 22 |
| `docs/cruise.md` | 21 |
| `assets/toolkit/scripts/agents/cruise.py` | 20 |
| `src/slipwai/project/cruise.py` | 16 |
| `VERSION` | 15 |
| `assets/toolkit/scripts/agents/registry.json` | 14 |
| `docs/delivery-loop.md` | 14 |
| `src/slipwai/project/commands.py` | 14 |


### What the graph says

Not read: not indexed: `./init --extension codegraph` builds the index, then `/survey` reads it.

## What this means for the map

The Structure row of `delivery/docs/convergence.md` stands at `named`
(detected; slipwai: tool; not under apps/: .). The ladder, and what
each rung asks of this repository:

- `as-found`: to move on, record what each application is — the entry points above say it: a `start` script, a main package or a web SDK is a service, a `bin` a tool, a test directory a suite — as `kind` on its record in `project.json` with provenance `confirmed`, then `/survey`; the row moves to `named`
- **`named`** — here: to move on, move each application under `apps/<name>/`, the layout a generated project has, as a slice of its own — the wrappers and recorded commands follow the path in `project.json`; the row moves to `laid-out`
- `laid-out`: to move on, give each application the hexagonal layers a generated service has — `domain/`, `ports/`, `adapters/` — and declare `"layout": "hexagonal"` on its record, which puts it under `make check-imports`; the row moves to `hexagonal`
- `hexagonal`: to move on, record a `typecheck` command for every application and make it green through the ratchet; the row moves to `typed`
- `typed`: nothing: this is where a generated project sits

The Platform row stands at `supported` (detected;
in support on 2026-09-28: Python 3.11; no audit command recorded for slipwai). It climbs `unknown` → `inventoried` → `supported` → `audited`
as *What it runs on* above is dated, brought into support product by product — each an option above, offered as a
method slice, never a version the factory bumps — and given an `audit` command the gate runs. `/survey` reads every
product against the table on the day it runs, so a runtime that leaves support while the work goes on moves this row
by itself — and re-dates the reading only when a product, a version, a status or the table moved, so the date above
is the day something last changed and not the day somebody last looked.

A rung is claimed only once the fact behind it holds; `make check-convergence` fails a row the tree contradicts.

## Where to cut

For `/strangle`: a capability's seam is one of the entry points above — a route, a command, a job, a process — with
as few of the most-depended-on files behind it as possible. The files every other file depends on are the last to
move and the first to pin (`/characterise`, `delivery/survey/pinned.md`). The hotspots are where the next
change lands anyway, so a seam there pays for itself; a capability nothing has touched in the whole window is a
candidate to leave where it is (`docs/change-strategy.md`).
