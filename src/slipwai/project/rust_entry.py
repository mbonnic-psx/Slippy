"""Rust's row of what each backend's entry point writes for each event-store answer.

A module of its own because `entry_stores.py` is at the line budget `scripts/check-structure.py` holds a module
to; the shape — `EntryStore` — is that module's, and `composition.py` reads this row beside the others.

Rust has no `*_OR_MEMORY` tail and no `nil`: the in-memory store is a binding made *above* the store's marked
region, and the region begins by dropping it before it binds the real one. That is the one spelling that is
warning-free in both states the pruner leaves behind — rustc warns on an assignment nobody reads and on a
binding shadowed before it is used, and the gate runs clippy with `-D warnings` — so a project generated with
SQLite and one pruned to the in-memory store both build clean.

No import at the top of the file sits inside a marked region: the formatter sorts a block of `use` lines and would
carry a marker comment to wherever its line sorted. The store's own names are imported in the region, inside the
function, and every import at the top is one the entry point uses whatever the pruner leaves.
"""
from __future__ import annotations

from .entry_stores import STORED, EntryStore, marked

# The one block of imports, per answer, in the order the formatter writes them. The adapter's module is imported and
# its items named through it, so no line is long enough for the formatter to wrap — which it would for one project
# name and not for another, since the crate's name is the first thing on the line.
NO_STORE_IMPORTS = """use delivery_starter::adapters::driving::http;
use delivery_starter::{config, observability};
"""
STORE_IMPORTS = """use delivery_starter::adapters::driven::event_store_memory::InMemoryEventStore;
use delivery_starter::adapters::driving::http;
use delivery_starter::application::ports::events::EventStore;
use delivery_starter::{config, observability};

/// What `/ready` asks of the event store, in the shape the HTTP adapter declares: the last global position in
/// the log, or zero when it is empty. A store that cannot answer it cannot serve a request either. Declared
/// here, because this is the one place that knows both the port and the adapter — the adapter imports no port.
struct StoreProbe<S>(S);

impl<S: EventStore + 'static> http::ReadinessProbe for StoreProbe<S> {
    fn check(&self) -> http::ProbeFuture<'_> {
        Box::pin(async move { self.0.head().await.map(drop).map_err(Into::into) })
    }
}

type Store = std::sync::Arc<dyn http::ReadinessProbe>;
"""
OPEN_HEAD = """    // The event store this project answered the event-store question with, opened once, here, and handed to
    // whatever needs it. Nothing else in this service constructs one.
    //
    // The marked block is the answer; delete it — which is what `./init --event-store memory` does — and the
    // in-memory store it starts as is what is left. Both states are valid at once, which is what a prune needs,
    // because pruning only ever subtracts.
    let store: Store = std::sync::Arc::new(StoreProbe(InMemoryEventStore::new()));
"""
# The store's own names are imported inside its region, where they are used: a `use` line that is a plain path is one
# the formatter never wraps, whatever the crate is called, and no import outside a region names a store the prune may
# have taken away. Only the crate's own paths are imported there: the formatter sorts a block of `use` lines by path,
# and where `sqlx` falls among them would depend on what the project is called.
SQLITE = (
    "    use delivery_starter::adapters::driven::event_store_sqlite::SqliteEventStore;\n"
    "    drop(store);\n"
    "    let store: Store = std::sync::Arc::new(StoreProbe(\n"
    "        SqliteEventStore::open(&settings.event_store_path).await?,\n"
    "    ));"
)
POSTGRES = (
    "    use delivery_starter::adapters::driven::event_store_postgres::PostgresEventStore;\n"
    "    use delivery_starter::application::ports::events::default_tags_of;\n"
    "    // The pool connects lazily, so this opens no socket while the process is starting: an unreachable\n"
    "    // database shows up as /ready answering 503, which is what it is. A bad connection string is a\n"
    "    // different thing and does stop the process, because nothing about it will get better on its own.\n"
    "    // An unset `DATABASE_URL` is not a bad one: it opens from libpq's defaults and its `PG*` variables, as\n"
    "    // the Go service's pool does, so the process starts and /ready says whether anything answers.\n"
    "    drop(store);\n"
    "    let options = if settings.database_url.is_empty() {\n"
    "        sqlx::postgres::PgConnectOptions::new()\n"
    "    } else {\n"
    "        settings\n"
    "            .database_url\n"
    "            .parse::<sqlx::postgres::PgConnectOptions>()\n"
    "            .map_err(|error| format!(\"DATABASE_URL cannot be used: {error}\"))?\n"
    "    };\n"
    "    let pool = sqlx::postgres::PgPoolOptions::new().connect_lazy_with(options);\n"
    "    let store: Store = std::sync::Arc::new(StoreProbe(PostgresEventStore::from_pool(\n"
    "        pool,\n"
    "        default_tags_of(),\n"
    "    )));"
)

RUST = EntryStore(
    entry="src/bin/serve.rs",
    imports={"none": NO_STORE_IMPORTS, **dict.fromkeys(STORED, STORE_IMPORTS)},
    open={None: OPEN_HEAD, "sqlite": OPEN_HEAD + marked(SQLITE, indent="    "), "postgres": OPEN_HEAD + marked(POSTGRES, indent="    ")},
    argument="Some(store)",
    absent="None",
)
