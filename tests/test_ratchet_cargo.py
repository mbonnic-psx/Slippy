"""The ratchet over clippy: each finding of the lint the Cargo row proposes is a key of its own (D10).

The ratchet keys a finding by a line naming a file at a position. Clippy's default format puts the position on a
separate ` --> file:line:col` line, so every finding in one file would be one key and a baselined crate would pass
with any number of new warnings; `--message-format=short` prints one line per finding. This holds the two together:
the command the survey proposes, and what the ratchet makes of the output that command prints.
"""
from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

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


def ratchet():
    spec = importlib.util.spec_from_file_location("ratchet_script", ROOT / "assets/adoption/scripts/ratchet.py")
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
            self.assertIn("cargo clippy --all-targets --message-format=short -- -D warnings", lint)
            keys = ratchet().findings_in(SHORT, root.resolve())
            self.assertEqual(len(keys), 2, keys)
            self.assertTrue(all(key.startswith("src/lib.rs: error:") for key in keys), keys)
            # Why the flag is there: the default format's position line names the file once per finding, with
            # nothing else on it, so both findings in the file are the one key.
            self.assertEqual(ratchet().findings_in(DEFAULT, root.resolve()), ["--> src/lib.rs"])


if __name__ == "__main__":
    unittest.main()
