"""What every ecosystem row is made of: the eight targets, what a row detects, and the helpers each row writes
its commands with. The rows themselves are `rows.py` and `cargo.py`; `__init__.py` is the table."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from ..bounded_read import read

# The eight targets, in the order a service's recipes spell them (`project/native_commands.py`).
TARGETS = ("install", "typecheck", "lint", "test", "integration", "adversarial", "audit", "mutation")
# What a wrapped application may record beside the eight, each with a target of its own and never proposed by the
# survey: `test-full`, a suite too slow for `verify`; `smoke`, the one command that starts the application and proves
# it answers — the fact the day-one gate cannot compose from the eight, and the floor beside the build
# (`programme.py`), because every false green the first adoptions shipped was a change nothing had started.
EXTRA = ("test-full", "smoke")
Commands = dict[str, str | None]


@dataclass(frozen=True)
class Detected:
    """One buildable directory, as its files describe it."""

    ecosystem: str
    language: str
    evidence: str
    commands: Commands
    # `kind` names the runtime a CI job sets up (`node`, `python`, `go`, `java`, `dotnet`, `php`, `ruby`, `rust`);
    # `version` is the pin the tree carries, or empty where it carries none.
    toolchain: dict[str, str]
    # What the build packages, where that decides which rung of the ladder is next: `war` for a Maven build that
    # makes one.
    packaging: str | None = None


def complete(**given: str | None) -> Commands:
    """Every target, in order; the ones not given are written no's."""
    return {target: given.get(target) for target in TARGETS}


def first_line(path: Path) -> str:
    text = read(path).strip()
    return text.splitlines()[0].strip().lstrip("v") if text else ""


def in_dir(directory: str, command: str) -> str:
    """A command run inside `directory`, from the repository root — as is when the directory is the root."""
    return command if directory == "." else f"cd {directory} && {command}"


def prefixed(directory: str, path: str) -> str:
    return path if directory == "." else f"{directory}/{path}"
