"""Each manifest a Cargo workspace root is asked about is read once per survey, however many members sit below it
(adversary W4): a counting read, a function in this tree, is passed through the seam the rules take."""
from __future__ import annotations

import tempfile
import unittest
from collections import Counter
from pathlib import Path

from test_survey import write

from slipwai.bounded_read import read
from slipwai.quick_wins import missing_lockfiles
from slipwai.survey import buildable

MEMBERS = 40
FILES = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/*"]\n',
    **{f"crates/m{n}/Cargo.toml": f'[package]\nname = "m{n}"\n' for n in range(MEMBERS)},
}


class Counting:
    """The real read, counting what it was asked for."""

    def __init__(self) -> None:
        self.asked: Counter[Path] = Counter()

    def __call__(self, path: Path) -> str:
        self.asked[path] += 1
        return read(path)


class EachManifestIsReadOnceTest(unittest.TestCase):
    def test_the_survey_reads_the_root_once_for_all_its_members(self) -> None:
        counting = Counting()
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), FILES)
            self.assertEqual([r.path for r in buildable(root, reader=counting)], ["."])
            self.assertEqual(counting.asked[root / "Cargo.toml"], 1)
        self.assertEqual(max(counting.asked.values()), 1, "no manifest is read twice")

    def test_the_lockfile_rule_reads_the_root_once_for_all_its_members(self) -> None:
        counting = Counting()
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), FILES)
            found = missing_lockfiles(root, sorted(FILES), reader=counting)
            self.assertEqual([f.where for f in found], ["Cargo.toml"])
            self.assertEqual(counting.asked[root / "Cargo.toml"], 1)
        self.assertEqual(max(counting.asked.values()), 1, "no manifest is read twice")

    def test_a_second_survey_reads_afresh_because_nothing_outlives_the_call(self) -> None:
        counting = Counting()
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), FILES)
            buildable(root, reader=counting)
            (root / "Cargo.toml").write_text("[package]\nname = 'r'\n")
            self.assertEqual(
                sorted(r.path for r in buildable(root, reader=counting))[:2], [".", "crates/m0"],
                "the edited root no longer owns its crates",
            )


if __name__ == "__main__":
    unittest.main()
