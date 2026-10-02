"""A Rust service serves HTTP through axum — the slice `http-axum`'s factory-level rules.

Every example enters at the boundary (`slipwai generate`, `slipwai migrate`) into a scratch directory and holds
what was written. The generated service's own rules are `#[cfg(test)]` modules in its assets, run by that
project's `cargo test`; this suite is what proves they arrive, and that the answer which takes them away
takes all of them.
"""
from __future__ import annotations

import json
import tempfile
from pathlib import Path

from support import FactoryTestCase, commit_all
from test_migrate import migrate
from test_replay import NEWER, git, newer_factory

from slipwai.catalog import axis_options

# What a Rust service is generated with when no transport is part of the tree: today, whatever it is not asked;
# once the axis is offered, the answer that says so. Both are the same tree, which is the point of pinning it.
PROFILES = ("standard", "event-modelling")
# Recorded exactly, so a transport arriving without being asked for is a change to these lists and not a surprise:
# the repository root's entries (a service with `--frontend none`), and every file under `apps/service/`.
ROOT_FILES = {
    "standard": [
        ".cargo", ".claude", ".editorconfig", ".git", ".github", ".gitignore", ".specify", "AGENTS.md", "Cargo.lock",
        "Cargo.toml", "LICENSE", "Makefile", "README.md", "SECURITY.md", "agents", "apps", "commands", "docs", "init",
        "packages", "project.json", "renovate.json", "rust-toolchain.toml", "scripts", "skills",
    ],
    "event-modelling": [
        ".cargo", ".claude", ".editorconfig", ".env.example", ".git", ".github", ".gitignore", ".specify",
        "AGENTS.md", "Cargo.lock", "Cargo.toml", "LICENSE", "Makefile", "README.md", "SECURITY.md", "agents", "apps",
        "commands", "docker-compose.yml", "docs", "init", "packages", "project.json", "renovate.json",
        "rust-toolchain.toml", "scripts", "skills",
    ],
}
SERVICE_FILES = {
    "standard": [
        "Cargo.toml", "deny.toml", "src/application/README.md", "src/domain/README.md", "src/health.rs",
        "src/lib.rs",
    ],
    "event-modelling": [
        "Cargo.toml", "build.rs", "deny.toml", "migrations/001_events.sql", "migrations/002_events_append_only.sql",
        "migrations/003_projection_checkpoints.sql", "migrations/004_event_tags.sql",
        "src/adapters/driven/checkpoint_store_memory.rs", "src/adapters/driven/checkpoint_store_postgres.rs",
        "src/adapters/driven/event_store_memory.rs", "src/adapters/driven/event_store_postgres.rs",
        "src/adapters/driven/mod.rs", "src/adapters/mod.rs", "src/application/README.md", "src/application/mod.rs",
        "src/application/ports/events.rs", "src/application/ports/mod.rs", "src/application/ports/read_models.rs",
        "src/bin/migrate.rs", "src/checkpoint_store_contract.rs", "src/domain/README.md",
        "src/event_store_contract.rs", "src/health.rs", "src/lib.rs", "src/projections.rs",
    ],
}
TRANSPORT_FILES = (
    "src/bin/serve.rs", "src/adapters/driving", "src/config.rs", "src/observability.rs", "openapi.yaml",
)


def without_transport(test: FactoryTestCase, parent: str, name: str, profile: str) -> Path:
    """A Rust project with no transport: no answer while the axis is not asked, `none` once it is."""
    answers = {"http": "none"} if axis_options("http", "rust", "none") else {}
    return test.generate(parent, name, profile, "rust", **answers)


def listing(root: Path) -> list[str]:
    return sorted(path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file())


class RustWithoutATransportTest(FactoryTestCase):
    """The Pin: what a Rust service is today, which `--http none` must stay and a record made before must keep."""

    def test_a_rust_service_with_no_transport_has_exactly_the_files_it_has_today_on_each_profile(self) -> None:
        for profile in PROFILES:
            with self.subTest(profile=profile), tempfile.TemporaryDirectory() as directory:
                repo = without_transport(self, directory, "quiet", profile)
                service = repo / "apps/service"

                self.assertEqual(listing(service), SERVICE_FILES[profile])
                for transport in TRANSPORT_FILES:
                    self.assertFalse((service / transport).exists(), transport)
                self.assertEqual(sorted(path.name for path in repo.iterdir()), ROOT_FILES[profile])

    def test_a_rust_service_with_no_transport_has_no_run_target_no_service_and_no_transport_keys(self) -> None:
        for profile in PROFILES:
            with self.subTest(profile=profile), tempfile.TemporaryDirectory() as directory:
                repo = without_transport(self, directory, "quiet", profile)

                makefile = (repo / "Makefile").read_text()
                self.assertNotIn("\ndev:", makefile)
                self.assertNotIn("cargo run --locked --bin serve", makefile)
                compose = repo / "docker-compose.yml"
                if compose.exists():
                    # A store may have a container; the service itself, which runs `make dev`, may not.
                    self.assertNotIn("make dev", self.settings(compose.read_text()))
                    declared = self.settings(compose.read_text()).splitlines()
                    self.assertEqual(
                        [line for line in declared if line.startswith("  ") and line.endswith(":") and line[2] != " "],
                        ["  postgres:", "  postgres-data:"],
                    )
                environment = repo / ".env.example"
                if environment.exists():
                    for key in ("HOST", "PORT", "PUBLIC_BASE_URL", "CORS_ALLOWED_ORIGINS", "OTEL_SERVICE_NAME"):
                        self.assertNotIn(f"\n{key}=", environment.read_text())
                selection = json.loads((repo / "project.json").read_text())["deployables"]["service"]["selection"]
                if axis_options("http", "rust", "none"):
                    self.assertEqual(selection["http"], "none")
                else:
                    self.assertNotIn("http", selection)

    def test_a_migrated_record_with_no_http_key_reads_it_as_no_transport(self) -> None:
        """A Rust project generated before the axis was asked records no `http` key. `Selection.option` reads an
        unasked axis as its `absent`, never the catalog default, so a newer factory carries it forward with the
        tree it has — and does not make a transport out of a question the project was never asked."""
        with tempfile.TemporaryDirectory() as directory:
            repo = without_transport(self, directory, "older", "standard")
            record = repo / "project.json"
            metadata = json.loads(record.read_text())
            metadata["deployables"]["service"]["selection"].pop("http", None)
            record.write_text(json.dumps(metadata, indent=2) + "\n")
            if git(repo, "status", "--porcelain").stdout:
                commit_all(repo, "The record as the generator wrote it before the axis was asked")
            factory = newer_factory(Path(directory), "\n## A section a newer factory added\n")

            result = migrate(repo, factory)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(git(repo, "status", "--porcelain").stdout, "")
            service = repo / "apps/service"
            for transport in TRANSPORT_FILES:
                self.assertFalse((service / transport).exists(), transport)
            selection = json.loads(record.read_text())["deployables"]["service"]["selection"]
            self.assertNotIn("http", selection)
            self.assertEqual(json.loads(record.read_text())["generator"]["updatedWith"], NEWER)
