"""R10 — taking the transport away: `./init --http none` subtracts it, and nothing else, from a Rust service.

Pruning only ever subtracts, so every file and every marked region the transport added has to be reachable by the
pruner: a file nobody listed is dead text, and a region in an unlisted file is never cut. Each example generates
with `axum`, answers the question again, and holds what is left to the project's own gate.
"""
from __future__ import annotations

import os
import subprocess
import tempfile
from pathlib import Path

from support import FactoryTestCase
from test_rust_http_locks import direct_names

from slipwai.assets import PRUNER

ENVIRONMENT = {**os.environ, "CARGO_BUILD_JOBS": "2"}
TRANSPORT_PATHS = (
    "src/adapters/driving", "src/config.rs", "src/observability.rs", "src/bin/serve.rs", "openapi.yaml",
)
TRANSPORT_CRATES = (
    "axum", "opentelemetry", "opentelemetry-otlp", "opentelemetry_sdk", "serde_path_to_error", "tracing",
    "tracing-opentelemetry", "tracing-subscriber", "http-body-util", "tower",
)


def init(repo: Path, *arguments: str) -> None:
    subprocess.run(["./init", *arguments], cwd=repo, check=True, stdout=subprocess.DEVNULL, env=ENVIRONMENT)


class RustTransportPruneTest(FactoryTestCase):
    def test_taking_the_transport_away_leaves_none_of_it_and_a_green_gate_on_each_profile(self) -> None:
        for profile in ("standard", "event-modelling"):
            with self.subTest(profile=profile), tempfile.TemporaryDirectory() as directory:
                repo = self.generate(directory, "pruned", profile, "rust", http="axum")

                init(repo, "--http", "none")

                service = repo / "apps/service"
                for path in TRANSPORT_PATHS:
                    self.assertFalse((service / path).exists(), path)
                manifest = (service / "Cargo.toml").read_text()
                # The lock's member entry, which lists what the manifest names: a crate the store pulls in on its
                # own (sqlx depends on `tracing`) is rightly still a package, and is not this service's to name.
                named = direct_names((repo / "Cargo.lock").read_text(), "pruned")
                for crate in TRANSPORT_CRATES:
                    self.assertNotIn(f"\n{crate} = ", manifest)
                    self.assertNotIn(crate, named)
                self.assertNotIn("backing-service:axum", (service / "Cargo.toml").read_text())
                selection = (repo / "project.json").read_text()
                self.assertIn('"http": "none"', selection)
                if (repo / "docker-compose.yml").exists():
                    self.assertNotIn("\n  service:\n", self.settings((repo / "docker-compose.yml").read_text()))
                self.assertNotIn("\ndev:", (repo / "Makefile").read_text())
                subprocess.run(["make", "verify"], cwd=repo, check=True, env=ENVIRONMENT, stdout=subprocess.DEVNULL)

    def test_what_is_left_declares_only_what_is_left(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "declared", "standard", "rust", http="axum")
            service = repo / "apps/service/src"
            self.assertIn("pub mod config;", (service / "lib.rs").read_text())
            self.assertIn("pub mod driving;", (service / "adapters/mod.rs").read_text())

            init(repo, "--http", "none")

            library = (service / "lib.rs").read_text()
            for module in ("config", "observability"):
                self.assertNotIn(f"pub mod {module};", library)
            self.assertIn("pub mod health;", library)
            # The module the store shares stays declared, and says nothing: it compiles empty.
            self.assertIn("pub mod adapters;", library)
            self.assertNotIn("pub mod driving;", (service / "adapters/mod.rs").read_text())

    def test_a_store_taken_away_leaves_an_entry_point_that_still_builds_with_no_warning(self) -> None:
        """Pruning only ever subtracts, so the in-memory store the entry point starts from is what a prune leaves —
        and a binding replaced before it is read is the warning rustc gives, which the gate refuses."""
        with tempfile.TemporaryDirectory() as directory:
            for store in ("sqlite", "postgres"):
                repo = self.generate(
                    directory, f"entry-{store}", "event-modelling", "rust", event_store=store, http="axum"
                )
                init(repo, "--event-store", "memory")
                source = (repo / "apps/service/src/bin/serve.rs").read_text()
                with self.subTest(store=store):
                    self.assertNotIn(store.capitalize(), source)
                    self.assertNotIn("backing-service", source)
                    for command in (
                        ["cargo", "fmt", "--check"],
                        ["cargo", "build", "--locked", "--all-targets"],
                        ["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"],
                    ):
                        result = subprocess.run(
                            command, cwd=repo / "apps/service", env=ENVIRONMENT, text=True, capture_output=True
                        )
                        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_the_pruner_knows_every_file_and_region_the_transport_adds(self) -> None:
        self.assertEqual(
            sorted(PRUNER.OWNED_FILES["axum"]["rust"]), sorted(TRANSPORT_PATHS),
        )
        self.assertEqual(PRUNER.OWNED_FILES["axum"]["any"], ("packages/api-client",))
        self.assertEqual(PRUNER.OWNED_FILES_PER_WEB_APP["axum"], ("src/routes", "tests/routes"))
        self.assertIn("axum", PRUNER.APP_SERVICE_FEATURES)
        self.assertEqual(sorted(PRUNER.PACKAGE_EDITS["rust"]["axum"]["packages"]), sorted(TRANSPORT_CRATES))
        for marked in ("src/lib.rs", "src/adapters/mod.rs", "src/bin/serve.rs", "src/config.rs"):
            self.assertIn(marked, PRUNER.MARKED_FILES_BY_LANGUAGE["rust"])
