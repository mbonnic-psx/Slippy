"""`adopt --refresh` over a Cargo root that appears after adoption: reported, never adopted behind anybody's back.

The Go precedent is `tests/test_pin.py`. An adopted repository has no Cargo candidate recorded, and a refresh
does not propose one: it reports the directory as `not wrapped`, and a maintainer adds it on purpose — a
`"generated": false` record in project.json's deployables — and runs the refresh again.
"""
from __future__ import annotations

import json
import tempfile
from pathlib import Path

from support import FactoryTestCase
from test_adopt import node_repository, slipwai
from test_replay import git


class RefreshCargoTest(FactoryTestCase):
    def test_a_cargo_root_added_after_adoption_is_reported_not_wrapped_and_not_recorded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = node_repository(Path(directory))
            self.assertEqual(slipwai(repo, "adopt", "--yes").returncode, 0)
            (repo / "tools/ledger/src").mkdir(parents=True)
            (repo / "tools/ledger/Cargo.toml").write_text('[package]\nname = "ledger"\nversion = "0.1.0"\n')
            (repo / "tools/ledger/src/lib.rs").write_text("pub fn a() -> u32 { 1 }\n")
            git(repo, "add", "-A")
            git(repo, "-c", "user.name=t", "-c", "user.email=t@local", "commit", "-q", "-m", "a crate")

            refreshed = slipwai(repo, "adopt", "--refresh")
            self.assertEqual(refreshed.returncode, 0, refreshed.stderr)
            self.assertIn("not wrapped: `tools/ledger` builds", refreshed.stdout)
            self.assertIn("from `tools/ledger/Cargo.toml`) and has no record", refreshed.stdout)
            self.assertIn("a `\"generated\": false` record in project.json's deployables", refreshed.stdout)
            document = json.loads((repo / "project.json").read_text())
            self.assertEqual(sorted(document["deployables"]), ["shop"], "a candidate is not proposed into the record")
            self.assertNotIn("cargo", (repo / "delivery/Makefile").read_text())
