"""Cargo's half of the Rust backend: what a service's manifest depends on, and the workspace lock that pins it.

A service's dependencies follow from its selection — none for the walking skeleton, the event store's crates
once that axis is answered, the HTTP transport's inside a marked region of their own, and `sqlx` inside a
marked region per store, so `./init --event-store memory` takes the crate out with the adapter that used it. The lock is committed per dependency set, because Cargo
refuses (`--locked`) a lock that disagrees with the manifests, and resolving one needs the network a
generation must not. One workspace holds every Rust service, so the lock is chosen for the *union* of their
dependency sets — Cargo unifies a crate's features across the workspace, so two services on two stores need
the lock that has both — and then carries one member entry per service.
"""
from __future__ import annotations

import itertools
import re

from ...assets import LANGUAGE_ROOT
from ...selection import Selection

# The crates the event store's code imports, and how each is asked for. Pinned to the release the committed
# locks were resolved against; `scripts/regenerate-locks.py` re-resolves them when one moves.
EVENT_STORE_CRATES = {
    "serde": '{ version = "1.0.229", features = ["derive"] }',
    "serde_json": '"1.0.151"',
    "thiserror": '"2.0.21"',
    "time": '{ version = "0.3.55", features = ["formatting", "macros"] }',
    "tokio": '{ version = "1.53.1", features = ["macros", "rt-multi-thread", "sync", "time"] }',
    "uuid": '{ version = "1.26.1", features = ["v4"] }',
}
# The crates only the transport's code imports, in the `axum` region of `[dependencies]`: the router, one span
# per request (`tracing` and the OpenTelemetry bridge) and the exporter that ships it, which is built without
# its defaults — traces only, as Go's is — and with `reqwest-rustls`, because an `https://` collector is an
# address the checked environment accepts. `tokio`, `serde` and `serde_json` are not here: the store needs
# them too, so they are written outside the region whenever either is present, and taking the transport away
# leaves them. `matched-path` is what lets a span be named for the route that matched rather than the path.
TRANSPORT = "axum"
TRANSPORT_CRATES = {
    "axum": '{ version = "0.8.9", default-features = false, features = ["http1", "json", "matched-path", "query", "tokio"] }',
    "opentelemetry": '"0.33.0"',
    "opentelemetry-otlp": '{ version = "0.33.0", default-features = false, features = ["http-proto", "reqwest-blocking-client", "reqwest-rustls", "trace"] }',
    "opentelemetry_sdk": '"0.33.0"',
    "serde_path_to_error": '"0.1.20"',
    "tracing": '"0.1.44"',
    "tracing-opentelemetry": '{ version = "0.34.0", default-features = false }',
    "tracing-subscriber": '{ version = "0.3.23", features = ["ansi", "fmt", "json", "registry", "std"] }',
}
# What the router's own tests dispatch with: `oneshot` runs a request through the real router with no socket,
# and `http-body-util` reads the answer. Not shipped in a build of the service.
TRANSPORT_DEV_CRATES = {
    "http-body-util": '"0.1.5"',
    "tower": '{ version = "0.5.3", features = ["util"] }',
}
# The runtime features the transport adds to the store's: a listener and the signals that stop it.
TRANSPORT_TOKIO_FEATURES = ("net", "signal")
# What the store and the transport both import, so it sits outside either's region.
SHARED_CRATES = ("serde", "serde_json", "tokio")
# One `sqlx` per store, in that store's marked region: a service holds exactly one store, so the two never
# meet in one manifest. Postgres also compiles its migrations into `bin/migrate`, which is `migrate!`.
SQLX_VERSION = "0.9.0"
SQLX_FEATURES = {
    "sqlite": ("runtime-tokio", "sqlite"),
    "postgres": ("runtime-tokio", "macros", "migrate", "postgres"),
}
STORES = tuple(SQLX_FEATURES)
DEPENDENCIES = "__DEPENDENCIES__"
DEV_DEPENDENCIES = "__DEV_DEPENDENCIES__"
TOKIO_VERSION = "1.53.1"
TOKIO_FEATURES = ("macros", "rt-multi-thread", "sync", "time")
LOCKS = LANGUAGE_ROOT / "rust/locks"
# The member name every committed lock is resolved under, replaced by each service's own.
PLACEHOLDER = "delivery-starter"


def declared_crates() -> frozenset[str]:
    """Every crate any Rust manifest region can declare, spelled as code names it (`-` read as `_`).

    Read off the tables above, so a crate added to one is guarded against as a project name without a second list.
    """
    names = {*EVENT_STORE_CRATES, *TRANSPORT_CRATES, *TRANSPORT_DEV_CRATES, "sqlx"}
    return frozenset(name.replace("-", "_") for name in names)


def sqlx(store: str) -> str:
    features = ", ".join(f'"{feature}"' for feature in SQLX_FEATURES[store])
    return f'sqlx = {{ version = "{SQLX_VERSION}", default-features = false, features = [{features}] }}\n'


def stored(selection: Selection) -> bool:
    return selection.has("memory")


def served(selection: Selection) -> bool:
    return selection.has(TRANSPORT)


def tokio(selection: Selection) -> str:
    """`tokio` once, asked for with the features of whichever of the store and the transport is present.

    Cargo unifies a crate's features across a workspace, so the lock does not care how they are split — but
    a manifest that named the crate twice would not parse, and a feature the transport needs must not vanish
    because the store was the one that wrote the line.
    """
    features = {*TOKIO_FEATURES, *(TRANSPORT_TOKIO_FEATURES if served(selection) else ())}
    listed = ", ".join(f'"{feature}"' for feature in sorted(features))
    return f'{{ version = "{TOKIO_VERSION}", features = [{listed}] }}'


def shared_crates(selection: Selection) -> dict[str, str]:
    """The crates the store and the transport both import, written outside both regions."""
    chosen = {name: EVENT_STORE_CRATES[name] for name in SHARED_CRATES}
    chosen["tokio"] = tokio(selection)
    return chosen


def dependencies(selection: Selection) -> str:
    """The `[dependencies]` a service's manifest holds for this selection, with each answer in its region.

    Every store's region is written and the unselected ones are cut by the same prune `./init` runs, so what
    decides which store a project has is one piece of code, not two. The transport's region is written when
    it was answered and cut the same way when it is taken away.
    """
    if not stored(selection) and not served(selection):
        return ""
    crates = shared_crates(selection)
    if stored(selection):
        crates |= {name: spec for name, spec in EVENT_STORE_CRATES.items() if name not in crates}
    text = "".join(f"{name} = {crates[name]}\n" for name in sorted(crates))
    if served(selection):
        text += region(TRANSPORT, "".join(f"{name} = {spec}\n" for name, spec in sorted(TRANSPORT_CRATES.items())))
    if stored(selection):
        text += "".join(region(store, sqlx(store)) for store in STORES)
    return text


# The whole section, comment and header, written only when something goes in it: a service with nothing to
# dispatch a request with has no such table, and its manifest is the one it had before there was a transport.
DEV_DEPENDENCIES_HEAD = """# What the service's own tests dispatch with. Like a transport's crates they sit in its marked region, so
# `./init --http none` takes them out with the router they exercise.
[dev-dependencies]
"""


def dev_dependencies(selection: Selection) -> str:
    """The `[dev-dependencies]` section the router's own tests need, crates in the transport's region."""
    if not served(selection):
        return ""
    crates = region(TRANSPORT, "".join(f"{name} = {spec}\n" for name, spec in sorted(TRANSPORT_DEV_CRATES.items())))
    return f"{DEV_DEPENDENCIES_HEAD}{crates}\n"


def region(feature: str, body: str) -> str:
    return f"# backing-service:{feature}:begin\n{body}# backing-service:{feature}:end\n"


def with_dependencies(manifest: str, selection: Selection) -> str:
    return manifest.replace(f"{DEPENDENCIES}\n", dependencies(selection)).replace(
        f"{DEV_DEPENDENCIES}\n", dev_dependencies(selection)
    )


def direct_crates(selection: Selection) -> list[str]:
    """The crates a service names directly — its member entry's dependencies in the lock, sorted as Cargo does.

    Dependencies and dev-dependencies alike: Cargo lists both under the member, so a lock written without the
    router's test crates is a lock `--locked` refuses.
    """
    names: set[str] = set()
    if stored(selection):
        names |= {*EVENT_STORE_CRATES, *(["sqlx"] if any(selection.has(store) for store in STORES) else [])}
    if served(selection):
        names |= {*SHARED_CRATES, *TRANSPORT_CRATES, *TRANSPORT_DEV_CRATES}
    return sorted(names)


def lock_variant(selections: list[Selection]) -> str | None:
    """Which committed lock fits these services together, or None where none of them depends on anything."""
    parts: list[str] = []
    if any(stored(selection) for selection in selections):
        parts += ["memory", *(store for store in STORES if any(selection.has(store) for selection in selections))]
    if any(served(selection) for selection in selections):
        parts.append(TRANSPORT)
    return "-".join(parts) or None


def empty_lock(names: list[str]) -> str:
    """The lock of a workspace that depends on nothing: each member at `0.1.0`, which Cargo writes exactly so."""
    return header() + "".join(member_entry(name, []) for name in sorted(names))


def header() -> str:
    return "# This file is automatically @generated by Cargo.\n# It is not intended for manual editing.\nversion = 4\n"


def member_entry(name: str, crates: list[str]) -> str:
    listed = "".join(f' "{crate}",\n' for crate in crates)
    dependencies_line = f"dependencies = [\n{listed}]\n" if crates else ""
    return f'\n[[package]]\nname = "{name}"\nversion = "0.1.0"\n{dependencies_line}'


ENTRY = re.compile(r'\n\[\[package\]\]\nname = "([^"]+)"\nversion = "([^"]+)"\n')


def lock(members: list[tuple[str, Selection]]) -> str:
    """The workspace lock for these services: the committed lock for their union, one member entry per service.

    The committed lock carries a single member, resolved under the placeholder name with every crate the union
    names. That entry is replaced by one per service listing only its own crates, and the packages are put back
    in the order Cargo writes them — by name, then version — so `--locked` finds exactly what it would write.
    """
    variant = lock_variant([selection for _, selection in members])
    if variant is None:
        return empty_lock([name for name, _ in members])
    committed = (LOCKS / variant / "Cargo.lock").read_text()
    body = committed[len(header()):]
    starts = [match.start() for match in ENTRY.finditer(body)] + [len(body)]
    entries = [body[start:end] for start, end in itertools.pairwise(starts)]
    kept = [entry for entry in entries if ENTRY.match(entry).group(1) != PLACEHOLDER]  # type: ignore[union-attr]
    kept += [member_entry(name, direct_crates(selection)) for name, selection in members]

    def order(entry: str) -> tuple[str, tuple[int, ...]]:
        match = ENTRY.match(entry)
        assert match is not None
        return match.group(1), tuple(int(part) for part in re.findall(r"\d+", match.group(2)))

    return header() + "".join(sorted(kept, key=order))
