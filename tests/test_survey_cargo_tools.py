"""The Cargo row's optional tools: audit and mutation are proposed where the candidate's directory configures them,
and are a written no elsewhere. Entered at the survey's boundary over trees written on disk, and at adopt's through
`slipwai adopt --yes` in a throwaway repository. Fakes only: the trees are real files, and no tool is run."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from test_adopt import node_repository, repository, slipwai
from test_survey import write

from slipwai.survey import survey

CRATE = {"Cargo.toml": '[package]\nname = "ledger"\nversion = "0.1.0"\n', "src/lib.rs": ""}
BEGIN = "# slipwai:delivery:begin"


def adopted_block(repo: Path) -> str:
    """The marked block adopt appended to `.gitignore`: what the factory added, and nothing of the repository's."""
    return (repo / ".gitignore").read_text().split(BEGIN, 1)[1]


def candidate(root: Path, path: str = ".") -> dict[str, str | None]:
    return {r.path: r for r in survey(root).roots}[path].found.commands


class PinnedBeforeTheSliceTest(unittest.TestCase):
    """Today's answers, observed green before any production change (the plan's Pin)."""

    def test_a_crate_with_none_of_the_tool_files_is_offered_neither_audit_nor_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            commands = candidate(write(Path(directory), CRATE))
        self.assertIsNone(commands["audit"])
        self.assertIsNone(commands["mutation"])

    def test_a_crate_with_none_of_the_tool_files_is_adopted_with_both_recorded_as_null(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "ledger", CRATE)
            result = slipwai(repo, "adopt", "--yes")
            self.assertEqual(result.returncode, 0, result.stderr)
            commands = json.loads((repo / "project.json").read_text())["deployables"]["ledger"]["commands"]
        self.assertIn("audit", commands)
        self.assertIsNone(commands["audit"])
        self.assertIn("mutation", commands)
        self.assertIsNone(commands["mutation"])

    def test_an_adopted_node_repository_block_carries_no_language_line_the_factory_added(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = node_repository(Path(directory))
            result = slipwai(repo, "adopt", "--yes")
            self.assertEqual(result.returncode, 0, result.stderr)
            block = adopted_block(repo)
        self.assertNotIn("node_modules/", block)
        self.assertNotIn("mutants.out/", block)

    def test_an_adopted_crate_block_has_no_mutants_out_which_the_ignore_rule_inverts_on_purpose(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "ledger", CRATE)
            result = slipwai(repo, "adopt", "--yes")
            self.assertEqual(result.returncode, 0, result.stderr)
            block = adopted_block(repo)
        self.assertNotIn("mutants.out/", block)


WORKSPACE = {"Cargo.toml": '[workspace]\nmembers = ["m"]\n\n[package]\nname = "ledger"\n',
             "m/Cargo.toml": '[package]\nname = "m"\n'}
AUDIT = "cargo deny check advisories"
SITE = {"crates/site/Cargo.toml": '[package]\nname = "site"\n'}
SITE_WORKSPACE = {"crates/site/Cargo.toml": '[workspace]\nmembers = ["m"]\n', "crates/site/m/Cargo.toml": ""}


def audit_of(files: dict[str, str], path: str = ".") -> str | None:
    with tempfile.TemporaryDirectory() as directory:
        return candidate(write(Path(directory), files), path)["audit"]


class AuditTest(unittest.TestCase):
    def test_a_deny_toml_in_the_candidates_directory_proposes_cargo_deny_check_advisories_as_audit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            commands = candidate(write(Path(directory), {**CRATE, "deny.toml": ""}))
        self.assertEqual(commands["audit"], AUDIT)
        self.assertIsNone(commands["mutation"])

    def test_each_of_the_other_two_names_cargo_deny_reads_proposes_it_too(self) -> None:
        for name in (".deny.toml", ".cargo/deny.toml"):
            self.assertEqual(audit_of({**CRATE, name: ""}), AUDIT, name)

    def test_none_of_the_three_names_present_proposes_no_audit(self) -> None:
        self.assertIsNone(audit_of({**CRATE, "mutants.toml": "", "cargo-deny.toml": "", "deny.toml.bak": ""}))

    def test_a_deny_toml_only_in_a_member_or_only_above_the_candidate_proposes_nothing(self) -> None:
        self.assertIsNone(audit_of({**WORKSPACE, "m/deny.toml": ""}))
        self.assertIsNone(audit_of({**SITE, "deny.toml": "", "Cargo.toml": '[package]\nname = "r"\n'}, "crates/site"))

    def test_a_directory_named_deny_toml_is_not_a_configuration(self) -> None:
        self.assertIsNone(audit_of({**CRATE, "deny.toml/keep": ""}))

    def test_a_workspace_root_is_audited_with_workspace_and_a_subdirectory_is_prefixed_once(self) -> None:
        self.assertEqual(audit_of({**WORKSPACE, "deny.toml": ""}), "cargo deny --workspace check advisories")
        self.assertEqual(
            audit_of({**SITE_WORKSPACE, "crates/site/deny.toml": ""}, "crates/site"),
            "cd crates/site && cargo deny --workspace check advisories",
        )
        self.assertEqual(audit_of({**SITE, "crates/site/deny.toml": ""}, "crates/site"),
                         "cd crates/site && cargo deny check advisories")


MUTANTS = ".cargo/mutants.toml"


def mutation_of(files: dict[str, str], path: str = ".") -> str | None:
    with tempfile.TemporaryDirectory() as directory:
        return candidate(write(Path(directory), files), path)["mutation"]


class MutationTest(unittest.TestCase):
    def test_a_cargo_mutants_toml_in_the_candidates_directory_proposes_cargo_mutants_as_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            commands = candidate(write(Path(directory), {**CRATE, MUTANTS: ""}))
        self.assertEqual(commands["mutation"], "cargo mutants")
        self.assertIsNone(commands["audit"])

    def test_a_bare_mutants_toml_or_a_directory_of_that_name_proposes_nothing(self) -> None:
        self.assertIsNone(mutation_of({**CRATE, "mutants.toml": ""}))
        self.assertIsNone(mutation_of({**CRATE, f"{MUTANTS}/keep": ""}))

    def test_a_members_mutants_toml_is_not_the_one_read_at_the_workspace_root(self) -> None:
        self.assertIsNone(mutation_of({**WORKSPACE, f"m/{MUTANTS}": ""}))

    def test_a_workspace_root_is_mutated_with_workspace_and_a_subdirectory_is_prefixed_once(self) -> None:
        self.assertEqual(mutation_of({**WORKSPACE, MUTANTS: ""}), "cargo mutants --workspace")
        self.assertEqual(
            mutation_of({**SITE_WORKSPACE, f"crates/site/{MUTANTS}": ""}, "crates/site"),
            "cd crates/site && cargo mutants --workspace",
        )
        self.assertEqual(mutation_of({**SITE, f"crates/site/{MUTANTS}": ""}, "crates/site"),
                         "cd crates/site && cargo mutants")

    def test_both_files_at_once_give_both_commands_independently_and_neither_is_guarded_or_scoped(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            commands = candidate(write(Path(directory), {**CRATE, "deny.toml": "", MUTANTS: ""}))
        self.assertEqual((commands["audit"], commands["mutation"]), (AUDIT, "cargo mutants"))
        for command in (commands["audit"], commands["mutation"]):
            for text in ("command -v", "--in-diff", "$(", "${"):
                self.assertNotIn(text, command or "")


if __name__ == "__main__":
    unittest.main()
