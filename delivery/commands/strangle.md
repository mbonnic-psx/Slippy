---
description: Move one capability out of the code that was here into a new home, behind a routing seam, and record it in the retirement ledger
argument-hint: <the capability to move, and where its requests enter>
---

# Strangle

Read `delivery/docs/change-strategy.md` first — the three strategies, the order to change in, and why dual-write is a
trap — and `delivery/skills/finding-seams/SKILL.md`. This command is one turn of the strangler fig: one capability, one
seam, one row in `delivery/retirement.md`. It is the Slice stage of adopting the method around code that existed before it.

## Refuse what is not ready

- If `$ARGUMENTS` names no capability, or names the whole system, stop and ask which capability, for which
  actor, and where its requests enter (an HTTP route, a message, a scheduled job, a file drop).
- If `delivery/survey/pinned.md` has no row for the behaviour this capability carries, stop: run `/characterise` for it first.
  A capability moved without its behaviour pinned is a rewrite with no way to tell a change from a regression.
- If `project.json`'s `why` says the trigger is a runtime, a framework or a packaging that has to go, say so:
  those are lower rungs on `delivery/docs/change-strategy.md`'s ladder and cheaper with a green gate; the strangler is
  the architecture rung and comes after them unless the capability itself is the trigger.
- **No decision, no strangling.** `project.json`'s `strategy.decided` must read `strangler-fig`. Today it
  reads `null` — recommended `leave-it`, decided nothing. A recommendation is not a decision, and neither is this command's argument:
  where `decided` is `null` or names another strategy, stop, say what is recorded, and point at the way to
  decide — an accepted ADR under `delivery/docs/adr/` with a `Strategy: strangler-fig` line, then `/survey`
  (`delivery/docs/change-strategy.md`, *Recommended for this repository*). Going beyond the recorded decision is how a
  programme quietly becomes a second system beside the first.

## Decide the seam

Read `delivery/survey/structure.md` first: its entry points are where a capability's requests enter, its most-depended-on files are
what a cut there drags along — the last to move and the first to pin — and its hotspots are where the next change
lands anyway. A seam at an entry point with little of the shared core behind it is the cheap one; say which
lines of the view chose it.

This project deploys to infrastructure the factory does not manage, so the routing seam is one the repository already has or gains for the purpose: a reverse-proxy or router rule, a feature toggle the legacy system already reads, a message subscription moved from one consumer to another. Name it, say how it is switched and switched back, and write that in `delivery/docs/deployment.md` beside the release path.

The seam is decided and written in the slice's plan before the code exists, with how it is switched back —
the same release-constraint decision `/drive` asks for, made concrete.

## Decide the data

`project.json`'s `database.schema` says where the schema lives; `delivery/docs/deployment.md` says who else reads the
tables. One writer per fact: the capability's data reaches its new home by change data capture from the legacy
store or by an outbox written in the legacy system's own transaction — never by writing the same fact in two
places. Where the new home is event-sourced, its history starts at a genesis event that records what the
legacy system handed over, and the legacy system is an external system in the model. Rehearse the move against
a copy with a reconciliation report; dual-run in shadow before an actor is served; roll back at least once in
rehearsal.

## Decide the new home

- **A generated service beside the code**: `/add-service <name> --language <language> --purpose "…"` — the factory scaffolds a service with the gate, the hexagonal layout and the axes, and the capability moves into it. Languages it can generate: `typescript`, `python`, `go`, `rust`, `java`.
- The same language as the code that is here, for `slipwai`, so the team keeps one language while the architecture changes.
- **A module inside the existing code**, where the strategy is the modular monolith in place: a context directory with its own `public` module, held to the import gate by declaring `"layout": "hexagonal"` on the application's record in `project.json`.
- Lifting the *existing* code into a generated service (`add-service --from <path>`) is not built; when it is, it is the extraction step here. Until then the capability is rewritten into its new home behind the seam, which is what the pinned tests exist to make safe.

## Do the slice

1. Write the row in `delivery/retirement.md` first, status *routed*: capability, from, to, routed by, pinned by (the
   `delivery/survey/pinned.md` rows), today's date. A slice that is not in the ledger did not happen.
2. `/drive` the slice as any other: the pinned characterisation tests are the acceptance tests of the move
   until the plan says which of them change, and why.
3. The slice ends **dark**: the new home deployed, the seam pointing at the old path, `make verify` green. A
   new row, status *moved*, when the seam serves the new home to real actors; *retired* when the old path
   serves none of it; *removed* when the old code is deleted — and that deletion is a slice too.

## Then

Say in the handover which rung of the ladder this was, what the seam is and how it is switched back, and which
row the ledger now ends on. The programme is finished when every row reads *removed*.
