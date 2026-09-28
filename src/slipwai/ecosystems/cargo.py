"""The Cargo row: a directory holding a `Cargo.toml` is Rust built by Cargo, found by the file name alone, so a
manifest the survey cannot parse is still recognised."""
from __future__ import annotations

from pathlib import Path

from .common import Detected, complete, in_dir, prefixed


def cargo(root: Path, directory: str) -> Detected | None:
    if not (root / directory / "Cargo.toml").is_file():
        return None
    return Detected(
        "cargo", "rust", prefixed(directory, "Cargo.toml"),
        complete(
            install=in_dir(directory, "cargo fetch --locked"),
            typecheck=in_dir(directory, "cargo check --all-targets"),
            lint=in_dir(
                directory, "cargo clippy --all-targets --message-format=short -- -D warnings && cargo fmt --check"
            ),
            test=in_dir(directory, "cargo test"),
        ),
        {"kind": "rust", "version": ""},
    )
