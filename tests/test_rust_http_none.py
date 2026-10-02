"""Scenario 9 — `--http none` generates what Rust generated before there was a transport, byte for byte.

`test_rust_http.py` holds the file set; this holds the bytes, against the digests of a tree generated at `40dacad`
(`test_rust_http_baseline.py`). Every file is compared, not the ones somebody thought of.
"""
from __future__ import annotations

import hashlib
import os
import subprocess
import tempfile
from pathlib import Path

from support import FactoryTestCase
from test_rust_http_baseline import BASELINE

PROFILES = ("standard", "event-modelling")

# The differences that are named and pending, each because the answer is recorded or catalogued rather than
# generated: `project.json` records `http: none` (expected, scenario 9 names it); the README's selection list gains
# a line for the axis; the shipped `scripts/backing-services.py` carries the catalog's `axum` rows. The last two
# await a decision on whether they are acceptable differences — the spec names only `project.json` — and are not
# settled by being listed here. Nothing else may differ.
NAMED_AND_PENDING = {"project.json", "README.md", "scripts/backing-services.py"}
# Re-resolved since the base by `make locks`, which moves transitive crates to their newest patch release and
# changes no dependency a service names; that the lock names the crates the manifest does is
# `test_rust_http_locks.py`'s, and that it resolves is the build's (`--locked`). Not a difference in what is generated.
RE_RESOLVED = {"Cargo.lock"}


def digests(root: Path) -> dict[str, str]:
    return {
        path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()[:12]
        for path in sorted(root.rglob("*"))
        if path.is_file() and ".git" not in path.relative_to(root).parts
    }


class NoTransportIsTheTreeOfBeforeTest(FactoryTestCase):
    def test_every_file_a_rust_project_with_no_transport_has_is_the_one_it_had_before(self) -> None:
        for column, profile in enumerate(PROFILES):
            with self.subTest(profile=profile), tempfile.TemporaryDirectory() as directory:
                repo = self.generate(directory, f"quiet-{profile}", profile, "rust", http="none")
                generated = digests(repo)
                before = {path: pair[column] for path, pair in BASELINE.items() if pair[column] is not None}

                self.assertEqual(sorted(generated), sorted(before))
                changed = sorted(
                    path for path, digest in generated.items()
                    if digest != before[path] and path not in NAMED_AND_PENDING | RE_RESOLVED
                )
                self.assertEqual(changed, [])


class NoTransportBuildsTest(FactoryTestCase):
    def test_a_rust_project_with_no_transport_passes_its_own_gate_on_both_profiles(self) -> None:
        """Scenario 9's promised proof. Since the default became `axum`, nothing else builds a Rust project with no
        transport, so what `--http none` generates is held to the project's own `make verify` here."""
        answers: tuple[tuple[str, dict[str, str]], ...] = (
            ("standard", {}), ("event-modelling", {"event_store": "sqlite"}),
        )
        with tempfile.TemporaryDirectory() as directory:
            for profile, axes in answers:
                with self.subTest(profile=profile):
                    repo = self.generate(directory, f"quiet-{profile}", profile, "rust", http="none", **axes)
                    self.assertFalse((repo / "apps/service/src/bin/serve.rs").exists())
                    result = subprocess.run(
                        ["make", "verify"], cwd=repo, text=True, capture_output=True,
                        env={**os.environ, "CARGO_BUILD_JOBS": "2"},
                    )
                    self.assertEqual(result.returncode, 0, result.stdout[-3000:] + result.stderr[-3000:])
