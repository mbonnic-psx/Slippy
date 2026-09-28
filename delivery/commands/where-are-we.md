---
description: Show where the product stands — the progress board, on demand, read off disk and changing nothing
argument-hint: [feature]
---

# Where are we

Answer the question the product owner would otherwise have to ask: what works, what is being built, what is
left, and what comes next. Say it in the actor's vocabulary, from artifacts on disk, and change nothing. For
the one-step answer — which slice, which stage, which command — `/whats-next` reads the same artifacts and
says only that.
Given a feature, read `specs/<feature>/`; given nothing, the active one recorded in `.specify/feature.json`, or the only one under `specs/`.

## The board

The board `delivery/commands/drive.md` opens every demo stop with, in the same order, so the two can never disagree:

- ✅ **Works now** — every accepted slice, one line each, as the thing the actor can do
- 🔧 **In progress** — the slice being worked, as the thing it will let the actor do, and the stage of
  `/drive`'s ladder it has reached, with the artifact that says so: an example map written, a plan without
  tasks, tasks half checked, a converged verdict waiting for its demo. Nothing in progress is one line
  saying so
- ⬜ **Still to come** — the remaining slices of the split, in order and by name, headed by one line of
  counts: `N of M slices accepted`
- ⚠️ **Not working yet** — every deliberate stub and every release constraint still in force, named as
  such, so a hole reads as "not yet" and not as a fault the actor has to find
- 🔀 **Ready (parallel)** — every slice that can start now: not done, every `depends_on` done, and not
  pre-empted by an open `CRITICAL` — in two groups, *claimed* (a `slice/<id>` branch on the forge, by whom
  and how long ago) and *unclaimed*, so another session knows which sibling is free
- ➡️ **Next (this session)** — the slices this `/drive` will take: every unclaimed ready slice whose
  contract is settled, concurrently; or the earliest ready slice in split order where the harness cannot
  delegate, unless the user picks another ready one
- ⛔ **Blocked** — remaining slices waiting on unmet `depends_on`, or an open `CRITICAL` ahead of them

The second line is the only one that differs from the demo stop's 🆕 *New in this demo*: a demo has something
new to show, and a question asked between demos has something half-built to report.

## Where it is read from

The board is derived from artifacts, not memory, the way `/drive` derives its entry stage:
the ordered split and its `## Slice graph`, the register at `specs/<feature>/slices/README.md` — a row
marks a slice accepted; a slice with `plan.md` under `slices/<id>/` and no row is in flight —
and the forge's `slice/<id>` branches (`git ls-remote --heads origin 'slice/*'`), which are the claims — read, never assumed: where that command fails the board says the claims could not be read, and shows no slice as unclaimed on the strength of a failed read. The slice in progress and its stage come from the ladder — the first
stage whose artifact is missing, empty, or still a placeholder — and its stubs from its `plan.md` and
`tasks.md`; the release constraints, if the project records any, from the slice's `plan.md`.

Before the split exists there is no board to draw. Say which stage of the ladder the work is at — principles,
specification, split — and what the next stage produces, and stop there.

## What it never does

- **Runs nothing.** No stage is entered, no task appended, no artifact edited: this is read-only, and safe to
  ask at any point on the ladder, including mid-implementation and in a session that has done nothing yet.
- **Counts slices, never tasks.** The task count is not on the board. Tasks stay in `tasks.md` for whoever
  is doing the work; a slice is a thing the actor can use, and fifty-eight tasks over two slices reads as
  fifty-eight features to somebody who did not write them.
- **Invents nothing.** A slice the split does not name is a question for `/story-splitting`, not a line on
  the board. A count an artifact cannot supply is written `unknown`, with the artifact that would have
  supplied it, never estimated.
- **Does not demo.** It hands over no command to paste and asks no question: that is the demo stop's job,
  and it belongs to `/drive`. Where the board shows a converged slice waiting for its demo, say so, and say
  that `/drive` is what runs it.

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
