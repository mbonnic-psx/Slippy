---
description: Draw the benchmark overview — what each slice cost and how each stage did — as a page beside the records
argument-hint: [feature]
---

# Benchmark

Every stage `/drive` runs is recorded in `specs/<feature>/slices/<id>/benchmark.json` (the stages above the slice
loop in `specs/<feature>/benchmark.json`). This command turns the records into the page a person reads, and reads
it back to them.

## Draw the page

```sh
python3 delivery/scripts/agents/benchmark.py overview $ARGUMENTS
```

With a feature named, that feature's page; with none, one page per feature that has a record. Each is
`specs/<feature>/benchmark.md`: the slices side by side — wall, tokens in and out, the models that ran, converge
passes, tasks appended, gaps before and after converge, mutation score, adversary findings, demo outcome, verify
failures, rework, sessions, tasks, files, lines — then every stage of every slice with the type and model that
ran it and what it reported, the notes (entries
still open, a stage re-entered after implementation, tokens the script could not read and why), and how to read
the numbers. The page is regenerated whole; never edit it, and never edit a record to change what it says. No
record yet is the script's own line, and the answer: nothing has been driven since the records began.

## Read it back

Show the slices table as the page has it, then say what it shows, in this order and only where the page
supports it:

- **Where the cost sits** — the stage or two that took most of the tokens and time, per slice, and whether that
  is the same stage every time.
- **What moved** — between one slice and the next, and across any change to a prompt, a skill, the layout or
  `.specify/models.json` the history records: cheaper or dearer, more or fewer converge passes, gaps found after
  converge, the mutation score. Host context grows through a session, so do not compare host tokens directly
  between slices spanning different numbers or lengths of sessions.
- **Whether the split paid** — where a stage ran on the `fast` role (`/model-delegation-settings` shows which), what it cost against
  the converge passes, verify failures and mutation score of the slices it ran in. A conclusion needs more than
  one slice; with one, say so.
- **What is unknown** — every `unknown` on the page, with the reason the record gives, and the harness or setting
  that would make it known. An unknown is never estimated.

Tokens are not prices, and the page compares this project on this harness with itself — say neither more nor
less. Then commit the page: `git add specs/<feature>/benchmark.md`, on its own or with the slice it follows.
`make -f delivery/Makefile benchmark` prints the same table in the terminal without writing anything.
