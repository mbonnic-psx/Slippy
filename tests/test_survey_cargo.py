"""The survey's Cargo row: a repository holding a `Cargo.toml` is proposed as Rust built by Cargo, with Cargo's own
tools for the targets Cargo answers and a written no for the rest — the same properties `test_survey.py` holds every
other ecosystem to, kept in a suite of their own so each stays readable end to end.
"""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from test_adopt import repository, slipwai
from test_survey import write

from slipwai.survey import Root, buildable, survey, toolchain_as


class CargoSurveyTest(unittest.TestCase):
    def only(self, roots: tuple[Root, ...]) -> Root:
        self.assertEqual(len(roots), 1, f"exactly one candidate expected, got {[r.path for r in roots]}")
        return roots[0]

    def test_a_directory_holding_a_cargo_toml_is_proposed_as_rust_built_by_cargo_with_its_commands(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "Cargo.toml": '[package]\nname = "ledger"\nversion = "0.1.0"\nedition = "2021"\n',
                "Cargo.lock": "", "src/lib.rs": "",
                "rust-toolchain.toml": '[toolchain]\nchannel = "1.79.0"\n',
            })
            found = survey(root)
            self.assertEqual([r.path for r in found.roots], ["."], "one crate is one candidate")
            crate = found.roots[0].found
            self.assertEqual((crate.language, crate.ecosystem, crate.evidence), ("rust", "cargo", "Cargo.toml"))
            self.assertEqual(crate.commands["install"], "cargo fetch --locked")
            self.assertEqual(crate.commands["typecheck"], "cargo check --all-targets")
            self.assertEqual(
                crate.commands["lint"],
                "cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt --check",
            )
            self.assertEqual(crate.commands["test"], "cargo test")
            for target in ("integration", "adversarial", "audit", "mutation"):
                self.assertIn(target, crate.commands)
                self.assertIsNone(crate.commands[target], f"{target} is a written no")
            self.assertEqual(crate.toolchain, {"kind": "rust", "version": ""}, "a pin in the tree is a later slice's")
            self.assertIsNone(crate.packaging)
            self.assertIn("rust", found.languages)
            self.assertIsNone(found.roots[0].role, "nothing beside the crate says what it is for")

    def test_a_crate_in_a_subdirectory_has_every_command_prefixed_once(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"crates/ledger/Cargo.toml": '[package]\nname = "ledger"\n'})
            found = self.only(survey(root).roots)
            self.assertEqual((found.path, found.found.evidence), ("crates/ledger", "crates/ledger/Cargo.toml"))
            commands = found.found.commands
            self.assertEqual(commands["install"], "cd crates/ledger && cargo fetch --locked")
            self.assertEqual(commands["typecheck"], "cd crates/ledger && cargo check --all-targets")
            self.assertEqual(
                commands["lint"],
                "cd crates/ledger && cargo clippy --all-targets --message-format=short -- -D warnings"
                " && cargo fmt --check",
            )
            self.assertEqual(commands["test"], "cd crates/ledger && cargo test")

    def test_a_malformed_cargo_toml_and_a_missing_lockfile_are_still_cargo(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"Cargo.toml": "[package\nname = \n"})
            found = self.only(survey(root).roots)
            self.assertEqual(found.found.ecosystem, "cargo")
            self.assertEqual(found.found.commands["install"], "cargo fetch --locked")

    def test_a_cargo_toml_in_a_directory_that_is_never_surveyed_is_not_proposed(self) -> None:
        """Each case keeps its `Cargo.toml` within `DEPTH`, so removing that one name from `SKIPPED` proposes it."""
        cases: tuple[tuple[str, str, frozenset[str]], ...] = (
            ("target", "target/Cargo.toml", frozenset()),
            ("vendor", "vendor/Cargo.toml", frozenset()),
            ("skipped fixture", "fixtures/Cargo.toml", frozenset({"fixtures"})),
        )
        for name, manifest, skipped in cases:
            with self.subTest(name), tempfile.TemporaryDirectory() as directory:
                root = write(Path(directory), {manifest: "[package]\n"})
                self.assertEqual(buildable(root, skipped), (), f"{manifest} is under a directory never surveyed")

    def test_a_package_json_beside_a_cargo_toml_is_reported_once_as_node(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "Cargo.toml": '[package]\nname = "native"\n', "package.json": json.dumps({"name": "native"}),
            })
            found = self.only(survey(root).roots)
            self.assertEqual((found.found.ecosystem, found.found.evidence), ("node", "package.json"))

    def test_saying_a_mixed_node_and_cargo_directory_is_rust_records_cargo_and_a_refresh_changes_nothing(self) -> None:
        files = {
            "Cargo.toml": '[package]\nname = "native"\n', "package.json": json.dumps({"name": "native"}),
            "README.md": "# native\n",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), files)
            self.assertEqual(
                toolchain_as(root, ".", "rust"), {"kind": "rust", "version": "", "ecosystem": "cargo"},
                "the survey reads Node here, and somebody saying Rust is saying which build is the application's",
            )
            repo = repository(Path(directory), "adopted", files)
            found = survey(repo).roots[0].found
            self.assertEqual(found.ecosystem, "node", "without the word, Node")
            result = slipwai(repo, "adopt", "--yes", "--language", f"{repo.name}=rust")
            self.assertEqual(result.returncode, 0, result.stderr)
            deployable = json.loads((repo / "project.json").read_text())["deployables"][repo.name]
            self.assertEqual(deployable["language"], "rust")
            self.assertEqual(deployable["toolchain"], {"kind": "rust", "version": "", "ecosystem": "cargo"})
            self.assertEqual(deployable["provenance"]["language"], "overridden")
            self.assertEqual(deployable["provenance"]["toolchain"], "overridden")
            refreshed = slipwai(repo, "adopt", "--refresh")
            self.assertEqual(refreshed.returncode, 0, refreshed.stderr)
            self.assertIn("refreshed: nothing", refreshed.stdout)

    def test_a_fuzz_crate_under_a_root_crate_is_a_candidate_of_its_own(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "Cargo.toml": '[package]\nname = "ledger"\n', "fuzz/Cargo.toml": '[package]\nname = "ledger-fuzz"\n',
            })
            found = survey(root)
            self.assertEqual([r.path for r in found.roots], [".", "fuzz"])
            self.assertEqual(found.roots[1].found.commands["test"], "cd fuzz && cargo test")

    def test_a_dockerfile_beside_a_crate_makes_it_a_service(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"Cargo.toml": '[package]\nname = "api"\n', "Dockerfile": "FROM rust\n"})
            found = self.only(survey(root).roots)
            self.assertEqual((found.role, found.role_evidence), ("service", "Dockerfile"))
