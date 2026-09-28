---
description: Say whether a /cruise runner is running, how the last iteration ended, whether it is parked and why, and show the tail of its feed
argument-hint: [lines]
---

# Cruise status

Where a `/cruise` run stands, from disk: the runner's pid file, the iteration log, and the feed the runner
writes as each iteration works. Reading only; nothing here changes the run.

```sh
python3 delivery/scripts/agents/cruise.py status
tail -n ${ARGUMENTS:-40} .specify/cruise-run.log
```

Put every line it printed in your reply, unchanged, in a fenced block, before anything else: the harness folds a command's output, so what it said reaches a person only through your reply. The first command says whether a runner is running, how many iterations
have run, what the last one ended on, whether the run is parked and for what, and whether any command was
refused a permission (`python3 delivery/scripts/agents/cruise.py denials` lists those). The second is the feed's tail — one line per
command, file, and delegate out and back — with the number of lines as the argument, forty by default; where
the log does not exist yet, say that no run has started here. To keep watching the run as it happens, type
`/cruise`: it finds the runner running and takes the watch seat. To end the run, `/cruise-stop`.
