---
description: Say what is next — one slice, one stage, one command — read off disk and changing nothing
argument-hint: [feature]
---

# What's next

Answer *what do I do now* in at most six lines. `/where-are-we` draws the whole board; this names the next step
and stops. Given a feature, read `specs/<feature>/`; given nothing, the active one recorded in `.specify/feature.json`, or the only one under `specs/`.

## The answer

Four labelled lines, in this order, every one in the actor's vocabulary rather than a test name:

- **Next:** the slice — its id and the thing it lets the actor do — or, above the split, the stage of the
  ladder still owing an artifact
- **Stage:** the part of that slice to do now — the first stage of `/drive`'s ladder whose artifact is
  missing, empty or still a placeholder, with the artifact that says so; mid-implementation, the next unchecked
  task by id and rule; after a converged verdict, the demo
- **Because:** the one fact that selected it — the last slice accepted, this one the earliest ready and
  unclaimed; or the slice in progress on this branch; or the only slice left unblocked
- **Run:** the command to type — `/drive <id>`, or the upstream command the stage names
  (`/story-splitting`, `/speckit-specify`, `/speckit-constitution`)

Where more than one slice is ready and unclaimed, headline the earliest in split order and add one line —
*and N more ready in parallel: …* — since `/drive` will take them together. Where nothing can start, the
first line is **Blocked:** with the `depends_on` or the open `CRITICAL` that holds it, and **Run:** is what
clears it. Where a slice is claimed by another session, say so and name the next unclaimed one instead.

## Where it is read from

The same artifacts the board is read from, so the two can never disagree:
the ordered split and its `## Slice graph`, the register at `specs/<feature>/slices/README.md` — a row
marks a slice accepted; a slice with `plan.md` under `slices/<id>/` and no row is in flight —
and the forge's `slice/<id>` branches (`git ls-remote --heads origin 'slice/*'`), which are the claims — read, never assumed: where that command fails the board says the claims could not be read, and shows no slice as unclaimed on the strength of a failed read. The stage comes from the ladder — the first stage whose artifact is missing, empty, or
still a placeholder — and the next task from that slice's `tasks.md`. Check the branch first, the way `/drive`
does: a checkout behind trunk answers this question wrongly with confidence, so say when it could not be
verified as current.

## What it never does

- **Runs nothing** and edits nothing: read-only, safe to ask at any point on the ladder.
- **Never more than six lines.** A second slice, a count of tasks, a history of what was done — that is the
  board, and `/where-are-we` draws it.
- **Invents nothing.** No slice the split does not name, no stage the artifacts do not select. Where the
  artifacts cannot say, the line reads `unknown` with the artifact that would have said, and **Run:** is
  `/where-are-we`.

## Under a `/cruise` run

Run `python3 delivery/scripts/agents/cruise.py where` first. It reads the run's pid file, log and checkpoint, changes nothing, and
prints **nothing where no runner is running** — then everything above stands exactly as written, and this
section does not apply. Where it printed lines, a run is going. Put every line it printed in your reply, unchanged, in a fenced block, before anything else: the harness folds a command's output, so what it said reaches a person only through your reply. Then the answer changes in one
place: the step for a person — **Run:** here, the board's ➡️ *Next* row there — is not a command to type,
because the runner is on it, at the slice and stage those lines name. Say so, and say what a person can do
from here: `/cruise` watches the run, `/cruise-tell` steers it, `/cruise-stop` ends it; where the lines say
the run is parked, the step is what the park names, and `/cruise-tell` with it resumes the run. Nothing else
changes: the board is read from the same artifacts, and a slice the runner holds is one in progress, never
one to take.
