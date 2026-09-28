"""The gate's CI for an adopted repository whose one candidate is Rust: pinned as it is, not as it should be.

`adopted_ci.SETUP` has no `rust` row (SG2 of `specs/001-rust-cargo-adopt`), so the workflow `adopt` writes checks
the code out and runs the gate with no step that installs a Rust toolchain, and the GitLab job has no image for it.
That is a deliberate hole, held here so the `ci-toolchain` slice, which closes it, turns these expectations around
on purpose and not by accident.
"""
from __future__ import annotations

import re
import tempfile
from pathlib import Path

from support import FactoryTestCase
from test_adopt import repository, slipwai

from slipwai.project.adopted_ci import GITLAB_IMAGES, SETUP

CRATE = {
    "Cargo.toml": '[package]\nname = "ledger"\nversion = "0.1.0"\nedition = "2021"\n',
    "Cargo.lock": "# lock\n",
    "src/lib.rs": "pub fn one() -> u32 { 1 }\n",
}


class AdoptedRustCiTest(FactoryTestCase):
    def adopted(self, directory: str, extra: dict[str, str]) -> Path:
        repo = repository(Path(directory), "ledger", {**CRATE, **extra})
        result = slipwai(repo, "adopt", "--yes")
        self.assertEqual(result.returncode, 0, result.stderr)
        return repo

    def test_the_toolchain_kind_a_cargo_crate_records_has_no_setup_row_and_no_gitlab_image(self) -> None:
        # The premise of the pins below, said once: the table this hole lives in.
        self.assertNotIn("rust", SETUP)
        self.assertNotIn("rust", GITLAB_IMAGES)

    def test_a_github_gate_for_a_lone_rust_crate_checks_out_and_runs_the_gate_with_no_toolchain_setup(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.adopted(directory, {".github/workflows/ci.yml": "on: push\njobs: {}\n"})

            workflow = (repo / ".github/workflows/verify-delivery.yml").read_text()

            self.assertEqual(re.findall(r"uses: (\S+)", workflow), ["actions/checkout@v6"])
            self.assertIn("run: make -f delivery/Makefile verify", workflow)
            for setup in ("setup-", "toolchain", "rustup", "rust"):
                self.assertNotIn(setup, workflow.split("steps:", 1)[1])

    def test_a_gitlab_job_for_a_lone_rust_crate_names_rust_as_what_the_image_must_carry(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.adopted(directory, {".gitlab-ci.yml": "test:\n  script: [true]\n"})

            job = (repo / "delivery/ci/verify-delivery.gitlab-ci.yml").read_text()

            self.assertIn("# image: choose one that carries `make` and rust", job)
            self.assertNotRegex(job, r"(?m)^  image:")
