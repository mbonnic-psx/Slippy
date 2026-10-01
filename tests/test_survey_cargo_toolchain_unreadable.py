"""The Cargo row's toolchain where a file cannot be read as text: rustup passes over a toolchain file it cannot read
and looks on (D24, research R15-R19), so the survey does, and a regular file it can read decides even when it names
nothing. Every test enters at the survey over a tree on disk, with a pin at the root above where a walk is in
question."""
from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path

from test_survey import write
from test_survey_cargo_toolchain import CRATE, toml

from slipwai.bounded_read import MAX_READ
from slipwai.survey import survey

PINNED = {"kind": "rust", "version": "1.85"}
NONE = {"kind": "rust", "version": ""}


class UnreadableToolchainFileTest(unittest.TestCase):
    def crate(self, make, above: bool = True) -> dict:
        """The toolchain recorded for `crates/a`, with `make(directory)` laying the files there and a root pin of
        1.85 above where `above` says."""
        with tempfile.TemporaryDirectory() as directory:
            files = {"crates/a/Cargo.toml": CRATE}
            if above:
                files["rust-toolchain.toml"] = toml("1.85")
            root = write(Path(directory), files)
            make(root / "crates/a")
            roots = [r for r in survey(root).roots if r.path == "crates/a"]
            self.assertEqual(len(roots), 1)
            return roots[0].found.toolchain

    def test_a_directory_named_either_file_is_passed_over_upward(self) -> None:
        for name in ("rust-toolchain", "rust-toolchain.toml"):
            with self.subTest(name):
                self.assertEqual(self.crate(lambda here, name=name: (here / name).mkdir()), PINNED)

    def test_a_directory_with_no_pin_above_is_still_no_pin(self) -> None:
        self.assertEqual(self.crate(lambda here: (here / "rust-toolchain.toml").mkdir(), above=False), NONE)

    def test_a_dangling_link_named_the_toml_is_passed_over_upward(self) -> None:
        self.assertEqual(self.crate(lambda here: (here / "rust-toolchain.toml").symlink_to("nowhere")), PINNED)

    def test_an_unreadable_name_is_passed_over_to_the_other_name_in_its_directory_first(self) -> None:
        def make(here: Path) -> None:
            (here / "rust-toolchain").mkdir()
            (here / "rust-toolchain.toml").write_text(toml("stable"))

        self.assertEqual(self.crate(make), {"kind": "rust", "version": "stable"})

    def test_a_file_that_is_not_utf8_is_passed_over(self) -> None:
        for name in ("rust-toolchain", "rust-toolchain.toml"):
            with self.subTest(name):
                self.assertEqual(self.crate(lambda here, name=name: (here / name).write_bytes(b"1.84\xff\n")), PINNED)

    def test_a_file_that_is_not_utf8_falls_to_the_toml_beside_it(self) -> None:
        def make(here: Path) -> None:
            (here / "rust-toolchain").write_bytes(b"1.84\xff\n")
            (here / "rust-toolchain.toml").write_text(toml("stable"))

        self.assertEqual(self.crate(make), {"kind": "rust", "version": "stable"})

    def test_a_file_with_mode_000_is_passed_over(self) -> None:
        def make(here: Path) -> None:
            path = here / "rust-toolchain"
            path.write_text("1.84\n")
            path.chmod(0)
            if os.access(path, os.R_OK):
                path.chmod(0o600)
                self.skipTest("this user can read a mode 000 file (root), so there is nothing unreadable to pass over")

        self.assertEqual(self.crate(make), PINNED)

    @unittest.skipUnless(hasattr(os, "mkfifo"), "no FIFOs on this platform")
    def test_a_fifo_is_never_opened_and_is_passed_over(self) -> None:
        # A FIFO opened for reading blocks until a writer comes (issue #13); rustup itself blocks on it (R20).
        self.assertEqual(self.crate(lambda here: os.mkfifo(here / "rust-toolchain")), PINNED)

    def test_a_file_that_is_read_and_names_nothing_usable_decides_with_an_empty_version(self) -> None:
        texts = {
            "empty": "", "invalid TOML": "[toolchain\nchannel = ", "no channel": '[toolchain]\ncomponents = []\n',
            "a path": '[toolchain]\npath = "/opt/rust"\n', "a channel that is not a name": toml("1.85 x"),
            "over the bounded read's limit": toml("1.84") + "#" * MAX_READ,
        }
        for name in ("rust-toolchain", "rust-toolchain.toml"):
            for what, text in texts.items():
                with self.subTest(f"{what} in {name}"):
                    found = self.crate(lambda here, name=name, text=text: (here / name).write_text(text))
                    self.assertEqual(found, NONE)


class LinkOutsideTheRepositoryTest(unittest.TestCase):
    """D26: a toolchain file whose link resolves outside the repository root is passed over as an unreadable one is."""

    def crate(self, make, above: bool = True) -> dict:
        """The toolchain for `crates/a` in a repository at `<scratch>/repo`; `make(here, outside)` lays files, where
        `outside` is a directory beside the repository holding `elsewhere` ("9.9")."""
        with tempfile.TemporaryDirectory() as scratch:
            outside = Path(scratch) / "outside"
            outside.mkdir()
            (outside / "elsewhere").write_text("9.9\n")
            files = {"crates/a/Cargo.toml": CRATE}
            if above:
                files["rust-toolchain.toml"] = toml("1.85")
            repo = Path(scratch) / "repo"
            repo.mkdir()
            root = write(repo, files)
            make(root / "crates/a", outside)
            roots = [r for r in survey(root).roots if r.path == "crates/a"]
            self.assertEqual(len(roots), 1)
            return roots[0].found.toolchain

    def test_a_link_outside_the_root_is_passed_over_to_the_pin_above(self) -> None:
        self.assertEqual(
            self.crate(lambda here, outside: (here / "rust-toolchain").symlink_to(outside / "elsewhere")), PINNED)

    def test_a_link_outside_the_root_with_no_other_pin_is_no_pin(self) -> None:
        self.assertEqual(
            self.crate(lambda here, outside: (here / "rust-toolchain").symlink_to(outside / "elsewhere"), above=False),
            NONE)

    def test_a_link_outside_the_root_falls_to_the_toml_beside_it(self) -> None:
        def make(here: Path, outside: Path) -> None:
            (here / "rust-toolchain").symlink_to(outside / "elsewhere")
            (here / "rust-toolchain.toml").write_text(toml("stable"))

        self.assertEqual(self.crate(make), {"kind": "rust", "version": "stable"})

    def test_a_link_inside_the_root_is_still_read(self) -> None:
        def make(here: Path, outside: Path) -> None:
            (here.parent.parent / "pins").mkdir()
            (here.parent.parent / "pins" / "chosen").write_text("1.70\n")
            (here / "rust-toolchain").symlink_to("../../pins/chosen")

        self.assertEqual(self.crate(make), {"kind": "rust", "version": "1.70"})

    def test_a_link_through_a_linked_directory_that_escapes_the_root_is_passed_over(self) -> None:
        def make(here: Path, outside: Path) -> None:
            (outside / "rust-toolchain").write_text("9.9\n")
            (here / "rust-toolchain").symlink_to("via/rust-toolchain")
            (here / "via").symlink_to(outside, target_is_directory=True)

        self.assertEqual(self.crate(make), PINNED)


class ByteOrderMarkTest(unittest.TestCase):
    def pinned(self, name: str, text: str) -> dict:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"Cargo.toml": CRATE, name: text})
            return survey(root).roots[0].found.toolchain

    def test_a_leading_byte_order_mark_is_removed_before_toml_is_parsed(self) -> None:
        for name in ("rust-toolchain.toml", "rust-toolchain"):
            with self.subTest(name):
                self.assertEqual(self.pinned(name, "﻿" + toml("1.85")), PINNED)

    def test_a_one_line_legacy_file_keeps_its_mark_and_names_no_toolchain(self) -> None:
        self.assertEqual(self.pinned("rust-toolchain", "﻿1.85\n"), NONE)  # R19b

    def test_only_a_leading_mark_is_removed(self) -> None:
        self.assertEqual(self.pinned("rust-toolchain.toml", toml("1.85") + "﻿"), NONE)


if __name__ == "__main__":
    unittest.main()
