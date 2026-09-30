"""The gate's CI for an adopted repository installs what the recorded commands need before it runs them.

A fresh CI checkout has none of what a laptop that ran `./delivery/init` has: no `node_modules`, no fetched crates,
no projected agent files. `make install` is the target that brings them, the recorded `install` of each application
and the agent projections, so every job that runs the gate or a smoke runs it first. Without it the first lint on
a Node repository could not run (`eslint` is not on the runner), and the ratchet, under `CI`, refuses — so the
workflow `adopt` wrote was red on its first push, on every repository with a package manager.
"""
from __future__ import annotations

import json
import re
import tempfile
from pathlib import Path

from support import FactoryTestCase
from test_adopt import repository, slipwai

NODE = {
    "package.json": '{"name": "shop", "scripts": {"test": "node --test"}}\n',
    "package-lock.json": '{"name": "shop", "lockfileVersion": 3, "requires": true, "packages": {}}\n',
    "index.js": "module.exports = 1;\n",
}


INSTALL, VERIFY, SMOKE = (f"make -f delivery/Makefile {target}" for target in ("install", "verify", "smoke"))


def run_steps(text: str) -> list[str]:
    """Every `run:` step and `script:` line, in order — what the job does, without the setup around it."""
    return re.findall(r"(?m)^\s*(?:- run: |- )(make -f delivery/Makefile \S+)$", text)


class AdoptedCiInstallTest(FactoryTestCase):
    def adopted(self, directory: str, extra: dict[str, str]) -> Path:
        repo = repository(Path(directory), "shop", {**NODE, **extra})
        result = slipwai(repo, "adopt", "--yes")
        self.assertEqual(result.returncode, 0, result.stderr)
        return repo

    def with_smoke(self, repo: Path) -> None:
        """Record a `smoke` for the one application, as /ground does, and let the record drive the files again."""
        document = json.loads((repo / "project.json").read_text())
        document["deployables"]["shop"]["commands"]["smoke"] = "node index.js"
        (repo / "project.json").write_text(json.dumps(document, indent=2) + "\n")
        result = slipwai(repo, "adopt", "--refresh")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_the_github_gate_installs_before_it_verifies(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.adopted(directory, {".github/workflows/ci.yml": "on: push\njobs: {}\n"})

            workflow = (repo / ".github/workflows/verify-delivery.yml").read_text()

            self.assertEqual(run_steps(workflow), [INSTALL, VERIFY])

    def test_the_github_smoke_job_installs_before_it_starts_anything(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.adopted(directory, {".github/workflows/ci.yml": "on: push\njobs: {}\n"})
            self.with_smoke(repo)

            workflow = (repo / ".github/workflows/verify-delivery.yml").read_text()

            smoke = workflow.split("\n  smoke:\n", 1)[1]
            self.assertEqual(run_steps(smoke), [INSTALL, SMOKE])

    def test_the_gitlab_jobs_install_before_they_verify_and_smoke(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.adopted(directory, {".gitlab-ci.yml": "test:\n  script: [true]\n"})
            self.with_smoke(repo)

            job = (repo / "delivery/ci/verify-delivery.gitlab-ci.yml").read_text()

            verify, smoke = job.split("\nsmoke-delivery:\n", 1)
            self.assertEqual(run_steps(verify), [INSTALL, VERIFY])
            self.assertEqual(run_steps(smoke), [INSTALL, SMOKE])
