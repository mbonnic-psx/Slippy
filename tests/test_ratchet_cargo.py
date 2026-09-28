"""The ratchet over clippy: each finding of the lint the Cargo row proposes is a key of its own (D10).

The ratchet keys a finding by a line naming a file at a position. Clippy's default format puts the position on a
separate ` --> file:line:col` line, so every finding in one file would be one key and a baselined crate would pass
with any number of new warnings; `--message-format=short` prints one line per finding. This holds the two together:
the command the survey proposes, and what the ratchet makes of the output that command prints.
"""
from __future__ import annotations

import importlib.util
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType

from test_survey import write

from slipwai.assets import ROOT
from slipwai.survey import survey

SHORT = """\
src/lib.rs:1:32: error: unneeded `return` statement
src/lib.rs:2:48: error: length comparison to zero: help: using `is_empty` is clearer and more idiomatic
error: could not compile `ledger` (lib) due to 2 previous errors
error: could not compile `ledger` (lib test) due to 2 previous errors
"""
DEFAULT = """\
error: unneeded `return` statement
 --> src/lib.rs:1:32
  |
1 | pub fn a() -> u32 { return 1; }
  |                     ^^^^^^^^^ help: remove `return`

error: length comparison to zero
 --> src/lib.rs:2:48
  |
2 | pub fn b(v: &[u8]) -> bool { v.len() == 0 }
  |                              ^^^^^^^^^^^^ help: using `is_empty` is clearer

error: could not compile `ledger` (lib) due to 2 previous errors
"""


def ratchet() -> ModuleType:
    spec = importlib.util.spec_from_file_location("ratchet_script", ROOT / "assets/adoption/scripts/ratchet.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ClippyRatchetTest(unittest.TestCase):
    def test_the_proposed_lint_prints_one_line_per_finding_and_each_is_its_own_ratchet_key(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "Cargo.toml": '[package]\nname = "ledger"\n', "src/lib.rs": "pub fn a() -> u32 { return 1; }\n",
            })
            lint = survey(root).roots[0].found.commands["lint"]
            assert lint is not None
            self.assertIn("cargo clippy --all-targets --message-format=short -- -D warnings", lint)
            keys = ratchet().findings_in(SHORT, root.resolve())
            self.assertEqual(len(keys), 2, keys)
            self.assertTrue(all(key.startswith("src/lib.rs: error:") for key in keys), keys)
            # Why the flag is there: the default format's position line names the file once per finding, with
            # nothing else on it, so both findings in the file are the one key.
            self.assertEqual(ratchet().findings_in(DEFAULT, root.resolve()), ["--> src/lib.rs"])


# What cargo 1.x prints, and exits 101 on, for a subcommand that is not installed (`cargo no-such-sub`).
NO_SUCH = (
    "error: no such command: `{sub}`\n\nhelp: view all installed commands with `cargo --list`\n"
    "help: find a package to install `{sub}` with `cargo search cargo-{sub}`\n"
)


def gate(directory: Path, cargo: str, command: str, env: dict[str, str] | None = None):
    """The shipped ratchet, copied where an adopted repository keeps it, run over `command` with a fake `cargo`
    (a script in the test tree) first on PATH."""
    scripts = directory / "delivery/scripts"
    scripts.mkdir(parents=True)
    shutil.copy(ROOT / "assets/adoption/scripts/ratchet.py", scripts / "ratchet.py")
    (directory / "project.json").write_text("{}\n")
    bin_ = directory / "bin"
    bin_.mkdir()
    fake = bin_ / "cargo"
    fake.write_text("#!/bin/sh\n" + cargo)
    fake.chmod(fake.stat().st_mode | stat.S_IEXEC)
    clean = {k: v for k, v in os.environ.items() if k not in ("CI", "RATCHET_TIGHTEN")}
    clean["PATH"] = f"{bin_}:{clean['PATH']}"
    return subprocess.run(
        [sys.executable, str(scripts / "ratchet.py"), "shop", "lint", "--", command],
        cwd=directory, text=True, capture_output=True, env={**clean, **(env or {})},
    )


class MissingSubcommandTest(unittest.TestCase):
    def test_a_cargo_subcommand_that_is_not_installed_is_not_runnable_and_never_baselined(self) -> None:
        for sub, command in (
            ("clippy", "cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt --check"),
            ("fmt", "cargo fmt --check"),
            ("audit", "cargo audit"),
        ):
            with self.subTest(sub), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                body = f"echo 'error: no such command: `{sub}`' >&2\nprintf '\\nhelp: x\\n' >&2\nexit 101\n"
                run = gate(root, body, command)
                self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
                self.assertIn(f"could not run — `cargo {sub}` is not on this machine (exit 101)", run.stderr)
                self.assertIn("nothing is recorded", run.stderr)
                self.assertFalse((root / "delivery/baseline.json").exists(), "a missing component is no baseline")

    def test_a_real_clippy_failure_with_exit_101_is_still_a_baseline(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            body = "cat <<'E' >&2\n" + SHORT + "E\nexit 101\n"
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("pub fn a() -> u32 { return 1; }\n")
            run = gate(root, body, "cargo clippy --all-targets --message-format=short -- -D warnings")
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
            self.assertIn("baseline recorded for shop lint — 2 finding(s)", run.stdout)


if __name__ == "__main__":
    unittest.main()
