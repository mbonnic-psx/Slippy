"""`slipwai adopt --refresh` over a Cargo crate that gains its tool files after adoption (OG6), and over a tree that is
already configured. Detected commands follow the tree, with the Makefile recipes; confirmed ones are a disagreement
and stay as written; a second refresh on a committed, unchanged tree changes nothing."""
from __future__ import annotations

import json
import tempfile
from pathlib import Path

from support import FactoryTestCase
from test_adopt import repository, slipwai
from test_adopt_next import in_terminal
from test_replay import git
from test_survey_cargo_tools import AUDIT, CRATE, MUTANTS

TOOLS = {"deny.toml": "", MUTANTS: ""}


def commit(repo: Path, message: str) -> None:
    git(repo, "add", "-A")
    if not git(repo, "status", "--porcelain").stdout.strip():
        return
    git(repo, "-c", "user.name=t", "-c", "user.email=t@local", "commit", "-q", "-m", message)


def add_tool_files(repo: Path) -> None:
    for relative, content in TOOLS.items():
        (repo / relative).parent.mkdir(parents=True, exist_ok=True)
        (repo / relative).write_text(content)
    commit(repo, "deny and mutants arrive")


def commands(repo: Path) -> dict:
    return json.loads((repo / "project.json").read_text())["deployables"]["ledger"]["commands"]


def recipe(repo: Path, target: str) -> str:
    text = (repo / "delivery/Makefile").read_text()
    return text.split(f"\n{target}:", 1)[1].split("\n\n", 1)[0]


class RefreshTest(FactoryTestCase):
    def test_a_detected_record_given_both_tool_files_is_refreshed_to_both_commands_and_the_recipes_follow(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "ledger", CRATE)
            self.assertEqual(slipwai(repo, "adopt", "--yes").returncode, 0)
            commit(repo, "adopted")
            self.assertIsNone(commands(repo)["audit"])
            add_tool_files(repo)
            refreshed = slipwai(repo, "adopt", "--refresh")
            self.assertEqual(refreshed.returncode, 0, refreshed.stderr)
            self.assertNotIn("disagrees:", refreshed.stdout)
            after = commands(repo)
            self.assertEqual((after["audit"], after["mutation"]), (AUDIT, "cargo mutants"))
            self.assertIn(AUDIT, recipe(repo, "audit"))
            self.assertIn("cargo mutants", recipe(repo, "mutation"))

    def test_a_confirmed_record_given_the_same_files_is_a_disagreement_and_is_left_as_written(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "ledger", CRATE)
            in_terminal(repo, "adopt", "--no-init")
            self.assertEqual(slipwai(repo, "adopt", "--confirm", "ledger").returncode, 0)
            commit(repo, "confirmed")
            before = commands(repo)
            add_tool_files(repo)
            refreshed = slipwai(repo, "adopt", "--refresh")
            self.assertEqual(refreshed.returncode, 0, refreshed.stderr)
            self.assertIn("disagrees:", refreshed.stdout)
            self.assertEqual(commands(repo), before)
            self.assertIsNone(commands(repo)["audit"])

    def test_a_second_refresh_on_a_configured_committed_tree_changes_nothing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "ledger", {**CRATE, **TOOLS})
            self.assertEqual(slipwai(repo, "adopt", "--yes").returncode, 0)
            commit(repo, "adopted")
            self.assertEqual(slipwai(repo, "adopt", "--refresh").returncode, 0)
            commit(repo, "first refresh")
            again = slipwai(repo, "adopt", "--refresh")
            self.assertEqual(again.returncode, 0, again.stderr)
            self.assertEqual(git(repo, "status", "--porcelain").stdout.strip(), "")


if __name__ == "__main__":
    import unittest

    unittest.main()
