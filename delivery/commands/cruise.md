---
description: Run /drive as driver and product owner, iteration after iteration, until every specification is satisfied — stopping only for a human
argument-hint: [--feature <name>] [kick-off: what this run is for, where the brief or PRD is] | unblock: <what the outer loop saw> | told: <a person's message>
---

# Cruise

`/drive` takes one slice from wherever it stands to an actor-visible demo and stops for a product decision, an
unavailable input, an exhausted split and the demo. This command runs **that ladder — `delivery/commands/drive.md`,
every rule as written** — with nobody at the wheel: it decides what the ladder would have asked a person,
runs each demo as the actor, and re-enters the ladder until the specification under `specs/<feature>/` is
satisfied. Nothing about what a stage produces changes; what changes is who answers. It stops for a human and
for nothing else. An iteration is one invocation of this command; the outer loop that re-invokes it with a
fresh context is `python3 delivery/scripts/agents/cruise.py run` (`make -f delivery/Makefile cruise`), on every harness. Typed in a session, nothing
re-invokes it, so the command starts that loop instead of running the ladder here (*Before anything*, below):
the runner is the one thing that continues a run, whatever the harness.

This repository adopted the method around code that was already there, and two of its stops are a person's word — the rows at the foot of the table. A run here proceeds on stated assumptions and `Proposed` records rather than parking, and a person confirms or overturns them afterwards.

## Before anything: refuse, or start

Read `.specify/cruise.json`. `enabled: false`, or `.specify/cruise.stop` present, is a refusal in one line that says which. No
`specs/<feature>/spec.md` is a refusal too: a specification is the one thing a person brings. In an iteration a
refusal still ends on a last line, because the runner reads nothing else and would spend its stuck budget on a
plain one: `enabled: false` or the stop file ends on `cruise: stopped: human` — a person turned it off — and a missing
specification on `cruise: parked: a specification under specs/<feature>/spec.md`; typed in a session, the
refusal is plain, since nothing reads it. Then run
`python3 delivery/scripts/agents/cruise.py loop`: it says what is reading this session's last line. **Where it says nobody is** — this
command was typed in a session, and no runner set `CRUISE_RUNNER` and `CRUISE_ITERATION` — run
`python3 delivery/scripts/agents/cruise.py start` with everything typed after `/cruise` as its arguments, verbatim, and repeat what it
printed: the runner it started drives the ladder from here, one fresh session per iteration, and this session
runs no stage of it. A refusal is the whole answer — a runner already running, the stop file present, no
harness on PATH it can run an iteration through. Then take the watch seat (*The watch seat*, below). **Where
it says the outer loop started this session**, this is an iteration: read the owner brief (`.specify/product-owner.md`)
and every standing entry in `specs/<feature>/decisions.md`, and say the iteration number from `specs/cruise-log.jsonl`, the branch and its
distance from trunk, and that a person stops this run with `touch .specify/cruise.stop`. Where `.codegraph/` is in the
tree, the runner has already opened, checked and synced it for this iteration: a caller or blast-radius question
is one call — `delivery/scripts/codegraph callers <symbol>`, or `codegraph_explore` — and `python3 delivery/scripts/agents/cruise.py status`
counts, per delegate, who asked it and who searched the source for a symbol first. Open a `skipper`, `hand` or `bosun`
benchmark entry around each delegation the way every stage is bracketed, and
pass `driver=cruise` to every `end` this iteration closes.

**The argument is the kick-off.** What a person typed after `/cruise` — what this run is for, where the brief
or the PRD is, which feature — reaches the first iteration of the run and no other: every later iteration
runs bare and derives its stage from disk. So the first iteration writes down whatever the kick-off says that
must outlive it — a PRD it names becomes the specification through the ladder's own stages, a preference it
states goes into the owner brief (`.specify/product-owner.md`), a scope it sets is a decision entry — before it does
anything else. Two things the runner passes itself recur: a feature named with `--feature` on `run` or `start`
(`make -f delivery/Makefile cruise FEATURE=<name>`) is the first word of every iteration's argument and scopes the run to that
feature's specification — the ladder is entered for it and no other — and `unblock: <reason>` is what the runner
says when a run makes no progress (*Blocked: the bosun protocol*, below). A third is a person's: `told: <message>`
is what somebody queued for the run through `/cruise-tell` (`python3 delivery/scripts/agents/cruise.py tell`) since the last iteration
started, one `told:` per message in the order they were sent, and it reaches the iteration the runner starts next —
never the one in flight, unless they ended it for the message. Read it before the first stage and act on it
first: a steer takes precedence over what the artifacts alone would make this iteration do, a fact the run lacked
is the answer to a block, and a scope or a preference is written down the way the kick-off is — into the owner
brief (`.specify/product-owner.md`) or a decision entry with `Decided by: human` — so it outlives this iteration. A message
never changes a setting; say so and point at `/cruise-settings` where one asks for that.

## The watch seat

After `start` — or where `start` said a runner is already running — run `python3 delivery/scripts/agents/cruise.py watch`. It prints
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
## Run the ladder, and answer at its stops

Run `delivery/commands/drive.md` from *Enter at the first incomplete stage* to its end, exactly as written — the entry
stage from artifacts, the branch check, *Who runs each stage*, the benchmark bracket, the ready-set rules and
the concurrent fan-out. Wherever that command would stop for a person, this table says what to do instead;
where the table is silent, the ladder's own rule stands.

| # | Where `/drive` stops | What `/cruise` does there | Recorded in |
|---|---|---|---|
| 1 | The checkout is behind trunk, or the fetch failed | Fetch and fast-forward where the tree is clean; rebase a `slice/<id>` branch that has local commits. A conflict parks. A fetch that could not run — no remote, or one this environment cannot reach — is what the ladder says it is, *could not verify this checkout is current*, said in the evidence line, and the run goes on: without a remote the local branch is the claim, as the ladder says. Never derive from a tree known to be stale | `specs/cruise-log.jsonl` |
| 2 | Principles: the constitution is unratified | `constitution: ratify` — the skipper drafts it with `/speckit-constitution` from the spec and the owner brief, answers `/constitution-coverage`, and ratifies it with the line `ratified by cruise (skipper) — pending human review`. `park` stops here instead | `constitution.md`, a decision entry |
| 3 | Product specification missing | Refuse to start. A spec is the one thing a person brings; `/cruise` writes no product from nothing | — |
| 4 | Which service or bounded context owns a slice | Decide against each service's recorded `purpose`; where none covers it, record the purpose the spec implies with `slipwai describe-service <name> --purpose` and decide. Contexts by the language test, recorded the same way with `--context` | the model or plan, `project.json`, a decision entry |
| 5 | Slice gaps: a question at a time over the criteria in `spec.md` | The conversational loop runs with the owner as the other party: each gap is answered — host or skipper by `decide` — and written back as the criterion or state. `gaps=N` still counts | the criteria in `spec.md`, a decision entry per question |
| 6 | Release constraint | Take the stage's own recommendation. `release: flagged` — every slice continues or opens a flag seeded `off`, so every merge is dark and a person flips keys. `park` stops at the push instead | `plan.md`, the flag file, a decision entry |
| 7 | "A release they want now" — no flag, or a flag already on, before the push | Never answered by the machine: a release nobody asked for is on the catastrophic list. Under `flagged` it does not arise; where it does, park with the exact question | a `parked` line in the log |
| 8 | A delegate hands back a product question in `plan.md` | Answer it, un-block the slice, re-dispatch with the entry as a pointer, never as a conclusion | `plan.md`, a decision entry |
| 9 | Converge appended Phase 4 tasks; the after-converge `/gaps` says stop | Already bounded by the ladder: continue to the demo as `delivery/commands/drive.md` says | — |
| 10 | The demo stop | Delegate to `drive-hand` with exactly what the stop hands a person, plus the acceptance script; take its verdict as the actor's and re-enter the ladder where demo feedback re-enters. Acceptance says `accepted-by: drive-hand` on the register row or status flip | `specs/<feature>/slices/<id>/demo-log.md`, `benchmark.json` `outcome=`, the register |
| 11 | The ready set is empty | Not a stop: the completion audit below. Only an audit with nothing left is `done` | `specs/<feature>/cruise-report.md`, decision entries |
| 12 | An input that is genuinely unavailable — a credential, an external system, a person's approval | Never invented. Mark the slice blocked, take the next ready slice, and hand the blocker to `drive-bosun` (*Blocked*, below): a stub behind the port, recorded as a stub. Park only at the catastrophic, or when the bosun could not move it | a decision entry, the stub in `plan.md`, ⛔ on the board |
| 13 | Ground: a convergence row still `unrecorded` on an axis the slice touches | A fact about the world is not a decision, and is never invented. The row stays `unrecorded`; the bosun works on the survey's `detected` value as a stated assumption, never marked `confirmed` | a decision entry naming the assumption |
| 14 | The change-strategy ADR at `Accepted` | The word is a person's. The bosun proceeds on the recommendation at `Proposed` and says so | the ADR at `Proposed`, a decision entry |
| 15 | Quick wins and method slices offered from the programme | Take the programme top-first, as the stage recommends | `project.json` `planned`, a decision entry |

## Deciding: the skipper protocol

A product question is decided, never deferred, and every decision is written twice — into the artifact the
stage owns, and as the next entry of `specs/<feature>/decisions.md`, which is the only place a person can read every decision
this run took. Read the standing entries before any decision, so a hundred answers stay consistent with each
other. Under `decide: recommended-first`, decide here when the stage itself recommends an answer (the
release-constraint stage says *recommend the answer with its reason rather than asking an open question*),
when a standing entry already covers the question, or when the specification or the constitution answers it
outright. Anything else is an **open question**: delegate it to one fresh `drive-skipper` delegate with the
question, the stage, the options and the recommendation in its brief — the spec, the constitution, the owner
brief and the log are the standing part of its own brief — and **the number its entry will carry**. `D<n>` is
allocated here, before dispatch: the next after the last entry in `specs/<feature>/decisions.md`, one per delegate in dispatch
order where several go out at once. The delegate returns the whole entry under that number and writes
nothing; this session appends it, in number order, and writes the decision into the artifact the stage owns.
Under `decide: skipper-always`, every question goes to the delegate. Several open questions in one turn are
several concurrent delegates, each with its own number; a slice delegate that handed one back does not wait
on the others. Every other identifier a decision adds to a shared artifact — a requirement, a criterion, an
example, a state — is allocated the same way: by this session, after the delegates return, in dispatch order.
A delegate cannot see what its siblings are adding, so it numbers nothing they share; two entries that came
back as the same `D3`, with requirement ranges that overlapped, were exactly the reconciliation by hand this
protocol exists to end.

The entry's shape, which `make -f delivery/Makefile check-decisions` holds:

```markdown
## D<n> — <the question, in one line>
- **Stage:** <stage> · **Slice:** <id> · **When:** <ISO instant> · **Iteration:** <n>
- **Question:** <as the stage raised it>
- **Options:** <each, marking the one the stage recommended>
- **Decision:** <one>
- **Why:** <in the actor's terms>
- **Decided by:** host (stage recommendation) | host (standing decision D<m>) | drive-skipper (<model>) | drive-bosun | human
- **Confidence:** high | medium | low · **Would reverse if:** <the one condition>
- **Written to:** <the artifact paths the answer went into>
- **Status:** standing | overridden by D<m> | overridden by human <date>
```

A decision that would break a constitution MUST is not available; the skipper says so and the question parks.
A fact nobody here has — a credential, a third party's behaviour, an approval — is `unavailable`, and the
skipper's brief says which those are: it is never decided, whatever `decide` says. A person overrides a
decision by editing its `Status` and writing the answer they want into the artifact; the next iteration
re-derives the entry stage from that artifact, the way demo feedback re-enters the ladder.

**A decision that outlives its slice is also an ADR.** Ask the `architecture-decisions` skill's one question
of every entry, host-decided or skipper-decided: would reversing it cost a migration rather than a refactor —
an event's schema or name, stream identity, tenancy, the store, personal data, identity, a new dependency, a
published contract? Where it would, write `delivery/docs/adr/NNNN-<title>.md` in Nygard's five sections at `Proposed` —
the run never accepts its own architecture decision — with the next unused number, allocated here the way
`D<n>` is, and name it in the entry's `Written to` beside the artifact. The entry is the log of what was
decided; the ADR is where the next slice looks for why, and `specs/<feature>/cruise-report.md` lists every ADR still `Proposed`.

## Demonstrating: the hand protocol

At the demo stop, compose everything `delivery/commands/drive.md` says the stop must contain — the board, the literal
command or URL, the seed data, the expected result, the running process — and hand it, with the slice's
acceptance script, to one fresh `drive-hand` delegate instead of a person. The brief also names where the hand's
ladder starts, which is `.specify/cruise.json`'s `hand` and nothing the delegate can read for itself: `browser` is
`agent-browser` where the slice has a screen, then a browser tool the harness exposes, then HTTP, then the CLI;
`http` starts at HTTP; `cli` at the CLI — each rung falling through to the next where it cannot run and saying
so, and none climbing back above the one the setting names. Its verdict is the actor's: `accepted`
continues to *After acceptance*, `behaviour` re-enters the ladder at the stage that owns the change with the
example that shows it, `implementation` is a task. Record `outcome=` on the demo entry from the verdict, and
write `accepted-by: drive-hand` beside the register row or status flip, so a person can tell which demos a person
has seen.

**Then stop what the demo started.** The ladder leaves the app running past the end of the turn because the
actor's first action is opening it; here the actor was the hand, and it has finished. Once the verdict is
recorded, end it — `make -f delivery/Makefile demo-down` or `make -f delivery/Makefile services-down` where the demo used them, otherwise the process
the stop started — before Phase 4 or the next slice's delegate, which starts what its own demo needs and would
otherwise find the port taken; the runner ends anything an iteration still leaves. The hand's writes are
`specs/<feature>/slices/<id>/demo-log.md` — one section per demo — and its evidence under `specs/<feature>/slices/<id>/demo/`:

```markdown
## <ISO instant> — <accepted | behaviour | implementation> · iteration <n> · drive-hand (<model>)
- **Started with:** <the literal command or URL> · **Seeded:** <what, or none>
- **Driven through:** agent-browser | <harness browser tool> | HTTP | CLI — <why, where not the first>
- **Examples:** <one line each — R1 e1: passed · R2 e1: failed, expected X, saw Y · R3 e2: unreachable, why>
- **Evidence:** <paths under demo/>
- **Feedback:** <what re-entered the ladder and at which stage, or the note for the next slice>
```

## When the ready set is empty: the completion audit

An exhausted split is where `/drive` stops and where this command does its last stage. Delegate `/gaps` over
the whole of `specs/<feature>/spec.md` against what shipped — one `drive-gaps` delegate per feature area,
concurrently, as the post-implementation pass is per seam — and put every finding to the skipper protocol:
a criterion nothing built becomes a slice, appended to the split with `/story-splitting`, and the ladder is
re-entered for it; a finding the owner rules out of scope is a decision entry saying so. Write
`specs/<feature>/cruise-report.md`: what the specification asked, what shipped, every out-of-scope decision, and every entry a person
has not yet reviewed. Only an audit with nothing left to build ends with `cruise: done`.

## The iteration contract

Spend this context on one unit of work, and then end the iteration rather than starting the next unit in a
context that has already carried one. **Before the split exists**, the unit is the upstream stages together —
principles, the specification and the split — through to the split's first ready set: each reads the one before it and none is a slice, so the
iteration does not end inside them; it ends when the split is written, or at a park. **From the split on**,
the unit is one slice through Phase 4 and its done marker, or one concurrent fan-out through its merges in
split order. `delivery/commands/drive.md` says *do not wait to be invoked again*; here the
outer loop is what re-invokes, with a fresh context, which is the rule every delegate already lives by.
Between stages, look for `.specify/cruise.stop`: present, finish the stage's own writes, commit what is green, and end
on `cruise: stopped: human`. At the same boundaries run `python3 delivery/scripts/agents/cruise.py told`: it prints what a person queued through
`/cruise-tell` since this iteration started, one `told:` line each, or nothing — and what it printed is acted on
before the next stage, exactly as a `told:` argument would have been; it is taken as it is printed, so the next
iteration is not given it again, and the runner puts it in this iteration's log entry.
At every stage boundary and every delegation, rewrite the checkpoint (*Checkpoint*, below).
Where nothing can move — every ready slice blocked and the bosun could not move one, or a blocker is on the
catastrophic list — end with `parked` and the exact thing a person must provide or decide; the loop waits,
it does not exit. The last line of every iteration is one
of these, and the outer loop reads nothing else:

- `cruise: continue`
- `cruise: done`
- `cruise: parked: <what a person must provide>`
- `cruise: stopped: human`

**An iteration ends only on one of those four lines.** Any other message that ends a turn is a stop, whatever
it says it is about to do: in Claude Code a message with no tool call *is* the end of the turn, so "continuing
into the plan now" is a stop that called itself progress. And an iteration is only ever a session the runner
started: where `python3 delivery/scripts/agents/cruise.py loop` said nobody is reading, this command started the runner and watched it
(*no outer loop is reading this: a `/cruise` typed in a session starts the runner — `python3 delivery/scripts/agents/cruise.py start` — and then watches it with `python3 delivery/scripts/agents/cruise.py watch`; the runner drives the ladder from here, a fresh session per iteration, and this session runs no stage of it*). Neither rule is left to this text. The runner reads the last line and re-invokes, on every
harness, and a session that ended without one is no progress to it. Where a harness lets a hook refuse the
end of a turn, the project's hook file runs `python3 delivery/scripts/agents/cruise.py stopping` there — `.claude/settings.json` runs
it as Claude Code's `Stop` hook, `.cursor/hooks.json` as Cursor's `stop`, `.gemini/settings.json` as Gemini
CLI's `AfterAgent`; `delivery/scripts/agents/registry.json`, `hooks`, says what each harness has — and while a runner
started the session, `specs/cruise-checkpoint.md` says an iteration is in flight and `.specify/cruise.stop` is absent, it refuses a
turn that ends on anything but a last line and hands back the checkpoint's `Next:` line as the reason. It
lets go after three holds against a checkpoint nothing rewrote, so a session that cannot move is not held
forever; rewriting the checkpoint at every stage boundary is what keeps it moving.

## Checkpoint: what survives a compacted context

A harness can summarise this context at any point — Claude Code compacts, Gemini CLI compresses — and what a
summary loses is the state nothing on disk carries: which delegates are out and with what manifest, a
question half-answered, which slice's demo comes next. So keep `specs/cruise-checkpoint.md` current: rewrite it at every
stage boundary and every delegation, in this shape:

```markdown
# Cruise checkpoint — iteration <n>
- **Feature:** <feature> · **Slice:** <id> · **Stage:** <stage> · **Written:** <ISO instant>
- **Delegates out:** <type · manifest · what it was asked>, one per line, or none
- **Open question:** <the question and the stage that raised it, or none>
- **Next:** <the one next step, in the ladder's words>
- **Rules:** run delivery/commands/drive.md as written; decide at its stops by the stop table; decisions to
  decisions.md; demos to demo-log.md; end with one of the four last lines; `.specify/cruise.stop` stops the run
```

At the start of every stage, and whenever this context looks summarised — the iteration number is not in
memory, or a summary opens the context — read the checkpoint before acting; `python3 delivery/scripts/agents/cruise.py resume` prints
it with the rules beside it, and prints nothing where no iteration is in flight. Where the harness can run a
command after compaction, the project's settings do that for you: `.claude/settings.json` runs `resume` on
Claude Code's `SessionStart` with the `compact` matcher and stamps the checkpoint on `PreCompact`, and
`delivery/scripts/agents/registry.json`, `compaction`, says what each harness can. A checkpoint left by an earlier
iteration is a lead, never a result: its delegates ended with that session, so verify what they left in the
tree before continuing. The runner deletes the checkpoint when an iteration ends `done` or `stopped`, and so
does the stop hook; one that ends `continue` or `parked` leaves it for the next.

## Blocked: the bosun protocol

Blocked is work before it is a stop. Whenever this ladder would park — an input nobody here has, a question
whose every option seems to break a MUST, a checkout that will not rebase, a delegate that died mid-slice, a
run the outer loop reports as making no progress (`/cruise unblock: <reason>` is how it says so) — first
mark the slice blocked, take the next ready slice, and delegate the blocker to one fresh `drive-bosun` delegate
with what was tried in its brief. Its standing brief carries the moves, in order: stub the world behind the
port the missing thing sits behind, recorded as a deliberate stub in `plan.md` so the board shows it under
*Not working yet*; narrow the reading so every MUST holds and defer the rest behind the flag, with the
amendment a person may want as an ADR at `Proposed`; repair the run. Every move is an entry in
`specs/<feature>/decisions.md` with `Decided by: drive-bosun`, its *Would reverse if* naming what a person must eventually
supply, and a task in the next slice to remove the stub when they do — so nothing done to get moving is done
silently, and a person reading the log sees every workaround in one place. `unblock: park` turns this off
and parks at once.

`unblocked` re-enters the ladder for the slice where it stood. `cannot` is a park with what was tried. And
`catastrophic` is the one word that always parks, because these are what no workaround may be:

- destroying data or history — dropping a database or volume, rewriting or deleting a shared branch, deleting what nobody can recover
- releasing what a person has not asked for — turning a flag on, deploying or promoting to production, merging anything that reaches a real actor
- spending or exposing — paying for anything, creating or revealing a secret, widening permissions
- weakening security — bypassing authentication, loosening a MUST about money, identity or a boundary in production code
- discarding a person's commits to make a checkout consistent
- making a gate pass by changing the gate — anything under `delivery/scripts/`, the `Makefile`, anything under `tools/`, CI, a harness's hook settings: a gate is satisfied in the tree it measures, or the run parks with the gate's own output as the reason
- a run the bosun could not move — the blocker survived its attempt, or the attempt itself would need one of the above

A park is therefore rare, and it says which of those it met, or what the bosun tried. `python3 delivery/scripts/agents/cruise.py
status` shows a parked run's reason beside its checkpoint.

**A failing gate is never repaired in the gate.** `make verify` red on the slice's own tree is the slice's
work. Red for a reason the tree cannot fix — a browser this machine has not got, a tool not installed, a
script of a kit's that crashes — is a park: `cruise: parked: <gate>: <its own last lines>`, so a person
reads what the gate said and not what an iteration made of it. A gate reported as skipped is neither. This is
not left to the text: on Claude Code, `python3 delivery/scripts/agents/cruise.py guard` runs as the `PreToolUse` hook of every editing
tool and refuses, in a runner's session, an edit under `delivery/scripts/`, `tools/`, the `Makefile`, CI or the hook
settings before it lands; and the runner compares those files before and after every iteration, on every
harness, and parks the run on any change — `controls_changed` on the log entry names the files — whatever
the iteration's last line said. A file installed under `tools/` by `./delivery/init --extension` is not a change.

## What holds throughout

- **Parallelism is inherited and widened.** Everything *Running ready slices concurrently* allows runs the
  same way here. What no longer serialises the fan-out are the two stops that were a person's: a delegate's
  product question is answered while its siblings keep running, and a slice's demo runs in the hand while
  the next slice's delegate is still converging. Phase 4 stays one slice at a time on `main`. The worktrees
  beside the checkout are writable on Claude Code because the runner starts every iteration with `--add-dir`
  for the directory the checkout sits in (`delivery/scripts/agents/registry.json`, `headless.worktreeFlags`); on a
  harness whose row has no such flag, make the worktree inside the tree where the harness offers one, or run
  the ready slices one at a time here and say so, as the ladder does where the harness cannot delegate.
- **Flags stay off.** Under `release: flagged` nothing this run merges is visible to a real actor until a
  person flips a key. Turning a flag on is never a decision the log can contain.
- **The constitution's MUSTs are the floor.** No decision waives one; a question whose every option breaks
  one goes to the bosun for the reading that keeps them all, and parks only if there is none.
- **The hand edits no code.** A defect it finds is a task; a fix there would make the verdict evidence for
  itself.
- **Stuck is detected.** `stuck_after` iterations with the same artifact fingerprint give the bosun one
  iteration to move it, then park the loop; the same open question raised twice in one iteration goes the
  same way. A run that loops is not a run.
- **The record says who drove.** `driver=cruise` on every benchmark entry, `skipper`, `hand` and `bosun` as stages
  of their own, so a decision's cost and a demo's cost are numbers `make -f delivery/Makefile benchmark` can read.
- **Settings change only through `/cruise-settings`**, never inside an iteration, and `.specify/cruise.json` is
  committed: a run's rules are a diff.
