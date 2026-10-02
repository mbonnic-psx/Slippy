"""R9 — the manifest and the locks: the transport's crates in a marked region, `tokio` once, and a lock for each union.

No cargo here: a committed lock is read as text and held to the crates the manifest names, so this runs in the
seconds a factory suite should. That a lock *resolves* with those crates is the matrix's `--locked` build.
"""
from __future__ import annotations

import re
import tomllib

from support import FactoryTestCase

from slipwai.assets import ROOT
from slipwai.project.languages import cargo
from slipwai.selection import Selection


def chosen(store: str | None = None, http: str | None = None) -> Selection:
    """The selection of one Rust service: which store it answered, and whether it answered `axum`."""
    answers = {}
    if store is not None:
        answers["event-store"] = store
    if http is not None:
        answers["http"] = http
    return Selection(answers)


# Every committed lock, by the selections whose union it is resolved for.
LOCK_VARIANTS = {
    "memory": [chosen("memory")],
    "memory-sqlite": [chosen("sqlite")],
    "memory-postgres": [chosen("postgres")],
    "memory-sqlite-postgres": [chosen("sqlite"), chosen("postgres")],
    "axum": [chosen(http="axum")],
    "memory-axum": [chosen("memory", "axum")],
    "memory-sqlite-axum": [chosen("sqlite", "axum")],
    "memory-postgres-axum": [chosen("postgres", "axum")],
    "memory-sqlite-postgres-axum": [chosen("sqlite", "axum"), chosen("postgres")],
}


MEMBER = r'\[\[package\]\]\nname = "{name}"\nversion = "[^"]+"\n(.*?)(?=\n\[\[package\]\]|\Z)'


def direct_names(lock_text: str, member: str) -> list[str]:
    """The crates a member of a committed lock lists, without the version Cargo adds where two share a name."""
    entry = re.search(MEMBER.format(name=re.escape(member)), lock_text, re.S)
    assert entry is not None, member
    listed = re.search(r"dependencies = \[\n(.*?)\]", entry.group(1), re.S)
    lines = listed.group(1).splitlines() if listed else []
    return [line.strip().strip(",").strip('"').split(" ")[0] for line in lines]


def pruned(text: str, selection: Selection) -> str:
    """The manifest text as generation leaves it: every marked region the selection does not hold, cut away.

    Written whole and cut afterwards is how the factory applies a selection (one prune, not two), so a test of
    what a manifest names has to cut the same regions to read it."""
    def region(match: re.Match[str]) -> str:
        return match.group(0) if selection.has(match.group(1)) else ""

    return re.sub(r"# backing-service:([\w|-]+):begin\n.*?# backing-service:\1:end\n", region, text, flags=re.S)


class RustManifestAndLocksTest(FactoryTestCase):
    """R9: the transport's crates sit in a marked region, `tokio` stays one key, and the committed locks cover them."""

    def test_the_lock_follows_the_union_of_the_stores_and_whether_any_service_serves(self) -> None:
        for variant, selections in LOCK_VARIANTS.items():
            self.assertEqual(cargo.lock_variant(selections), variant)
        # Nothing to depend on is the empty lock, whether the axes were never asked or the transport was refused.
        self.assertIsNone(cargo.lock_variant([chosen()]))
        self.assertIsNone(cargo.lock_variant([chosen(http="none")]))
        self.assertEqual(cargo.lock_variant([chosen("sqlite", "none")]), "memory-sqlite")

    def test_tokio_is_one_key_with_the_features_of_the_store_and_the_transport(self) -> None:
        for selection, features in (
            (chosen("sqlite", "axum"), {"macros", "net", "rt-multi-thread", "signal", "sync", "time"}),
            (chosen(http="axum"), {"macros", "net", "rt-multi-thread", "signal", "sync", "time"}),
            (chosen("sqlite"), {"macros", "rt-multi-thread", "sync", "time"}),
        ):
            with self.subTest(selection=selection):
                text = pruned(cargo.dependencies(selection), selection)

                # `tomllib` refuses a key written twice, which is what a second `tokio` in a region would be.
                manifest = tomllib.loads("[dependencies]\n" + text)
                self.assertEqual(set(manifest["dependencies"]["tokio"]["features"]), features)
                self.assertEqual(len(re.findall(r"^tokio = ", text, re.M)), 1)
                unmarked = re.sub(r"# backing-service:(\w+):begin\n.*?# backing-service:\1:end\n", "", text, flags=re.S)
                self.assertIn("tokio = ", unmarked)

    def test_the_transport_alone_needs_its_own_crates_and_none_of_the_store_s(self) -> None:
        text = pruned(cargo.dependencies(chosen(http="axum")), chosen(http="axum"))

        dependencies = tomllib.loads("[dependencies]\n" + text)["dependencies"]
        self.assertTrue({"axum", "tokio", "serde", "serde_json", "tracing", "tracing-subscriber"} <= set(dependencies))
        self.assertFalse({"thiserror", "time", "uuid", "sqlx"} & set(dependencies))
        # The transport's crates are in a region of their own, and `serde` and `serde_json` are not: the store
        # needs them too, so taking the transport away must leave them.
        region = re.search(r"# backing-service:axum:begin\n(.*?)# backing-service:axum:end\n", text, re.S)
        assert region is not None
        inside = set(tomllib.loads("[dependencies]\n" + region.group(1))["dependencies"])
        self.assertIn("axum", inside)
        self.assertFalse(inside & {"serde", "serde_json", "tokio"})

    def test_the_dev_dependencies_the_router_tests_use_are_in_a_region_of_their_own(self) -> None:
        manifest = (ROOT / "assets/languages/rust/app/Cargo.toml").read_text()

        served = cargo.with_dependencies(manifest, chosen("sqlite", "axum"))
        quiet = cargo.with_dependencies(manifest, chosen("sqlite"))

        with_router = tomllib.loads(pruned(served, chosen("sqlite", "axum")))
        without = tomllib.loads(pruned(quiet, chosen("sqlite")))
        self.assertEqual(set(with_router["dev-dependencies"]), {"http-body-util", "tower"})
        self.assertEqual(without.get("dev-dependencies", {}), {})
        self.assertIn("# backing-service:axum:begin\nhttp-body-util", served)
        self.assertNotIn("__DEPENDENCIES__", served + quiet)
        self.assertNotIn("__DEV_DEPENDENCIES__", served + quiet)

    def test_a_manifest_names_exactly_the_crates_the_lock_member_lists(self) -> None:
        manifest = (ROOT / "assets/languages/rust/app/Cargo.toml").read_text()
        for variant, selections in LOCK_VARIANTS.items():
            for selection in selections:
                with self.subTest(variant=variant, selection=selection):
                    parsed = tomllib.loads(pruned(cargo.with_dependencies(manifest, selection), selection))
                    named = sorted({*parsed["dependencies"], *parsed.get("dev-dependencies", {})})

                    self.assertEqual(cargo.direct_crates(selection), named)

    def test_every_committed_lock_lists_every_crate_its_services_name_directly(self) -> None:
        for variant, selections in LOCK_VARIANTS.items():
            with self.subTest(variant=variant):
                lock_text = (cargo.LOCKS / variant / "Cargo.lock").read_text()

                listed = direct_names(lock_text, cargo.PLACEHOLDER)

                needed = {crate for selection in selections for crate in cargo.direct_crates(selection)}
                self.assertEqual(sorted(needed), sorted(listed))

    def test_a_served_and_a_quiet_service_take_the_union_lock_each_member_listing_its_own(self) -> None:
        text = cargo.lock([("served", chosen("sqlite", "axum")), ("quiet", chosen("memory"))])

        self.assertEqual(text.count('name = "delivery-starter"'), 0)
        self.assertEqual(sorted(direct_names(text, "served")), cargo.direct_crates(chosen("sqlite", "axum")))
        self.assertEqual(sorted(direct_names(text, "quiet")), cargo.direct_crates(chosen("memory")))
        self.assertNotIn("axum", direct_names(text, "quiet"))
        # The union lock carries the transport's packages once, for the member that asked.
        self.assertEqual(text.count('name = "axum"\n'), 1)

    def test_the_fragment_s_catch_up_says_migrate_moves_a_rust_lock_and_how_a_conflict_is_resolved(self) -> None:
        fragment = (ROOT / "changelog.d/rust-http-axum.md").read_text()
        catch_up = fragment.split("**Catch-up.**", 1)[1]

        self.assertIn("slipwai migrate", catch_up)
        self.assertIn("Cargo.lock", catch_up)
        self.assertIn("re-lock", catch_up)
