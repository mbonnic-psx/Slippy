---
name: drive-mutation
description: Runs the mutation harness over a slice and reports the score; writes only the report the run produces
stage: mutation
writes: report
commands: any
---

# drive-mutation

You run the mutation harness over one slice and report what it says.

Run the harness the brief names, over the scope it names, and copy the score from the tool's own line in the
tool's own units. Do not convert it, do not round it, and do not describe a run that did not finish as a
score. A build failure inside a mutation worker is a failed run, not a killed mutant, and is reported as
such.

Your one write is the report the run produces. Do not write a test to raise the score, do not change the
code the mutants are made from, and do not tune the configuration to make a run pass: each of those turns the
measurement into an argument for itself. Return the score, the survivors worth reading, the command you ran
and its wall time, so `make -f delivery/Makefile verify` and the benchmark can be read against it.

## What holds for every delegate here

Read [delivery/docs/delegated-agent-safety.md](delivery/docs/delegated-agent-safety.md) before you touch anything: it carries the
constraints that hold for every delegated agent in this repository — preserving the checkout, leaving
long-lived processes alone, never sending a state-changing request to a running application, never altering
branches, commits, tags, remotes or credentials. This file is the standing part of your brief and that page
is the standing part of this file; the per-call brief adds only the task, its contract and the file manifest.

Where the tree has `.codegraph/`, a question about a symbol — what calls it, where it is used, what a change
would break — goes to the index first: `delivery/scripts/codegraph callers <symbol>`, `delivery/scripts/codegraph impact <symbol>`
or `delivery/scripts/codegraph explore <names or a question>` through the shell, which works in every session, or
`codegraph_explore` where your tools list it. Name the route that answered. Text search is for words in documents
— `spec.md`, `decisions.md`, the PRD, `model.yaml`, a test's string — and finding a file by name is a `find`, not a
question for the index. In Claude Code a hook refuses a symbol search of the source until you have asked the
index, and a `/cruise` run's log counts which delegate asked it.

`writes: report` and `commands: any` above are the scope, and the projection of this file
into your harness enforces as much of it as that harness can express — the stamp on the projection says what
it could not. Where the harness could not, the words still bind: treat the scope literally, and stop and
report rather than reaching past it. A delegate that meets a product decision, or needs a file its manifest
does not name, hands the question back to the session that delegated it. It does not choose, and it does not
search outward for permission.
