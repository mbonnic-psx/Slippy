---
description: Take the watch seat beside a running /cruise — print the feed as the runner writes it, return at each boundary and watch again, answer a person typing here — without starting anything
---
# Cruise watch

The watch seat on its own. `/cruise` takes it after starting the runner, and a `/cruise-status` between reads it
once and stops; this sits back down where the feed left off, in this session, and starts nothing — where no
runner is running, `watch` says so and this ends. `make -f delivery/Makefile cruise-watch` is the same seat from a terminal.
To take it: run `python3 delivery/scripts/agents/cruise.py watch`. It prints
what the iteration does as it happens, one line per command, file, and delegate out and back, and returns at
the iteration's end, a park, the run's end, once the feed has gone quiet for a moment, or after a minute and a
half with nothing new; its last line says which. **Put every line it printed in your reply, unchanged, in a
fenced block, before anything else** — the harness folds a command's output, so the feed reaches a person only
through your reply — **and where it says the run continues, or the iteration is in flight, run `watch` again
at once**; where it says parked, ended, or no runner, repeat what it said and end the turn. A watch the
harness cut short — a tool timeout, with no last line from `watch` — is watched again, not asked about. Where
the harness can run a command in the background and re-invoke this session with its output when it returns,
run `watch` that way, so the turn ends between watches and a person can type in the gap. Watching is only ever
reading — the runner needs nothing from this session, and a turn that ends here ends nothing else — so it is
never a reason to run a stage of the ladder in this session.

**A person typing here is talking to you, not stopping the run.** Answer them — what the feed shows, what
`.specify/cruise.json` says (`python3 delivery/scripts/agents/cruise.py` prints every setting and what it controls), what `python3 delivery/scripts/agents/cruise.py status`
says, what `specs/<feature>/decisions.md` records — and change a setting through `/cruise-settings` where they ask; it takes
effect at the next iteration. Where what they typed is for the run — a steer, a fact it was missing, a scope, an
answer to the question it parked on — queue it with `python3 delivery/scripts/agents/cruise.py tell <<'EOF'` … `EOF` (`/cruise-tell`),
the message as they said it, and repeat what the script printed: the next iteration carries it, a parked run
resumes with it, and `--now` as its first word ends the iteration in flight for it, which is done only when they
ask for that. Then watch again. Only `touch .specify/cruise.stop`, `python3 delivery/scripts/agents/cruise.py stop`, or
`make -f delivery/Makefile cruise-stop` ends the run, and only when they ask for that. Where they ask where the run stands or what is next, `/where-are-we` and `/whats-next` read the runner's state first (`where`) and answer from it — never by sending the question down to the iteration.
