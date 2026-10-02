"""A Rust project's crate is named after the project, and a project may be called what a sysroot crate is called.

`serve` is a binary of the crate, and a crate called `test` or `std` collides with the sysroot's own, so the name is
prefixed as it is for a leading digit. `core` builds and is left alone.
"""
from __future__ import annotations

import os
import subprocess
import tempfile
from pathlib import Path

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
        for name in ("core", "tests", "std-lib", "acme"):
            with self.subTest(project=name):
                self.assertEqual(crate_name(name, first()), name)

    def test_a_later_service_is_already_qualified_and_a_leading_digit_still_is_prefixed(self) -> None:
        self.assertEqual(crate_name("test", App("api", "apps/api", "service", "rust", None, 3001)), "test-api")
        self.assertEqual(crate_name("1st", first()), "app-1st")

    def test_a_project_called_test_builds_with_a_transport(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "test", "standard", "rust", http="axum")
            for arguments in (("build", "--locked", "--all-targets"),):
                result = subprocess.run(
                    ["cargo", *arguments], cwd=repo / "apps/service", text=True, capture_output=True,
                    env={**os.environ, "CARGO_BUILD_JOBS": "2"},
                )
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn('name = "app-test"', (repo / "Cargo.lock").read_text())
