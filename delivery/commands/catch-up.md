---
description: After a migration, work through what the newer factory now asks of code it did not write
---

# Catch up

`slipwai migrate` merges what the factory generates today over what this project has become. What a merge
cannot do is change code the factory never wrote — so a gate that arrived with it can fail code that was
correct on the day it was written, and a rule that arrived with it can contradict a decision this project
already made and recorded. Neither is a mistake by anyone. Both are this command.

Run it after every migration, before the merge is pushed. A migration runs on `main`, on a clean tree — never on a
`slice/<id>` branch, where the host's files it rewrites fall into the slice's diff and `check-slice-scope` refuses them.

## Read what the migration left

`.slipwai/catch-up.md` is written by `slipwai migrate` before it returns — whether the merge committed or stopped at
conflicts — with one section per version this project just took, each with what that version asks of a
repository that already existed. Those sentences were written by the people who made the changes, in the
commits that made them, and this is the only place a project can read them — the factory's `CHANGELOG.md`
is not a file in this repository.

Where the migration could not tell which versions were crossed — a project scaffolded before factory 1.6.0
recorded no version, or both sides were snapshots of one release — the file says so under its own heading
and lists what it can. Read that sentence first: it says how to read the list under it.

If the file is not there, that is not a sign that nothing is owed. Either the migration's report said it
could not be written and why, or the `slipwai` that ran was older than 1.9.0, the version that started
writing it, or no migration has run in this tree. Say which of those it is to the user before anything
else, and read the factory's `CHANGELOG.md` for the versions crossed if you can reach one. Never conclude
from an absent file that the versions crossed asked nothing.

## Then work it

1. Read every **Owes** line. Some ask for something no gate can check — an answer this project has to give,
   a tool to re-run, a step in the account it deploys to. Those are yours to do and nothing will remind you.
2. `make verify`. Take the target it stops at, fix that, run it again, and repeat — one gate at a time,
   because a second failure is often the first one's consequence and disappears with it.
3. Delete `.slipwai/catch-up.md` when the work is done. It is git-ignored and disposable; the factory's changelog is the
   record that lasts.

## Work each failure back to the rule that caused it

A failing gate after a migration is not a bug report; it is a rule arriving. So for each one, say which of
these it is before changing anything, because the right response differs:

- **The rule is right and this project has not done it yet.** The ordinary case. Adopt it: change the code,
  not the gate. `.` and its siblings are yours to change; the gate script is the factory's.
- **The rule is right and this project already decided the opposite, on purpose.** The decision is written
  down somewhere — a plan, a `tasks.md` line, an ADR — and the migration did not know about it. This is the
  one case that is not yours to settle alone: put the two side by side, the rule and the recorded decision,
  and ask the user which stands. Then write the answer where the next migration will find it.
- **The rule does not fit this project.** Possible, and worth saying out loud rather than working around.
  Say so to the user with the reason, and raise it against the factory. Do not weaken the gate to pass.

## Use the code index, if this project has one

Every question in the step above is the same shape — *where else does this pattern appear, and what would
change if I fixed it* — which is a blast-radius question, and grep is a poor instrument for one. If
`AGENTS.md` carries an extension block for a code index (CodeGraph, `.codegraph/`, `codegraph_explore` over
MCP or the `codegraph` CLI), that is what it is for: ask it for every caller of the symbol a gate named,
every reference to the identifier the rule is about, and what depends on the file you are about to change.
Ask it *before* grepping and before opening files one at a time, and answer from it in this session rather
than assuming another session has or lacks the same route. A delegate checks its own MCP, CLI and `npx`
routes and names which one answered.

A project with no such block has no index and grep is the tool; say which one you used, so a review can
tell a search that was exhaustive from one that was a guess.

## Rules

- **Never edit a gate to make it pass.** A gate script came from the factory, the next migration will bring
  it back, and a locally softened copy is a conflict later plus a false green in between. If it is wrong,
  it is wrong for every project the factory made, and it is a change to make there.
- **One rule adopted, one commit.** The migration is already its own commit (two, where the harness
  projections were re-derived). Keeping each adoption separate is what makes a bisect readable afterwards
  and what lets one of them be reverted without the rest.
- **A decision that changes is a decision rewritten.** Where the second case above sends the user's answer
  against something already recorded, edit that record rather than leaving the two to disagree — the next
  person to read it is the next migration's problem otherwise.
- **Green before pushed.** `make verify` passes before the migration and everything after it leaves this
  machine.
