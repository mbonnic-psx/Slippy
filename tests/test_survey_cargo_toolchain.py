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


if __name__ == "__main__":
    unittest.main()
