"""What a build ecosystem's files say about the code they build, and how its own tools answer the Make targets.

The table behind `survey.py` (brownfield adoption; experimental as `AGENTS.md` defines the word). One
row per ecosystem the survey can recognise — by the manifest file that starts its build — with the language
it is written in, how to tell its toolchain's version, and the command its own tools run for each of the
eight Make targets a service owes. A target the ecosystem has no answer for is `None`, which the manifest
records as a written `null` and the Makefile runs as a line that says so; it is never guessed at.

Deliberately not `catalog.json`: the catalog is the contract for what the factory can *generate*, and a C#
row there would be a claim nothing behind it keeps. These rows say only what can be *recognised*, and every
command here is a proposal the person confirms or overrides — `survey.py` records which.
"""
from __future__ import annotations

from collections.abc import Callable
from pathlib import Path

from .cargo import cargo, declares_workspace, member_of_workspace
from .common import EXTRA, TARGETS, Commands, Detected, prefixed, read
from .rows import aggregates, ant, dotnet, go, gradle, maven, node, php, python, ruby

# In the order tried, so a directory with a `package.json` beside a `pyproject.toml` is reported once, as Node,
# and a `pom.xml` beside a leftover `build.xml` as Maven; the survey says which file decided it.
ECOSYSTEMS: tuple[Callable[[Path, str], Detected | None], ...] = (
    node, python, go, maven, gradle, ant, dotnet, php, ruby, cargo,
)

__all__ = [
    "ECOSYSTEMS", "EXTRA", "TARGETS", "Commands", "Detected", "aggregates", "cargo", "declares_workspace",
    "member_of_workspace", "prefixed", "read",
]
