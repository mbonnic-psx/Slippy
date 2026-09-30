"""A Cargo workspace in a directory an outer build hides (D15): proposed as Cargo where its manifest declares a
workspace, hidden where it does not. Entered at the survey's boundary over trees written on disk, as
`test_survey_cargo_workspace.py` is."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from test_survey import write

from slipwai.quick_wins import missing_lockfiles
from slipwai.survey import buildable

WORKSPACE = '[workspace]\nmembers = ["a"]\nresolver = "2"\n'
NATIVE = {
    "packages/native/package.json": '{"name": "native"}',
    "packages/native/Cargo.toml": WORKSPACE,
    "packages/native/a/Cargo.toml": '[package]\nname = "a"\n',
}
OUTER = {
    "npm": {"package.json": json.dumps({"name": "top", "workspaces": ["packages/*"]})},
    "maven": {"pom.xml": "<project><modules><module>packages/native</module></modules></project>",
              "packages/native/pom.xml": "<project/>"},
    "gradle": {"settings.gradle": "include 'packages:native'\n", "build.gradle": "plugins {}\n",
               "packages/native/build.gradle": "plugins {}\n"},
}


def candidates(files: dict[str, str]) -> list[tuple[str, str]]:
    with tempfile.TemporaryDirectory() as directory:
        return [(r.path, r.found.ecosystem) for r in buildable(write(Path(directory), files))]


class WorkspaceInAnOwnedDirectoryTest(unittest.TestCase):
    def test_a_napi_package_with_its_own_workspace_inside_an_npm_workspace_is_proposed_as_cargo(self) -> None:
        self.assertEqual(candidates({**OUTER["npm"], **NATIVE}), [(".", "node"), ("packages/native", "cargo")])

    def test_it_is_proposed_the_workspace_commands_and_its_member_is_not_proposed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {**OUTER["npm"], **NATIVE})
            found = {r.path: r for r in buildable(root)}
        self.assertEqual(
            found["packages/native"].found.commands["test"], "cd packages/native && cargo test --workspace",
        )
        self.assertNotIn("packages/native/a", found)

    def test_the_same_under_a_maven_modules_root_and_a_gradle_settings_root(self) -> None:
        for outer in ("maven", "gradle"):
            with self.subTest(outer):
                files = {**OUTER[outer], **{k: v for k, v in NATIVE.items() if "package.json" not in k}}
                self.assertEqual(candidates(files), [(".", outer), ("packages/native", "cargo")])

    def test_an_owned_package_with_a_plain_crate_manifest_stays_hidden(self) -> None:
        files = {**OUTER["npm"], "packages/native/package.json": "{}", "packages/native/Cargo.toml": '[package]\n'}
        self.assertEqual(candidates(files), [(".", "node")])

    def test_an_owned_package_with_a_go_module_stays_hidden(self) -> None:
        files = {**OUTER["npm"], "packages/native/package.json": "{}", "packages/native/go.mod": "module m\n"}
        self.assertEqual(candidates(files), [(".", "node")])

    def test_a_directory_that_is_not_owned_stays_the_ecosystem_that_detected_it(self) -> None:
        files = {"packages/native/package.json": '{"name": "native"}', **{
            k: v for k, v in NATIVE.items() if "package.json" not in k}}
        self.assertEqual(candidates(files), [("packages/native", "node")])

    def test_the_lockfile_rule_agrees_on_each_tree(self) -> None:
        """The member needs no lock beside it in either tree; the crate that is a build, or is a plain crate the
        survey leaves to its owner, needs its own."""
        trees = {
            "workspace": ({**OUTER["npm"], **NATIVE}, {"packages/native"}),
            "plain": ({**OUTER["npm"], "packages/native/package.json": "{}",
                       "packages/native/Cargo.toml": "[package]\n"}, {"packages/native"}),
        }
        for name, (files, expected) in trees.items():
            with self.subTest(name), tempfile.TemporaryDirectory() as directory:
                root = write(Path(directory), files)
                crates = {Path(f).parent.as_posix() for f in files if f.endswith("Cargo.toml")}
                proposed = {r.path for r in buildable(root)} & crates
                unlocked = {Path(f.where).parent.as_posix() for f in missing_lockfiles(root, sorted(files))
                            if f.where.endswith("Cargo.toml")}
                self.assertEqual(unlocked, expected)
                if name == "workspace":
                    self.assertEqual(proposed, unlocked, "a crate is a build exactly when it is not a member")


if __name__ == "__main__":
    unittest.main()
