"""The Cargo row's toolchain: the pin a Rust repository carries, read the way rustup reads it, recorded as written —
or an empty version where nothing usable is pinned. Every test enters at the survey over a tree on disk."""
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from test_survey import write

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


if __name__ == "__main__":
    unittest.main()
