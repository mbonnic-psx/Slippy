---
description: Queue a message for a running /cruise — a steer, a fact it lacked, a scope — which the next iteration carries; `--now` ends the iteration in flight for it
argument-hint: [--now] <what the run should know or do next>
---

# Cruise tell

A person's word to a run that is already going. It is queued, never pushed into the iteration in flight: the
runner reads the inbox before it starts each iteration and hands the message over as `told: <message>` in that
iteration's argument — the route the kick-off takes — and the iteration itself asks for what was queued
between stages. With `--now` as the first word, the iteration in flight is ended for it, the way `/cruise-stop
now` ends one, and the next iteration starts at once with the message; its increment commits are on the slice
branch, and the stage's uncommitted work is what it costs.

```sh
python3 delivery/scripts/agents/cruise.py tell <<'EOF'
$ARGUMENTS
EOF
```

The message goes in as written — the quoted heredoc is so a quote or a `$` inside it never reaches the shell —
and `--now` is read from its first word. Put every line it printed in your reply, unchanged, in a fenced block, before anything else: the harness folds a command's output, so what it said reaches a person only through your reply. It says whether the message waits for the iteration in
flight to end, resumes a parked run, or ended the iteration for it — and, where no runner is running, that the
first iteration of the next run carries it. What was queued and not yet taken is in `.specify/cruise-inbox.jsonl`; `/cruise-status`
lists it, and every message an iteration was given is in that iteration's entry in the run's log. A message
is not a setting: `/cruise-settings` is still how a rule of the run changes. `make -f delivery/Makefile cruise-tell
MSG="…"` is the same from a terminal, `CRUISE_FLAGS=--now` for the immediate form.
