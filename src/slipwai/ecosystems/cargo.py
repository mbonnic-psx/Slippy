"""The Cargo row: a directory holding a `Cargo.toml` is Rust built by Cargo, found by the file name alone, so a
manifest the survey cannot parse is still recognised. The row reads one fact from the manifest — whether it declares
a workspace — because that decides whether check, clippy and test are told to cover every member."""
from __future__ import annotations

import re
from collections.abc import Callable
from pathlib import Path

from .common import Detected, complete, in_dir, prefixed, read

# A `[workspace]` or `[workspace.<x>]` table header at the start of a line: a workspace root. Not `workspace = true`
# in a dependency, `package.workspace = "…"` or a comment, which are a member pointing at a root, and not a
# `[[workspace…]]` array table. A U+FEFF counts as leading space on any line, not only the first: Cargo reads a
# manifest that begins with a byte order mark, and a match that is slightly wider than Cargo's is the safe way round.
WORKSPACE = re.compile(r"(?m)^[ \t\ufeff]*\[[ \t]*workspace[ \t]*[.\]]")


def declares_workspace(manifest: Path) -> bool:
    """Whether the manifest holds a workspace table header; a manifest that cannot be read declares none."""
    return WORKSPACE.search(read(manifest)) is not None


def member_of_workspace(root: Path, manifest: Path, present: Callable[[Path], bool] | None = None) -> bool:
    """Whether a `Cargo.toml` (a path from `root`) declares no workspace itself and one above it does, whatever
    ecosystem the directory of that root is reported as: Cargo writes one `Cargo.lock` at the workspace root and
    builds every member from it. `present` says which manifests count as there — the files on disk, or the ones
    Git tracks — and the survey's ownership and the lockfile rule both ask here, so they cannot disagree."""
    if declares_workspace(root / manifest):
        return False
    there = present or (lambda candidate: (root / candidate).is_file())
    return any(
        there(above) and declares_workspace(root / above)
        for above in (parent / "Cargo.toml" for parent in manifest.parent.parents)
    )


def cargo(root: Path, directory: str) -> Detected | None:
    if not (root / directory / "Cargo.toml").is_file():
        return None
    flag = " --workspace" if declares_workspace(root / directory / "Cargo.toml") else ""
    return Detected(
        "cargo", "rust", prefixed(directory, "Cargo.toml"),
        complete(
            install=in_dir(directory, "cargo fetch --locked"),
            typecheck=in_dir(directory, f"cargo check{flag} --all-targets"),
            lint=in_dir(
                directory,
                f"cargo clippy{flag} --all-targets --message-format=short -- -D warnings && cargo fmt --check",
            ),
            test=in_dir(directory, f"cargo test{flag}"),
        ),
        {"kind": "rust", "version": ""},
    )
