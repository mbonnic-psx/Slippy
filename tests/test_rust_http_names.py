"""A Rust project's crate is named after the project, and a project may be called what a sysroot crate is called.

`serve` is a binary of the crate, and a crate called `test` or `std` collides with the sysroot's own, so the name is
prefixed as it is for a leading digit. So is `core`, a keyword, and any crate the manifest can declare: a package
named `axum` or `tokio` is a dependency cycle or a `use` that means the wrong crate.
"""
from __future__ import annotations

import os
import subprocess
import tempfile

from support import FactoryTestCase

from slipwai.project.languages.rust import crate_name
from slipwai.services import App

RESERVED = ("test", "std", "alloc", "proc_macro")


def first(name: str = "service") -> App:
    return App(name, f"apps/{name}", "service", "rust", None, 3000, first=True)


class CrateNameTest(FactoryTestCase):
    def test_every_reserved_sysroot_crate_name_is_prefixed(self) -> None:
        for name in RESERVED:
            with self.subTest(project=name):
                self.assertEqual(crate_name(name, first()), f"app-{name}")

    def test_a_reserved_name_is_matched_whatever_its_case_and_only_whole(self) -> None:
        self.assertEqual(crate_name("Test", first()), "app-test")
        self.assertEqual(crate_name("proc-macro", first()), "app-proc-macro")
        for name in ("tests", "std-lib", "acme"):
            with self.subTest(project=name):
                self.assertEqual(crate_name(name, first()), name)

    def test_a_declared_crate_a_keyword_and_core_are_prefixed(self) -> None:
        from slipwai.project.languages.cargo import declared_crates

        self.assertLessEqual({"axum", "tokio", "tower", "tracing", "opentelemetry", "serde", "sqlx"}, declared_crates())
        for name in ("axum", "tokio", "tower", "self", "type", "core", "crate", "super", "async", "try", "gen",
                     "opentelemetry-otlp", "tracing_subscriber", "Serde"):
            with self.subTest(project=name):
                self.assertEqual(crate_name(name, first()), f"app-{name.lower()}")

    def test_a_declared_crate_names_a_later_service_only_when_the_qualified_name_is(self) -> None:
        self.assertEqual(crate_name("acme", App("tokio", "apps/tokio", "service", "rust", None, 3001)), "acme-tokio")

    def test_a_later_service_is_already_qualified_and_a_leading_digit_still_is_prefixed(self) -> None:
        self.assertEqual(crate_name("test", App("api", "apps/api", "service", "rust", None, 3001)), "test-api")
        self.assertEqual(crate_name("1st", first()), "app-1st")

    def test_a_project_called_test_builds_with_a_transport(self) -> None:
        self.assertBuildsAs("test", "app-test")

    def test_projects_named_after_a_crate_or_a_keyword_build_with_a_transport(self) -> None:
        for name in ("axum", "tokio", "self", "type", "core"):
            with self.subTest(project=name):
                self.assertBuildsAs(name, f"app-{name}")

    def assertBuildsAs(self, project: str, crate: str) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, project, "standard", "rust", http="axum")
            result = subprocess.run(
                ["cargo", "build", "--locked", "--all-targets"], cwd=repo / "apps/service", text=True,
                capture_output=True, env={**os.environ, "CARGO_BUILD_JOBS": "2"},
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn(f'name = "{crate}"', (repo / "Cargo.lock").read_text())
