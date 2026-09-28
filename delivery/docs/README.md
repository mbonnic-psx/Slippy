# Documentation

Start with [getting started](getting-started.md), then use [the workflow](workflow.md). Every page under `delivery/docs/` is listed here.

## Start here

- [`getting-started.md`](getting-started.md) — bootstrap Spec Kit, run the gate, and start the first slice
- [`workflow.md`](workflow.md) — the delivery loop each slice travels, drawn from the constitution to the hardening passes

## This project

- [`whats-included.md`](whats-included.md) — everything that was generated, in one list
- [`architecture.md`](architecture.md) — the hexagonal boundary, the services, and the bounded contexts they hold
- [`gates.md`](gates.md) — what `make verify` runs, and the gates beyond it
- [`skills-and-commands.md`](skills-and-commands.md) — the skill catalogue and the commands adapted to this project
- [`agent-harnesses.md`](agent-harnesses.md) — how `delivery/skills/`, `delivery/commands/` and `delivery/agents/` are projected into each coding agent
- [`delegated-agent-safety.md`](delegated-agent-safety.md) — the standing safety boundary every delegated brief references
- [`speckit-preset.md`](speckit-preset.md) — how this project's templates are installed into Spec Kit without editing it
- [`evolving-the-project.md`](evolving-the-project.md) — this repository owns every file: change anything, and merge a newer factory's output when offered

## Production

- [`deployment.md`](deployment.md) — what runs in AWS for this project's services and how a commit gets there, drawn from its own answers

## Decisions

- [`adr/0001-record-architecture-decisions.md`](adr/0001-record-architecture-decisions.md) — the decision to record decisions, and the ADR shape

## Adopted here

- [`adoption.md`](adoption.md) — how the method was installed around the code that was here, what that forfeits, and where each fact came from
- [`convergence.md`](convergence.md) — where this repository stands on every ladder a generated project sits at the top of, and what is planned to move it
- [`change-strategy.md`](change-strategy.md) — strangler fig, modular monolith in place or rewrite — and when to leave it
