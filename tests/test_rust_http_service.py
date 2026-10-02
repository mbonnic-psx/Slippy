"""R3 to R8 — the generated service's own rules, proved by the project's own `cargo test`.

The rules themselves are `#[cfg(test)]` modules in the assets (`assets/backing-services/rust/`), written as the
service's tests and dispatched through `Router::oneshot` and `config::load_from`, so none of them opens a socket
or reads this process's environment. What this suite proves is that they *arrive* in a generated project and
that cargo runs them there: each example names the filter a rule's tests live under and a count they must reach,
so a rule whose tests were not generated cannot pass as a filter that matched nothing.
"""
from __future__ import annotations

import os
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

from support import FactoryTestCase

CARGO_ENVIRONMENT = {**os.environ, "CARGO_BUILD_JOBS": "2"}


class GeneratedServiceTest(FactoryTestCase):
    """One standard-profile Rust service on `axum`, generated once and tested per rule."""

    directory: tempfile.TemporaryDirectory[str]
    repo: Path

    @classmethod
    def setUpClass(cls) -> None:
        cls.directory = tempfile.TemporaryDirectory()
        cls.repo = cls().generate(cls.directory.name, "served", "standard", "rust", http="axum")

    @classmethod
    def tearDownClass(cls) -> None:
        cls.directory.cleanup()

    def cargo_test(self, filter_: str, at_least: int) -> None:
        """Runs the service's tests under `filter_` and requires that at least `at_least` ran and all passed."""
        result = subprocess.run(
            ["cargo", "test", "--locked", "--lib", filter_],
            cwd=self.repo / "apps/service", env=CARGO_ENVIRONMENT, text=True, capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        ran = re.search(r"test result: ok\. (\d+) passed", result.stdout)
        self.assertIsNotNone(ran, result.stdout)
        assert ran is not None
        self.assertGreaterEqual(int(ran.group(1)), at_least, f"only {ran.group(1)} tests ran under {filter_}")

    def cargo(self, *arguments: str) -> None:
        result = subprocess.run(
            ["cargo", *arguments], cwd=self.repo / "apps/service", env=CARGO_ENVIRONMENT, text=True, capture_output=True
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_what_has_been_generated_is_formatted_and_lint_clean(self) -> None:
        """The gate every generated project runs, over every module the transport has added so far."""
        self.cargo("fmt", "--check")
        self.cargo("clippy", "--locked", "--all-targets", "--", "-D", "warnings")

    def test_the_adapter_s_routes_are_held_by_its_own_tests(self) -> None:
        self.cargo_test("adapters::driving::http::tests", at_least=13)

    def test_what_a_browser_meets_first_is_held_by_the_wrapper_s_own_tests(self) -> None:
        self.cargo_test("adapters::driving::http::security", at_least=7)


if __name__ == "__main__":
    unittest.main()
