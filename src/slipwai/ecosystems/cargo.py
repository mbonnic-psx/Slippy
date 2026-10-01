"""The Cargo row: a directory holding a `Cargo.toml` is Rust built by Cargo, found by the file name alone, so a
manifest the survey cannot parse is still recognised. The row reads one fact from the manifest — whether it declares
a workspace — because that decides whether check, clippy and test are told to cover every member. Beside the
manifest it reads one configuration fact for the audit: a cargo-deny configuration file (`deny.toml`, `.deny.toml`
or `.cargo/deny.toml`) in the candidate's own directory, and for the mutation `.cargo/mutants.toml` there. It reads the
toolchain the repository pins from `rust-toolchain` or `rust-toolchain.toml` as rustup does (D19): the nearest
directory holding either that it can read, from the candidate's up to the repository root, decides (D24)."""
from __future__ import annotations

import os
import re
import stat
import tomllib
from collections.abc import Callable
from pathlib import Path

from ..bounded_read import MAX_READ
from .common import Detected, complete, in_dir, prefixed, read

Reader = Callable[[Path], str]
# A reader that can say "could not be read as text" (`None`) apart from "read, and empty" (`""`).
TextReader = Callable[[Path], str | None]

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

# What cargo-deny reads in the directory it is run in: any of these three names, as a regular file.
DENY = ("deny.toml", ".deny.toml", ".cargo/deny.toml")
# What cargo-mutants reads at the directory it is run in (the workspace root, for a workspace). A bare `mutants.toml`
# is not read by it.
MUTANTS = ".cargo/mutants.toml"


def optional_tools(root: Path, directory: str, flag: str) -> dict[str, str | None]:
    """The commands whose tools are the crate's own choice, each proposed only where its configuration file is a
    regular file in the candidate's directory; elsewhere the key is a written no. `flag` is the row's `--workspace`."""
    here = root / directory
    deny = any((here / name).is_file() for name in DENY)
    return {
        "audit": in_dir(directory, f"cargo deny{flag} check advisories") if deny else None,
        "mutation": in_dir(directory, f"cargo mutants{flag}") if (here / MUTANTS).is_file() else None,
    }


# In the order rustup prefers them within one directory: the legacy file wins.
TOOLCHAIN_FILES = ("rust-toolchain", "rust-toolchain.toml")


def pinned_channel(text: str) -> str:
    """The `toolchain.channel` of a TOML text, as written; empty where there is no usable one. A leading byte order
    mark is removed first: rustup reads past it (R19), and `tomllib` does not."""
    try:
        toolchain = tomllib.loads(text.removeprefix("\ufeff")).get("toolchain")
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


# What rustup will install by name, and so what a workflow's `toolchain:` input can safely carry (D23): a channel
# holding a newline, a space, a quote or a replacement character is nothing rustup reads, and is not recorded.
TOOLCHAIN_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]*")


def readable_text(path: Path, limit: int = MAX_READ) -> str | None:
    """The file's text; `None` where it cannot be read as UTF-8 text — missing, a dangling link, not a regular file
    (never blocked on: issue #13), unreadable, not valid UTF-8; empty where it is larger than `limit`, as
    `bounded_read.read` has it, which is a file that was read and names nothing."""
    try:
        descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
    except OSError:
        return None
    try:
        if not stat.S_ISREG(os.fstat(descriptor).st_mode):
            return None
        with os.fdopen(descriptor, "rb", closefd=False) as handle:
            data = handle.read(limit + 1)
    except OSError:
        return None
    finally:
        os.close(descriptor)
    if len(data) > limit:
        return ""
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def rust_toolchain(root: Path, directory: str, reader: TextReader = readable_text) -> str:
    """The toolchain channel the candidate pins, as rustup would find it: from the candidate's directory up to and
    including the repository root, never above, the nearest directory holding a toolchain file it can read decides —
    even one that names no channel — and a name that cannot be read as text (D24) is passed over, to the other name
    in its directory and then upward. No file read on the way up is no pin."""
    here = root / directory
    while True:
        for name in TOOLCHAIN_FILES:
            text = reader(here / name)
            if text is not None:
                channel = legacy_channel(text) if name == "rust-toolchain" else pinned_channel(text)
                return channel if TOOLCHAIN_NAME.fullmatch(channel) else ""
        if here == root or root not in here.parents:
            return ""
        here = here.parent


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
            **optional_tools(root, directory, flag),
        ),
        {"kind": "rust", "version": rust_toolchain(root, directory)},
    )
