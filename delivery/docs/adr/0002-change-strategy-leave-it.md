# 0002. Change strategy: leave the architecture where it is

Date: 2026-09-28

## Status

Proposed

Strategy: leave-it

## Context

This repository adopted the delivery method around the factory's own code. `delivery/docs/change-strategy.md`
recommends a strategy from the recorded trigger and the convergence map. The trigger, recorded in
`project.json` as `why`, is to run `/cruise` on the factory itself to build Rust support in `adopt` (issue #10)
and execution-model delegation (issue #9). It names no platform out of support, no delivery problem, no change
problem, no capability the current structure cannot carry and no host to move off, so no strategy other than
leaving the architecture where it is follows from it.

The work the trigger names is additive inside the one existing application: a new recognisable ecosystem in the
survey, and new delegation behaviour in the agent scripts. Neither needs a new service, a strangler, or a
rewrite.

## Decision

We will leave the architecture where it is. Slices land in the existing `slipwai` package under `src/slipwai/`
and its bundled assets, held to the factory's own `make verify`, and the delivery rungs of the convergence map
(safety net, platform audit, constitution) are taken as the programme offers them, one per slice.

This ADR is drafted by `/cruise` at `Proposed`. Only a person accepts it.

## Consequences

The Strategy row can move to `decided` once a person accepts this ADR, and to `done` with it, since *leave it*
finishes the axis. `/strangle` stays unused: no capability moves, and `delivery/retirement.md` is not written.

A later trigger that names a structural problem — a capability the package cannot carry, a platform out of
support — is a reason to supersede this ADR with a new one, not to edit it.
