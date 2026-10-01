"""The ignore lines on the attended path: `slipwai adopt` in a terminal records every directory as a candidate and no
application, and `--confirm` later regenerates the Makefile but never the `.gitignore` block. The block adopt writes
therefore has to carry the Cargo lines for a recorded Cargo candidate, as it does for a recorded Cargo application."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from support import FactoryTestCase
from test_adopt import repository, slipwai
from test_adopt_next import in_terminal
from test_survey_cargo_tools import CRATE, IGNORED, MUTANTS, adopted_block

NODE = {"package.json": json.dumps({"name": "shop", "scripts": {"lint": "eslint ."}})}


class AttendedAdoptionTest(FactoryTestCase):
    def attended(self, parent: Path, files: dict[str, str]) -> Path:
        repo = repository(parent, "ledger", files)
        output = in_terminal(repo, "adopt", "--no-init")
        self.assertTrue((repo / "project.json").is_file(), output)
        return repo

    def test_a_crate_adopted_in_a_terminal_has_the_lines_while_it_is_only_a_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.attended(Path(directory), CRATE)
            record = json.loads((repo / "project.json").read_text())
            block = adopted_block(repo)
        self.assertEqual(record["deployables"], {})
        self.assertIn(IGNORED, block)

    def test_confirming_the_candidate_leaves_the_block_with_each_line_once(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.attended(Path(directory), {**CRATE, MUTANTS: ""})
            confirmed = slipwai(repo, "adopt", "--confirm", "ledger")
            block = adopted_block(repo)
        self.assertEqual(confirmed.returncode, 0, confirmed.stderr)
        self.assertEqual((block.count("mutants.out/\n"), block.count("mutants.out.old/\n")), (1, 1))

    def test_a_node_only_tree_adopted_in_a_terminal_has_the_block_it_always_had(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.attended(Path(directory), NODE)
            block = adopted_block(repo)
        self.assertNotIn("mutants.out", block)


if __name__ == "__main__":
    unittest.main()
