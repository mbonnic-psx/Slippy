"""Scenario 9 — `--http none` generates what Rust generated before there was a transport, byte for byte.

`test_rust_http.py` holds the file set; this holds the bytes, against the digests of a tree generated at `40dacad`
(`test_rust_http_baseline.py`). Every file is compared, not the ones somebody thought of.
"""
from __future__ import annotations

import hashlib
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
                    if digest != before[path] and path not in NAMED_AND_PENDING
                )
                self.assertEqual(changed, [])
