PATCH

**`cruise.py stop --now` ends the harness session it interrupts, not only the shell in front of it.** The runner
starts each iteration through a shell and stopped only that shell, so the harness under it — and anything the
harness started — went on running, and on a paid API went on spending, after `stop --now` said the iteration
was ended. Each iteration now runs in its own process group, and `stop --now` or Ctrl-C on a foreground
`make cruise` ends the whole group.

**Catch-up.** `slipwai migrate` brings the new `scripts/agents/cruise.py`. A run stopped with `--now` before it
may have left a harness running: `pgrep -fa "cruise"` finds it.
