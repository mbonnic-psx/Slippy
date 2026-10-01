"""The Cargo row's toolchain: the pin a Rust repository carries, read the way rustup reads it, recorded as written —
or an empty version where nothing usable is pinned. Every test enters at the survey over a tree on disk."""
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

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


if __name__ == "__main__":
    unittest.main()
