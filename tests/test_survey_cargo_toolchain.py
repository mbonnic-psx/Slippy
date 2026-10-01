"""The Cargo row's toolchain: the pin a Rust repository carries, read the way rustup reads it, recorded as written —
or an empty version where nothing usable is pinned. Every test enters at the survey over a tree on disk."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from test_adopt import repository, slipwai
from test_survey import write

from slipwai.bounded_read import MAX_READ
from slipwai.survey import survey

CRATE = '[package]\nname = "ledger"\nversion = "0.1.0"\n'


def toml(channel: str) -> str:
    return f'[toolchain]\nchannel = "{channel}"\n'


class ToolchainPinTest(unittest.TestCase):
    def candidate(self, files: dict[str, str], path: str = ".") -> dict:
        """The toolchain the survey records for the candidate at `path` in a tree of `files`."""
        with tempfile.TemporaryDirectory() as directory:
            found = survey(write(Path(directory), files))
            matching = [r for r in found.roots if r.path == path]
            self.assertEqual(len(matching), 1, f"one candidate at {path}, got {[r.path for r in found.roots]}")
            return matching[0].found.toolchain

    def test_a_rust_toolchain_toml_in_the_crates_directory_is_the_recorded_version(self) -> None:
        self.assertEqual(
            self.candidate({"Cargo.toml": CRATE, "rust-toolchain.toml": toml("1.85")}),
            {"kind": "rust", "version": "1.85"},
        )

    def test_every_channel_shape_is_recorded_exactly_as_written(self) -> None:
        for channel in ("1.85", "1.85.0", "stable", "nightly", "nightly-2025-01-01", "1.85-beta", "Stable"):
            with self.subTest(channel):
                self.assertEqual(
                    self.candidate({"Cargo.toml": CRATE, "rust-toolchain.toml": toml(channel)}),
                    {"kind": "rust", "version": channel},
                    "the record holds kind and version and nothing else",
                )

    def test_the_candidates_evidence_stays_the_cargo_toml(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"Cargo.toml": CRATE, "rust-toolchain.toml": toml("1.85")})
            self.assertEqual(survey(root).roots[0].found.evidence, "Cargo.toml")

    def test_nothing_usable_pinned_is_an_empty_version_and_never_an_error(self) -> None:
        none = {"kind": "rust", "version": ""}
        pins = {
            "invalid TOML": "[toolchain\nchannel = ",
            "toolchain is not a table": 'toolchain = "1.85"\n',
            "table with no channel": '[toolchain]\ncomponents = ["clippy"]\n',
            "channel is not a string": "[toolchain]\nchannel = 3\n",
            "a path, a place on somebody's machine": '[toolchain]\npath = "/opt/rust"\n',
            "a path beside a channel": '[toolchain]\nchannel = "1.85"\npath = "/opt/rust"\n',
            "an empty file": "",
            "a byte order mark": "\ufeff" + toml("1.85"),
            "over the bounded read's limit": toml("1.85") + "#" * MAX_READ,
        }
        for name, text in pins.items():
            with self.subTest(name):
                self.assertEqual(self.candidate({"Cargo.toml": CRATE, "rust-toolchain.toml": text}), none)

    def test_a_directory_named_rust_toolchain_toml_is_no_pin(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"Cargo.toml": CRATE})
            (root / "rust-toolchain.toml").mkdir()
            self.assertEqual(survey(root).roots[0].found.toolchain, {"kind": "rust", "version": ""})

    def test_rust_version_in_the_manifest_is_not_a_pin(self) -> None:
        manifest = CRATE + 'rust-version = "1.70"\n'
        self.assertEqual(self.candidate({"Cargo.toml": manifest}), {"kind": "rust", "version": ""})

    def test_no_toolchain_file_anywhere_is_an_empty_version(self) -> None:
        self.assertEqual(self.candidate({"Cargo.toml": CRATE, "src/lib.rs": ""}), {"kind": "rust", "version": ""})

    def legacy(self, text: str, beside: str | None = None) -> str:
        """The version recorded for a crate whose `rust-toolchain` holds `text`, and `rust-toolchain.toml` `beside`."""
        files = {"Cargo.toml": CRATE, "rust-toolchain": text}
        if beside is not None:
            files["rust-toolchain.toml"] = beside
        return self.candidate(files)["version"]

    def test_a_one_line_rust_toolchain_is_that_line_stripped(self) -> None:
        for name, text in (("plain", "1.85\n"), ("padded", "  1.85  \n"), ("no newline", "1.85"), ("crlf", "1.85\r\n")):
            with self.subTest(name):
                self.assertEqual(self.legacy(text), "1.85")

    def test_a_leading_v_is_not_stripped(self) -> None:
        self.assertEqual(self.legacy("v1.85\n"), "v1.85")

    def test_a_legacy_file_of_several_lines_is_toml(self) -> None:
        self.assertEqual(self.legacy('# pinned for the MSRV\n[toolchain]\nchannel = "1.85"\n'), "1.85")
        self.assertEqual(self.legacy(toml("1.85")), "1.85")

    def test_a_legacy_file_of_several_lines_that_is_not_toml_is_no_pin(self) -> None:
        for name, text in (("blank line after", "1.85\n\n"), ("blank line before", "\n  1.85  \n")):
            with self.subTest(name):
                self.assertEqual(self.legacy(text), "")

    def test_a_legacy_file_holding_toml_with_a_non_string_channel_is_no_pin(self) -> None:
        self.assertEqual(self.legacy("[toolchain]\nchannel = 3\n"), "")

    def test_the_legacy_file_wins_over_the_toml_beside_it(self) -> None:
        self.assertEqual(self.legacy("1.84\n", beside=toml("1.85")), "1.84")

    def test_an_empty_legacy_file_is_no_pin_and_the_toml_beside_it_is_not_consulted(self) -> None:
        self.assertEqual(self.legacy("", beside=toml("1.85")), "")

    def test_a_pin_above_the_crate_is_found_by_walking_up_to_the_root(self) -> None:
        files = {"Cargo.toml": CRATE, "crates/a/Cargo.toml": CRATE, "rust-toolchain.toml": toml("1.85")}
        with tempfile.TemporaryDirectory() as directory:
            roots = {r.path: r.found.toolchain for r in survey(write(Path(directory), files)).roots}
        self.assertEqual(roots["crates/a"], {"kind": "rust", "version": "1.85"})
        self.assertEqual(roots["."], {"kind": "rust", "version": "1.85"}, "the zero-step walk still reads the root")

    def test_a_nearer_file_that_names_no_channel_decides_and_the_search_stops(self) -> None:
        files = {
            "crates/a/Cargo.toml": CRATE, "rust-toolchain.toml": toml("1.85"),
            "crates/a/rust-toolchain.toml": '[toolchain]\ncomponents = ["clippy"]\n',
        }
        self.assertEqual(self.candidate(files, "crates/a"), {"kind": "rust", "version": ""})

    def test_a_dangling_link_in_the_candidates_directory_decides_as_an_empty_pin(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"crates/a/Cargo.toml": CRATE, "rust-toolchain.toml": toml("1.85")})
            (root / "crates/a/rust-toolchain").symlink_to("nowhere")
            self.assertEqual(survey(root).roots[0].found.toolchain, {"kind": "rust", "version": ""})

    def test_a_pin_above_the_repository_root_is_never_read(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            outside = write(Path(directory), {"rust-toolchain.toml": toml("1.85"), "repo/Cargo.toml": CRATE})
            self.assertEqual(survey(outside / "repo").roots[0].found.toolchain, {"kind": "rust", "version": ""})

    def test_another_ecosystems_candidate_beside_a_pin_does_not_read_it(self) -> None:
        files = {"package.json": json.dumps({"name": "web"}), "rust-toolchain.toml": toml("1.85")}
        with tempfile.TemporaryDirectory() as directory:
            found = survey(write(Path(directory), files)).roots[0].found
        self.assertEqual(found.ecosystem, "node")
        self.assertNotEqual(found.toolchain.get("version"), "1.85")
        self.assertEqual(found.toolchain["kind"], "node")

    def adopted(self, parent: Path, files: dict[str, str]) -> tuple[dict, str]:
        """The deployable `adopt --yes` records for a repository of `files`, and its survey page."""
        repo = repository(parent, "adopted", files)
        result = slipwai(repo, "adopt", "--yes")
        self.assertEqual(result.returncode, 0, result.stderr)
        deployable = json.loads((repo / "project.json").read_text())["deployables"][repo.name]
        return deployable, (repo / "delivery/survey/survey.md").read_text()

    def test_adopting_a_pinned_crate_records_the_pin_and_the_survey_page_shows_it(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            deployable, page = self.adopted(Path(directory), {"Cargo.toml": CRATE, "rust-toolchain.toml": toml("1.85")})
        self.assertEqual(deployable["toolchain"], {"kind": "rust", "version": "1.85", "ecosystem": "cargo"})
        self.assertIn("- `.` — cargo, rust, from `Cargo.toml`, rust 1.85;", page)

    def test_adopting_an_unpinned_crate_shows_nothing_after_the_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            deployable, page = self.adopted(Path(directory), {"Cargo.toml": CRATE})
        self.assertEqual(deployable["toolchain"], {"kind": "rust", "version": "", "ecosystem": "cargo"})
        self.assertIn("- `.` — cargo, rust, from `Cargo.toml`;", page)
        self.assertNotIn("rust 1", page)


if __name__ == "__main__":
    unittest.main()
