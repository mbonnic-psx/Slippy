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


def gate(directory: Path, cargo: str, command: str, env: dict[str, str] | None = None, target: str = "lint"):
    """The shipped ratchet, copied where an adopted repository keeps it, run over `command` with a fake `cargo`
    (a script in the test tree) first on PATH."""
    scripts = directory / "delivery/scripts"
    scripts.mkdir(parents=True, exist_ok=True)
    shutil.copy(ROOT / "assets/adoption/scripts/ratchet.py", scripts / "ratchet.py")
    (directory / "project.json").write_text("{}\n")
    bin_ = directory / "bin"
    bin_.mkdir(exist_ok=True)
    fake = bin_ / "cargo"
    fake.write_text("#!/bin/sh\n" + cargo)
    fake.chmod(fake.stat().st_mode | stat.S_IEXEC)
    clean = {k: v for k, v in os.environ.items() if k not in ("CI", "RATCHET_TIGHTEN")}
    clean["PATH"] = f"{bin_}:{clean['PATH']}"
    return subprocess.run(
        [sys.executable, str(scripts / "ratchet.py"), "shop", target, "--", command],
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


    def test_the_tool_is_read_past_a_cd_an_assignment_and_env(self) -> None:
        """The survey proposes `cd ledger && cargo clippy …` for a crate one level down; the tool is `cargo` there
        as much as at the root (adversary R1)."""
        body = "echo 'error: no such command: `clippy`' >&2\nexit 101\n"
        for command in ("cd sub && cargo clippy", "RUSTFLAGS=-Dwarnings cargo clippy",
                        "env RUSTFLAGS=x cargo clippy", "cd sub && env -u CI RUSTFLAGS=x cargo clippy"):
            with self.subTest(command), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / "sub").mkdir()
                run = gate(root, body, command)
                self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
                self.assertIn("could not run — `cargo clippy` is not on this machine", run.stderr)
                self.assertFalse((root / "delivery/baseline.json").exists())

    def test_rustups_component_not_installed_is_not_runnable_either(self) -> None:
        """rustup's proxy, for a toolchain without the component, prints this and exits 1 — observed with rustup
        1.29.0 on 2026-09-29 (adversary R2)."""
        for sub, command in (("clippy", "cargo clippy"), ("fmt", "cargo fmt --check")):
            with self.subTest(sub), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                body = (f"echo \"error: 'cargo-{sub}' is not installed for the toolchain 'stable-x86_64-unknown-linux-"
                        f"gnu'.\" >&2\necho 'help: run `rustup component add {'rustfmt' if sub == 'fmt' else sub}`' "
                        ">&2\nexit 1\n")
                run = gate(root, body, command)
                self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
                self.assertIn(f"could not run — `cargo {sub}` is not on this machine (exit 1)", run.stderr)
                self.assertFalse((root / "delivery/baseline.json").exists())

    def test_colour_does_not_hide_that_a_subcommand_is_missing(self) -> None:
        """`CARGO_TERM_COLOR=always`, which Rust CI sets, wraps the word in escapes (adversary R5)."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            body = ("printf '\\033[1m\\033[91merror\\033[0m\\033[1m:\\033[0m no such command: `clippy`\\n' >&2\n"
                    "exit 101\n")
            run = gate(root, body, "cargo clippy")
            self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
            self.assertIn("could not run — `cargo clippy`", run.stderr)


# What cargo 1.98 printed, and exited 101 on, for `cargo clippy` over a Tauri crate on a machine without libdbus's
# headers (an adoption of a Rust desktop app, 2026-09-30), trimmed of the build script's pkg-config detail.
BUILD_SCRIPT = """\
    Checking serde_spanned v1.1.1
error: failed to run custom build command for `libdbus-sys v0.2.7`

Caused by:
  process didn't exit successfully: `/w/target/debug/build/libdbus-sys-a5ad98f7/build-script-build` (exit status: 101)
  --- stderr
  The system library `dbus-1` required by crate `libdbus-sys` was not found.

  thread 'main' (1476261) panicked at /home/u/.cargo/registry/src/index.crates.io/libdbus-sys-0.2.7/build.rs:25:9:
  explicit panic
warning: build failed, waiting for other jobs to finish...
"""


class BuildScriptTest(unittest.TestCase):
    """A build that stops in a dependency's build script never reached the code, so it is no baseline: recorded, its
    exit code would pass every later run that got no further."""

    def test_a_build_script_that_did_not_finish_is_not_runnable_and_never_baselined(self) -> None:
        for target, command in (
            ("lint", "cd sub && cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt --check"),
            ("typecheck", "cd sub && cargo check --all-targets"),
            ("test", "cd sub && cargo test"),
        ):
            with self.subTest(target), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / "sub").mkdir()
                run = gate(root, f"cat <<'E' >&2\n{BUILD_SCRIPT}E\nexit 101\n", command, target=target)
                self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
                self.assertIn("could not run — the build script of `libdbus-sys v0.2.7` did not finish (exit 101)",
                              run.stderr)
                self.assertIn(f"{target} never reached this repository's code", run.stderr)
                self.assertFalse((root / "delivery/baseline.json").exists(), "a build that stopped is no baseline")

    def test_ratchet_tighten_does_not_record_it_either(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            run = gate(root, f"cat <<'E' >&2\n{BUILD_SCRIPT}E\nexit 101\n", "cargo test", {"RATCHET_TIGHTEN": "1"},
                       target="test")
            self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
            self.assertFalse((root / "delivery/baseline.json").exists())

    def test_a_baseline_already_recorded_from_one_does_not_pass_it_on_the_exit_code(self) -> None:
        """The baseline an earlier ratchet wrote for exactly this output: exit 101, no findings."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "delivery").mkdir()
            (root / "delivery/baseline.json").write_text('{"shop": {"lint": {"exit": 101, "findings": []}}}\n')
            run = gate(root, f"cat <<'E' >&2\n{BUILD_SCRIPT}E\nexit 101\n", "cargo clippy")
            self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
            self.assertIn("did not finish", run.stderr)

    def test_colour_does_not_hide_it(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            body = ("printf '\\033[1m\\033[91merror\\033[0m\\033[1m:\\033[0m failed to run custom build command for "
                    "`openssl-sys v0.9.103`\\n' >&2\nexit 101\n")
            run = gate(root, body, "cargo check")
            self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
            self.assertIn("the build script of `openssl-sys v0.9.103` did not finish", run.stderr)


def cargo_test(failing: tuple[str, ...], thread: int, crash: bool = False) -> str:
    """What `cargo test` prints (1.98, libtest) for these failing tests; a crash cuts the run short."""
    lines = ["running 3 tests"] + [f"test {name} ... FAILED" for name in failing] + ["test tests::ok ... ok", ""]
    if crash:
        return "\n".join(lines + [
            "error: test failed, to rerun pass `--lib`", "", "Caused by:",
            "  process didn't exit successfully: `/w/target/debug/deps/ledger-1f2e3d` (signal: 6, SIGABRT: process "
            "abort signal)", ""])
    lines += ["failures:", ""]
    for name in failing:
        lines += [f"---- {name} stdout ----", "", f"thread '{name}' ({thread}) panicked at src/lib.rs:7:22:",
                  "assertion `left == right` failed", "  left: 1", " right: 2", ""]
    lines += ["failures:"] + [f"    {name}" for name in failing] + [
        "", f"test result: FAILED. 1 passed; {len(failing)} failed; 0 ignored", "",
        "error: test failed, to rerun pass `--lib`", ""]
    return "\n".join(lines)


class QuarantinedCargoTestTest(unittest.TestCase):
    """A red `cargo test` quarantined by `make ratchet-tighten` passes on that state and fails on a new failure."""

    def run_twice(self, first: str, second: str) -> subprocess.CompletedProcess:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("pub fn a() {}\n")
            recorded = gate(root, f"cat <<'E'\n{first}E\nexit 101\n", "cargo test", {"RATCHET_TIGHTEN": "1"},
                            target="test")
            self.assertEqual(recorded.returncode, 0, recorded.stdout + recorded.stderr)
            self.assertIn("QUARANTINED", recorded.stdout)
            return gate(root, f"cat <<'E'\n{second}E\nexit 101\n", "cargo test", target="test")

    def test_the_same_failure_on_another_thread_is_the_same_finding(self) -> None:
        """libtest names the OS thread in the panic line, and it differs every run (adversary R3)."""
        again = self.run_twice(cargo_test(("tests::known_red",), 520735), cargo_test(("tests::known_red",), 520753))
        self.assertEqual(again.returncode, 0, again.stdout + again.stderr)
        self.assertIn("none new", again.stdout)

    def test_a_new_failure_that_does_not_panic_is_new(self) -> None:
        """A test returning `Err` prints no panic line; its name on the `... FAILED` line is its key (R7)."""
        again = self.run_twice(cargo_test(("tests::known_red",), 1),
                               cargo_test(("tests::known_red", "more::returns_err"), 2))
        self.assertEqual(again.returncode, 1, again.stdout + again.stderr)
        self.assertIn("test: more::returns_err", again.stderr)

    def test_a_crash_is_new_against_a_quarantined_suite(self) -> None:
        """A test binary killed by a signal prints no location and exits 101, as the quarantined state did (R4)."""
        again = self.run_twice(cargo_test(("tests::known_red",), 1), cargo_test((), 2, crash=True))
        self.assertEqual(again.returncode, 1, again.stdout + again.stderr)
        self.assertIn("SIGABRT", again.stderr)


if __name__ == "__main__":
    unittest.main()
