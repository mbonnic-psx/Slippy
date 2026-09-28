# Getting started

`slipwai` was scaffolded with the `standard` foundation,  backend services, and the `none` frontend.
Bootstrap Spec Kit interactively
with `./delivery/init`, or select an
integration explicitly, for example `./delivery/init --integration claude` or
`./delivery/init --integration cursor-agent`. All additional arguments pass through to
`specify init --here --force`.

Start with the smallest actor-visible product slice in the code that already exists — the factory made none of it, and `project.json` records where it is. Run `make verify`
before and after each increment; it is the same gate CI runs.

Use `/drive` for the complete delivery loop and `/where-are-we` for the progress board at any point; both read
artifacts on disk rather than memory, so one resumes safely and the other changes nothing. See `delivery/commands/`, `delivery/skills/` and `delivery/agents/`.
`/cruise` runs the loop with nobody at the wheel — it decides product questions as the owner and runs each demo
as the actor, and stops only for a person; it ships switched off, and `delivery/commands/cruise.md` says how to start it.

When ratifying a constitution, do not paste an Event Modeling or event-sourcing mandate into this
standard profile; `make check-speckit` checks that boundary. `make check-constitution` checks the
other direction — that minimum CD and the practices the skills teach are still in there.
