"""Where a pinned Rust crate's toolchain goes once `adopt --yes` has recorded it, and where it does not (TG9).

The pin is recorded as the candidate's version. The adopted CI sets up no Rust in this slice, so the GitHub workflow
names neither a Rust setup action nor the version; the GitLab job's comment, `docs/adoption.md` and `/ground`'s
Platform line read it and say `rust 1.85`. Each test adopts the crate through the command and reads the file.
"""
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from support import FactoryTestCase
from test_adopt import repository, slipwai
from test_survey_cargo_toolchain import CRATE, toml

GITHUB = {".github/workflows/ci.yml": "on: push\njobs: {}\n"}
GITLAB = {".gitlab-ci.yml": "test:\n  script: [true]\n"}


class PinnedCrateHandOverTest(FactoryTestCase):
    def adopted(self, directory: str, forge: dict[str, str]) -> Path:
        """A crate pinned to 1.85, beside the CI files of `forge`, adopted."""
        files = {"Cargo.toml": CRATE, "rust-toolchain.toml": toml("1.85"), **forge}
        repo = repository(Path(directory), "adopted", files)
        result = slipwai(repo, "adopt", "--yes")
        self.assertEqual(result.returncode, 0, result.stderr)
        return repo

    def test_the_github_workflow_names_no_rust_setup_and_not_the_pin(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            workflow = (self.adopted(directory, GITHUB) / ".github/workflows/verify-delivery.yml").read_text()
        self.assertNotIn("1.85", workflow)
        self.assertNotIn("rust", workflow.lower())

    def test_the_gitlab_job_comment_names_rust_and_the_pin(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            job = (self.adopted(directory, GITLAB) / "delivery/ci/verify-delivery.gitlab-ci.yml").read_text()
        self.assertIn("# image: choose one that carries `make` and rust 1.85, or install them in before_script", job)

    def test_the_adoption_page_names_the_pin_beside_the_application(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            page = (self.adopted(directory, GITHUB) / "delivery/docs/adoption.md").read_text()
        self.assertIn("(cargo, rust 1.85)", page)

    def test_ground_reports_the_runtime_as_detected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            ground = (self.adopted(directory, GITHUB) / "delivery/commands/ground.md").read_text()
        self.assertIn("runs on rust 1.85 (detected)", ground)


if __name__ == "__main__":
    unittest.main()
