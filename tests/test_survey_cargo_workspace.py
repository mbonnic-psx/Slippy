"""The survey's Cargo workspaces: a workspace root is proposed once, at its root, with commands that cover every
member, and none per member; a workspace below another is a candidate of its own. Entered at the survey's boundary
over trees written on disk, as `test_survey_cargo.py` is."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from test_adopt import repository, slipwai
from test_survey import write

from slipwai.quick_wins import missing_lockfiles
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

    def test_a_table_whose_name_only_starts_with_workspace_is_not_a_root(self) -> None:
        for header in ("[workspacefoo]", "[workspace-x]", "[workspaces]", "[workspace_x]", "[ workspace x ]"):
            with self.subTest(header):
                self.assertEqual(self.proposed(f'[package]\nname = "a"\n\n{header}\nx = 1\n')["test"],
                                 "cargo test")

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


class WorkspaceOwnsItsMembersTest(unittest.TestCase):
    def paths(self, files: dict[str, str]) -> list[str]:
        with tempfile.TemporaryDirectory() as directory:
            return [r.path for r in buildable(write(Path(directory), files))]

    def test_a_virtual_workspace_is_one_candidate_at_its_root(self) -> None:
        self.assertEqual(self.paths(VIRTUAL), ["."], "no member is proposed")

    def test_a_root_that_is_a_workspace_and_a_package_is_still_one_candidate(self) -> None:
        files = {
            "Cargo.toml": '[workspace]\nmembers = ["helper"]\n\n[package]\nname = "app"\n',
            "helper/Cargo.toml": '[package]\nname = "helper"\n',
        }
        self.assertEqual(self.paths(files), ["."])

    def test_a_member_is_owned_wherever_it_sits_below_the_root_and_members_is_not_read(self) -> None:
        files = {
            "Cargo.toml": '[workspace]\nmembers = ["elsewhere"]\n',
            "a/b/Cargo.toml": '[package]\nname = "deep"\n',
            "other/Cargo.toml": '[package]\nname = "other"\n',
        }
        self.assertEqual(self.paths(files), ["."])

    def test_a_plain_root_crate_with_a_fuzz_crate_keeps_two_candidates(self) -> None:
        files = {"Cargo.toml": '[package]\nname = "a"\n', "fuzz/Cargo.toml": '[package]\nname = "a-fuzz"\n'}
        self.assertEqual(self.paths(files), [".", "fuzz"])


TAURI = {
    "package.json": json.dumps({"name": "cairn", "private": True, "scripts": {"test": "node --test"}}),
    "package-lock.json": "{}\n", "README.md": "# cairn\n",
    "src-tauri/Cargo.toml": (
        '[workspace]\nmembers = ["helper"]\n\n[workspace.package]\nedition = "2021"\n\n'
        '[workspace.dependencies]\nserde = "1"\n\n[package]\nname = "cairn"\n\n[lib]\nname = "cairn_lib"\n\n'
        '[[bin]]\nname = "cairn"\n\n[features]\napp = []\n'
    ),
    "src-tauri/helper/Cargo.toml": '[package]\nname = "helper"\n',
}


class TauriShapeTest(unittest.TestCase):
    def test_a_crate_with_its_members_below_a_node_root_is_proposed_once_beside_node(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            roots = buildable(write(Path(directory), TAURI))
        self.assertEqual([(r.path, r.found.ecosystem, r.found.evidence) for r in roots], [
            (".", "node", "package.json"), ("src-tauri", "cargo", "src-tauri/Cargo.toml"),
        ])
        commands = roots[1].found.commands
        self.assertEqual(commands["typecheck"], "cd src-tauri && cargo check --workspace --all-targets")
        self.assertEqual(commands["test"], "cd src-tauri && cargo test --workspace")

    def test_adopting_it_records_exactly_those_two_deployables(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "cairn", TAURI)
            result = slipwai(repo, "adopt", "--yes")
            self.assertEqual(result.returncode, 0, result.stderr)
            deployables = json.loads((repo / "project.json").read_text())["deployables"]
        self.assertEqual(sorted(d["path"] for d in deployables.values()), [".", "src-tauri"])


class RecordedBeforeTheRuleTest(unittest.TestCase):
    def test_a_refresh_updates_the_root_and_reports_a_member_recorded_by_an_earlier_snapshot(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "shop", {**VIRTUAL, "README.md": "# shop\n"})
            self.assertEqual(slipwai(repo, "adopt", "--yes").returncode, 0)
            record = json.loads((repo / "project.json").read_text())
            root = record["deployables"]["shop"]
            for target in ("typecheck", "lint", "test"):
                root["commands"][target] = root["commands"][target].replace(" --workspace", "")
            member = json.loads(json.dumps(root))
            member["path"] = "crates/ledger"
            member["commands"] = {t: f"cd crates/ledger && {c}" if c else None for t, c in root["commands"].items()}
            record["deployables"]["ledger"] = member
            (repo / "project.json").write_text(json.dumps(record, indent=2) + "\n")
            refreshed = slipwai(repo, "adopt", "--refresh")
            self.assertEqual(refreshed.returncode, 0, refreshed.stderr)
            after = json.loads((repo / "project.json").read_text())["deployables"]
        self.assertIn("shop: commands refreshed from `Cargo.toml`", refreshed.stdout)
        self.assertEqual(after["shop"]["commands"]["test"], "cargo test --workspace")
        self.assertIn(
            "ledger: nothing the survey recognises builds at `crates/ledger` any more; its record stands as written",
            refreshed.stdout,
        )
        self.assertEqual(after["ledger"], member, "the member's record is left exactly as written")


NESTED = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/*"]\n',
    "crates/ledger/Cargo.toml": '[package]\nname = "ledger"\n',
    "fuzz/Cargo.toml": '[package]\nname = "ledger-fuzz"\n\n[workspace]\nmembers = ["."]\n',
}


class NestedWorkspaceTest(unittest.TestCase):
    def test_a_workspace_below_a_workspace_root_is_a_candidate_of_its_own(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            roots = buildable(write(Path(directory), NESTED))
        self.assertEqual([r.path for r in roots], [".", "fuzz"], "the member is owned, the nested root is not")
        commands = roots[1].found.commands
        self.assertEqual(commands["typecheck"], "cd fuzz && cargo check --workspace --all-targets")
        self.assertEqual(commands["test"], "cd fuzz && cargo test --workspace")

    def test_the_nested_root_owns_what_is_below_it(self) -> None:
        files = {**NESTED, "fuzz/targets/Cargo.toml": '[package]\nname = "t"\n'}
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual([r.path for r in buildable(write(Path(directory), files))], [".", "fuzz"])

    def test_a_plain_member_with_no_header_stays_owned_under_a_root_beside_a_nested_one(self) -> None:
        files = {**NESTED, "crates/ledger/tools/Cargo.toml": '[package]\nname = "tools"\n'}
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual([r.path for r in buildable(write(Path(directory), files))], [".", "fuzz"])


class NpmWorkspaceRegressionTest(unittest.TestCase):
    def test_an_npm_workspaces_package_below_an_npm_workspace_root_is_still_owned(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "package.json": json.dumps({"name": "top", "workspaces": ["packages/*"]}),
                "packages/app/package.json": json.dumps({"name": "app", "workspaces": ["inner/*"]}),
            })
            self.assertEqual([r.path for r in buildable(root)], ["."], "the nested npm workspace is owned")


ROOTS = {
    "node": {"package.json": '{"name": "napi"}'}, "python": {"pyproject.toml": "[project]\nname = 'maturin'\n"},
    "go": {"go.mod": "module m\n\ngo 1.22\n"}, "maven": {"pom.xml": "<project/>"},
    "gradle": {"build.gradle": "plugins {}\n"}, "ant": {"build.xml": "<project/>"},
    "dotnet": {"app.csproj": "<Project/>"}, "php": {"composer.json": "{}"}, "ruby": {"Gemfile": ""},
}
RUST = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/*"]\n',
    "crates/a/Cargo.toml": '[package]\nname = "a"\n', "crates/b/Cargo.toml": '[package]\nname = "b"\n',
}


class WorkspaceRootBesideAnEarlierManifestTest(unittest.TestCase):
    """A directory reported once, by the ecosystem tried first, still owns the crates below it where its
    `Cargo.toml` declares a workspace (D14): no member is proposed."""

    def candidates(self, files: dict[str, str]) -> list[tuple[str, str]]:
        with tempfile.TemporaryDirectory() as directory:
            return [(r.path, r.found.ecosystem) for r in buildable(write(Path(directory), files))]

    def test_each_earlier_row_beside_a_workspace_root_is_one_candidate_and_no_member_is_proposed(self) -> None:
        for ecosystem, manifest in ROOTS.items():
            with self.subTest(ecosystem):
                self.assertEqual(self.candidates({**manifest, **RUST}), [(".", ecosystem)])

    def test_a_member_directory_that_also_holds_a_package_json_is_still_node(self) -> None:
        files = {**ROOTS["python"], **RUST, "crates/a/package.json": '{"name": "a"}'}
        self.assertEqual(self.candidates(files), [(".", "python"), ("crates/a", "node")])

    def test_a_workspace_below_the_root_is_still_its_own_candidate_beside_an_earlier_manifest(self) -> None:
        files = {**ROOTS["node"], **RUST, "fuzz/Cargo.toml": '[package]\nname = "f"\n[workspace]\n'}
        self.assertEqual(self.candidates(files), [(".", "node"), ("fuzz", "cargo")])

    def test_a_plain_crate_beside_an_earlier_manifest_does_not_own_the_crates_below_it(self) -> None:
        files = {**ROOTS["node"], "Cargo.toml": '[package]\nname = "r"\n', "fuzz/Cargo.toml": '[package]\n'}
        self.assertEqual(self.candidates(files), [(".", "node"), ("fuzz", "cargo")])

    def test_the_survey_and_the_lockfile_rule_name_the_same_members(self) -> None:
        trees = {
            "napi": {**ROOTS["node"], **RUST},
            "virtual": RUST,
            "nested": {**RUST, "fuzz/Cargo.toml": '[workspace]\n', "fuzz/t/Cargo.toml": "[package]\n"},
            "plain": {"Cargo.toml": "[package]\n", "tools/Cargo.toml": "[package]\n"},
        }
        for name, files in trees.items():
            with self.subTest(name), tempfile.TemporaryDirectory() as directory:
                root = write(Path(directory), files)
                crates = {Path(f).parent.as_posix() for f in files if f.endswith("Cargo.toml")}
                proposed = {r.path for r in buildable(root)} & crates
                unlocked = {Path(f.where).parent.as_posix() for f in missing_lockfiles(root, sorted(files))
                            if f.where.endswith("Cargo.toml")}
                self.assertEqual(proposed, unlocked, "a crate is a build exactly when it is not a member")
