"""Rust's service layout: which committed asset lands where under a Rust service, per marker feature.

A table of its own rather than a block in `service_layouts.py` and `read_side_layouts.py`, only because those
modules are at their size budget; the shape is theirs, and `with_read_side` folds the two halves together.
"""
from __future__ import annotations

# One module per file, declared by the `mod.rs` beside it; `src/lib.rs` declares the top-level ones from
# the files present (`languages/rust.py`). The contract suites are crate modules compiled only for tests,
# so every adapter's own `#[cfg(test)]` module can run them.
RUST_WRITE_SIDE: dict[str, dict[str, str]] = {
    "memory": {
        "src/application/mod.rs": "application_mod.rs",
        "src/application/ports/mod.rs": "ports_mod.rs",
        "src/application/ports/events.rs": "events.rs",
        "src/adapters/mod.rs": "adapters_mod.rs",
        "src/adapters/driven/mod.rs": "driven_mod.rs",
        "src/adapters/driven/event_store_memory.rs": "event_store_memory.rs",
        "src/event_store_contract.rs": "event_store_contract.rs",
    },
    "sqlite": {"src/adapters/driven/event_store_sqlite.rs": "event_store_sqlite.rs"},
    "postgres": {
        "src/adapters/driven/event_store_postgres.rs": "event_store_postgres.rs",
        # `sqlx::migrate!` compiles the SQL into the binary, which is what the production image runs;
        # `build.rs` is what tells Cargo a new migration file means a rebuild.
        "src/bin/migrate.rs": "migrate_main.rs",
        "build.rs": "migrations_build.rs",
        "migrations/001_events.sql": "../sql/001_events.sql",
        "migrations/002_events_append_only.sql": "../sql/002_events_append_only.sql",
    },
}


# The read side's half: the checkpoint port and its adapters, the runner, and the two migrations.
RUST_READ_SIDE: dict[str, dict[str, str]] = {
    "memory": {
        "src/application/ports/read_models.rs": "read_models.rs",
        # The runner and its ticker: a loop ended by the shutdown signal it is given, since no framework
        # owns one here. Importing no transport, like every other backend's.
        "src/projections.rs": "projections.rs",
        "src/adapters/driven/checkpoint_store_memory.rs": "checkpoint_store_memory.rs",
        "src/checkpoint_store_contract.rs": "checkpoint_store_contract.rs",
    },
    "sqlite": {"src/adapters/driven/checkpoint_store_sqlite.rs": "checkpoint_store_sqlite.rs"},
    "postgres": {
        "src/adapters/driven/checkpoint_store_postgres.rs": "checkpoint_store_postgres.rs",
        "migrations/003_projection_checkpoints.sql": "../sql/003_projection_checkpoints.sql",
        "migrations/004_event_tags.sql": "../sql/004_event_tags.sql",
    },
}
