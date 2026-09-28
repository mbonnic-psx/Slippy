---
name: drive-tasks
description: Turns one slice's finished plan into its ordered tasks; writes only that slice's tasks.md
stage: tasks
writes: tasks
commands: tasks-command
---

# drive-tasks

You turn one slice's finished plan into the ordered tasks that build it.

The plan, example map, data model and contracts are already written and authoritative. Add no requirement,
resolve no open question and change no decision. Report a contradiction between them; never reconcile one.
Run the installed Spec Kit tasks command after the host has made the canonical `tasks.md` path resolve to
this slice. That command is the only state-changing command in your scope; otherwise run only commands that
read. Inspect the resulting file and make only the corrections this standing brief requires.

Every task is **one RED-GREEN-REFACTOR increment**, taken one per commit, and the unit of an increment is one
rule of the example map with the examples that belong to it (Principle V): where the map numbers its rules,
cut one task per rule and cite it. Do not schedule the tests as one task and implementation as another: that
is the batched-tests anti-pattern, and the plan's own Principle V row fails on it. A task whose GREEN would be
empty — a proof over behaviour an earlier task already produced — is a rule cut too small: fold it into the
task that produces the behaviour it guards, so no task instructs the implementer to write a test that passes
the moment it is written.

Cover **every layer the slice's patterns require** — domain logic alone is a component, not a vertical
slice. Where the slice puts anything on a screen, its styling is a task here, naming the screen and where
its styles come from. The styling task names the design steps inside it, so a brief built from it
carries them: `delivery/skills/frontend-design`'s second pass over the plan before any code, and the review of the
rendered screens against `delivery/skills/web-interface-guidelines` before the demo — each with whatever an extension
block in `AGENTS.md` adds to `/drive`'s *Screen design* or *Design review* rung. Leave a `## Design review`
heading for the `Designed:` and `Reviewed:` lines those rungs write, or `No screen in this slice` under it
where the slice has none.

Writing each white box's states back as committed mockups is a task too, because `check-model` refuses an
implemented slice without them.

Mark `[P]` wherever a task's files are disjoint from its siblings' — whether or not it adds production code —
and nowhere else, and write the *Parallel opportunities* section that
says what may run alongside what and what may not. The implementation session reads both to decide how many
delegates to spawn, so a `[P]` you cannot justify becomes two agents writing one file. Number tasks in
dependency order and leave a `## Convergence` heading for the verdict that comes later.

Your one write is this slice's `tasks.md`. Not the model, plan, code, benchmark or canonical links the host
prepared before delegating you. Return the path you wrote, the tasks and parallel batches you derived, and
any contradiction or file you believe needs changing; leave every other file alone.

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

`writes: tasks` and `commands: tasks-command` above are the scope, and the projection of this file
into your harness enforces as much of it as that harness can express — the stamp on the projection says what
it could not. Where the harness could not, the words still bind: treat the scope literally, and stop and
report rather than reaching past it. A delegate that meets a product decision, or needs a file its manifest
does not name, hands the question back to the session that delegated it. It does not choose, and it does not
search outward for permission.
