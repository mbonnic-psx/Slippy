# Pinned behaviour

What `/characterise` has pinned, one row per behaviour: the current behaviour of code that existed before
the delivery method did, recorded at the seam where it can be observed, so that a change to it can be told
apart from a regression. A slice reads this before it changes code that is here; `/strangle` reads it when a
behaviour moves. Rows are appended, never rewritten — a behaviour that stopped being pinned says so in a new row.

| Date | Behaviour | Seam | Tests | Runs with |
|---|---|---|---|---|
| 2026-09-28 | The nine existing ecosystems (node, python, go, maven, gradle, ant, dotnet, php, ruby) are each detected, commanded and owned as today, and a mixed directory is reported by the one tried first | `survey.buildable` / `survey.survey` over a tree on disk | `tests/test_survey.py`; the seven fixtures of `scripts/test-adoption.py` | `make test TESTS=test_survey` · `make test-adoption` |
| 2026-09-28 | A directory holding only a `[package]` `Cargo.toml` is surveyed as nothing — `buildable` returns `()` (observed by hand; the slice `single-crate` changes it on purpose, and its first RED test is this observation inverted) | `survey.buildable` over a tree on disk | none yet — `tests/test_survey.py` gains it in slice `single-crate` | `make test TESTS=test_survey` |
| 2026-10-02 | A Rust service generated with no `--http` answer, on either profile, gets no transport — no `src/bin/serve.rs`, no `src/adapters/driving/`, no `.env.example`, no Compose `service`, no `make dev` — and `project.json` records a selection with no `http` key (observed by hand: `slipwai generate rust-now --language rust --profile standard` → `selection: {}`). Feature 003's slice `http-axum` changes it on purpose (D2): afterwards no answer means `axum`, and `--http none` is this tree | `slipwai generate --language rust` (`write_project`) over a fresh directory | none yet — `tests/test_rust_http.py` gains the characterisation, file set recorded, in slice `http-axum` (T001) | `make test TESTS=test_rust_http` |
| 2026-10-02 | A Rust deployable whose `project.json` record has no `http` key is carried forward by `slipwai migrate` with no transport — the unasked axis reads as its `absent`, `none`, never the catalog default. Unchanged by slice `http-axum`, and must stay so | `slipwai migrate` over a generated Rust project | none yet — `tests/test_rust_http.py` gains it in slice `http-axum` (T001) | `make test TESTS=test_rust_http` |
