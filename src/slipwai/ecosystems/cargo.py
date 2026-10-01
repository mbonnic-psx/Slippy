"""The Cargo row: a directory holding a `Cargo.toml` is Rust built by Cargo, found by the file name alone, so a
manifest the survey cannot parse is still recognised. The row reads one fact from the manifest — whether it declares
a workspace — because that decides whether check, clippy and test are told to cover every member, and it reads the
toolchain the repository pins from `rust-toolchain.toml`, the way rustup does."""
from __future__ import annotations

import os
import re
import tomllib
from collections.abc import Callable
from pathlib import Path

from .common import Detected, complete, in_dir, prefixed, read

Reader = Callable[[Path], str]

# A `[workspace]` or `[workspace.<x>]` table header at the start of a line: a workspace root. Not `workspace = true`
# in a dependency, `package.workspace = "…"` or a comment, which are a member pointing at a root, and not a
# `[[workspace…]]` array table. A U+FEFF counts as leading space on any line, not only the first: Cargo reads a
# manifest that begins with a byte order mark, and a match that is slightly wider than Cargo's is the safe way round.
WORKSPACE = re.compile(r"(?m)^[ \t\ufeff]*\[[ \t]*workspace[ \t]*[.\]]")


def declares_workspace(manifest: Path, memo: dict[Path, bool] | None = None, reader: Reader = read) -> bool:
    """Whether the manifest holds a workspace table header; a manifest that cannot be read declares none. A caller
    that asks about one manifest many times — every member asks about its ancestors — passes its own `memo`, which
    lives as long as its survey does, and `reader` is the seam through which a manifest is read."""
    if memo is not None and manifest in memo:
        return memo[manifest]
    answer = WORKSPACE.search(reader(manifest)) is not None
    if memo is not None:
        memo[manifest] = answer
    return answer


def member_of_workspace(
    root: Path, manifest: Path, present: Callable[[Path], bool] | None = None,
    memo: dict[Path, bool] | None = None, reader: Reader = read,
) -> bool:
    """Whether a `Cargo.toml` (a path from `root`) declares no workspace itself and one above it does, whatever
    ecosystem the directory of that root is reported as: Cargo writes one `Cargo.lock` at the workspace root and
    builds every member from it. `present` says which manifests count as there — the files on disk, or the ones
    Git tracks — and the survey's ownership and the lockfile rule both ask here, so they cannot disagree. `memo`
    and `reader` are `declares_workspace`'s."""
    if declares_workspace(root / manifest, memo, reader):
        return False
    there = present or (lambda candidate: (root / candidate).is_file())
    return any(
        there(above) and declares_workspace(root / above, memo, reader)
        for above in (parent / "Cargo.toml" for parent in manifest.parent.parents)
    )


# In the order rustup prefers them within one directory: the legacy file wins.
TOOLCHAIN_FILES = ("rust-toolchain", "rust-toolchain.toml")


def pinned_channel(text: str) -> str:
    """The `toolchain.channel` of a TOML text, as written; empty where there is no usable one."""
    try:
        toolchain = tomllib.loads(text).get("toolchain")
    except tomllib.TOMLDecodeError:
        return ""
    if not isinstance(toolchain, dict) or "path" in toolchain:  # a path is a place on somebody's machine
        return ""
    channel = toolchain.get("channel")
    return channel if isinstance(channel, str) else ""


def legacy_channel(text: str) -> str:
    """A `rust-toolchain` as rustup reads it: exactly one line is the channel, stripped and nothing else removed;
    more than one is TOML, so `pinned_channel` reads it; none is no pin."""
    lines = text.split("\n")
    if lines[-1] == "":
        lines.pop()
    if not lines:
        return ""
    return lines[0].strip() if len(lines) == 1 else pinned_channel(text)


def rust_toolchain(root: Path, directory: str, reader: Reader = read) -> str:
    """The toolchain channel the candidate's directory pins, or empty where nothing usable is."""
    here = root / directory
    for name in TOOLCHAIN_FILES:
        if os.path.lexists(here / name):
            text = reader(here / name)
            return legacy_channel(text) if name == "rust-toolchain" else pinned_channel(text)
    return ""


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
        {"kind": "rust", "version": rust_toolchain(root, directory)},
    )
