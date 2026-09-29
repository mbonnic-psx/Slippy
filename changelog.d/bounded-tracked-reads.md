PATCH

**`slipwai adopt` no longer reads a tracked symlink to a device until the machine runs out of memory.**
The survey's secrets scan skipped a large file by its `stat()` size, and `stat()` follows a symlink: a tracked
link to `/dev/zero` reported a size of 0, passed, and was read without end — on 2026-09-28 that took a whole WSL
machine down. Every file the survey reads (experimental, with the rest of adoption) is now opened without
blocking, read only when it is a regular file, and read no further than a limit, so a device, a FIFO, a broken
symlink or a file past the limit is read as empty instead of hanging the run or exhausting memory.
