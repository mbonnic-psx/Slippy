"""Reading a file from a tree this factory did not write, without letting the file decide what the read costs.

A tracked symlink to `/dev/zero` reports a size of 0 and never ends, a FIFO blocks the open itself, and a link to a
directory is no text at all: on 2026-09-28 the survey read the first of those until WSL ran out of memory
(issue #13). So the file is opened without blocking, checked by the descriptor it gave, and read no further than a
limit — one byte past it says the file is too big, whatever its size claimed.
"""
from __future__ import annotations

import os
import stat
from pathlib import Path

# The most a tracked file is read for: past it the file is not text anybody wrote, and reading on would let one
# file hold the whole machine's memory.
MAX_READ = 16 * 1024 * 1024


def read(path: Path, limit: int = MAX_READ) -> str:
    """The file's text, or empty where it is not a regular file of at most `limit` bytes."""
    try:
        descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
    except OSError:
        return ""
    try:
        if not stat.S_ISREG(os.fstat(descriptor).st_mode):
            return ""
        with os.fdopen(descriptor, "rb", closefd=False) as handle:
            data = handle.read(limit + 1)
    except OSError:
        return ""
    finally:
        os.close(descriptor)
    return "" if len(data) > limit else data.decode("utf-8", "replace")
