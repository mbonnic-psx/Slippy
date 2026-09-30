"""The survey's Cargo workspaces: a workspace root is proposed once, at its root, with commands that cover every
member, and none per member; a workspace below another is a candidate of its own. Entered at the survey's boundary
over trees written on disk, as `test_survey_cargo.py` is."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from test_survey import write

from slipwai.survey import buildable, survey


VIRTUAL = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/*"]\nresolver = "2"\n',
    "crates/ledger/Cargo.toml": '[package]\nname = "ledger"\n',
    "crates/report/Cargo.toml": '[package]\nname = "report"\n',
}
FLAGGED = {
    "install": "cargo fetch --locked",
    "typecheck": "cargo check --workspace --all-targets",
    "lint": "cargo clippy --workspace --all-targets --message-format=short -- -D warnings && cargo fmt --check",
    "test": "cargo test --workspace",
}
PLAIN = {
    "install": "cargo fetch --locked",
    "typecheck": "cargo check --all-targets",
    "lint": "cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt --check",
    "test": "cargo test",
}


def commands_of(root: Path, path: str = ".") -> dict[str, str | None]:
    roots = {r.path: r for r in survey(root).roots}
    return roots[path].found.commands


class WorkspaceCommandsTest(unittest.TestCase):
    def proposed(self, manifest: str, path: str = "Cargo.toml") -> dict[str, str | None]:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {path: manifest})
            return commands_of(root, path.removesuffix("/Cargo.toml") if "/" in path else ".")

    def test_a_virtual_workspace_is_proposed_the_workspace_commands_and_nothing_more(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            commands = commands_of(write(Path(directory), VIRTUAL))
        for target, command in FLAGGED.items():
            self.assertEqual(commands[target], command, target)
        for target in ("integration", "adversarial", "audit", "mutation"):
            self.assertIsNone(commands[target], f"{target} is a written no")

    def test_a_manifest_that_is_a_workspace_and_a_package_gets_the_same_commands(self) -> None:
        manifest = '[workspace]\nmembers = ["helper"]\n\n[package]\nname = "app"\n'
        self.assertEqual(self.proposed(manifest), {**FLAGGED, **{t: None for t in (
            "integration", "adversarial", "audit", "mutation")}})

    def test_any_workspace_table_header_makes_a_root(self) -> None:
        for header in ("[workspace.package]", "[workspace.dependencies]", "  [ workspace ]", "\t[workspace]  # root"):
            with self.subTest(header):
                manifest = f'[package]\nname = "app"\n\n{header}\nx = "1"\n'
                self.assertEqual(self.proposed(manifest)["test"], "cargo test --workspace")

    def test_what_only_points_at_a_workspace_is_not_a_root(self) -> None:
        cases = {
            "a dependency": '[dependencies]\nhelper = { path = "h", workspace = true }\n',
            "a member pointing at its root": '[package]\nname = "m"\nworkspace = "../.."\n',
            "a dotted key": '[package]\nname = "m"\npackage.workspace = "../.."\n',
            "a comment": '[package]\nname = "m"\n# [workspace]\n',
            "an array table": '[package]\nname = "m"\n[[workspace.x]]\nk = 1\n',
        }
        for name, manifest in cases.items():
            with self.subTest(name):
                self.assertEqual(self.proposed(manifest), {**PLAIN, **{t: None for t in (
                    "integration", "adversarial", "audit", "mutation")}})

    def test_a_malformed_manifest_is_flagged_exactly_when_a_header_line_is_in_it(self) -> None:
        self.assertEqual(self.proposed("[workspace\nmembers = \n[workspace]\n = = \n")["test"],
                         "cargo test --workspace")
        self.assertEqual(self.proposed("[package\nname = \n")["test"], "cargo test")

    def test_an_unreadable_manifest_is_not_flagged(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"Cargo.toml": "[workspace]\n"})
            (root / "Cargo.toml").chmod(0)
            self.assertEqual(commands_of(root)["test"], "cargo test")

    def test_a_root_crate_with_no_header_keeps_the_single_crate_commands_word_for_word(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            commands = commands_of(write(Path(directory), {"Cargo.toml": '[package]\nname = "a"\n'}))
        for target, command in PLAIN.items():
            self.assertEqual(commands[target], command, target)

    def test_a_workspace_in_a_subdirectory_has_every_command_prefixed_once(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            commands = commands_of(write(Path(directory), {"crates/site/Cargo.toml": "[workspace]\n"}), "crates/site")
        self.assertEqual(commands["install"], "cd crates/site && cargo fetch --locked")
        self.assertEqual(commands["typecheck"], "cd crates/site && cargo check --workspace --all-targets")
        self.assertEqual(
            commands["lint"],
            "cd crates/site && cargo clippy --workspace --all-targets --message-format=short -- -D warnings"
            " && cargo fmt --check",
        )
        self.assertEqual(commands["test"], "cd crates/site && cargo test --workspace")


class NpmWorkspaceRegressionTest(unittest.TestCase):
    def test_an_npm_workspaces_package_below_an_npm_workspace_root_is_still_owned(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "package.json": json.dumps({"name": "top", "workspaces": ["packages/*"]}),
                "packages/app/package.json": json.dumps({"name": "app", "workspaces": ["inner/*"]}),
            })
            self.assertEqual([r.path for r in buildable(root)], ["."], "the nested npm workspace is owned")
