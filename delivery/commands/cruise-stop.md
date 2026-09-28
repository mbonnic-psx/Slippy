---
description: End a /cruise run after the iteration in flight, or at once with `now`
argument-hint: [now]
---

# Cruise stop

A person's stop, the one thing a `/cruise` run stops for. Without an argument the run ends after the iteration
in flight — the command checks between stages, finishes the stage's own writes, commits what is green, and
ends. With `now`, the iteration in flight is ended too: its increment commits are on the slice branch, and the
next run re-derives its stage from disk, so nothing is lost but the stage's uncommitted work.

```sh
python3 delivery/scripts/agents/cruise.py stop $(test "$ARGUMENTS" = now && echo --now)
```

Put every line it printed in your reply, unchanged, in a fenced block, before anything else: the harness folds a command's output, so what it said reaches a person only through your reply. Then say one thing more: `.specify/cruise.stop` is now present, and `/cruise` refuses to
start another run until it is removed — `rm .specify/cruise.stop` when they want one. Where no runner was running, the
script says so and the file is written anyway, for the same reason. `make -f delivery/Makefile cruise-stop` is the same
from a terminal, `CRUISE_FLAGS=--now` for the immediate form.
