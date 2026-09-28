# [PROJECT_NAME] — the product owner's brief

`/cruise` runs `/drive` with nobody at the wheel, and this page is the owner it decides for. The `drive-skipper`
delegate reads it before every product decision, after the specification and the constitution and before the
standing entries in `specs/<feature>/decisions.md`. Edit it at any time: the next decision reads the new text, and no run has
to stop for that. Leave a section as its placeholder and the skipper decides that ground from the
specification, the constitution and the decisions already taken — and says so in the entry.

This file is human-owned. `/cruise` reads it and never writes it; `slipwai migrate` never rewrites it.

## Who the actor is

[ONE_PARAGRAPH: who uses this product, in their own words for what they do — "a campaign organiser
recording a night's battles", not "the user". Where there are several, name each and say which one a slice
serves by default when the specification does not.]

## What the product is for

[ONE_PARAGRAPH: the outcome the actor gets that they could not get before, and the one thing that would
make this product pointless if it were wrong. The specification says what to build; this says why, which is
what a decision between two readings of it turns on.]

## Priorities and tie-breakers

[Ordered, most important first. These decide between options the specification leaves open:

1. [e.g. the simpler journey over the richer one]
2. [e.g. an actor sees their own data before anyone sees a report over everyone's]
3. [e.g. correctness of money paths over throughput — and `cycle=example` for those]]

## Taste

[What "good" looks like here that no criterion states: tone of the copy, density of a screen, what an
error message says, whether a list defaults to newest first. A decision that cannot cite this section or the
specification is a guess, and the entry says which.]

## Out of scope

[What this product deliberately does not do, so the completion audit does not open a slice for it: an
integration deferred, an actor not served yet, a report nobody asked for. Each line is a decision already
taken; the skipper cites it rather than re-taking it.]

## Always ask a person

[Questions that park a run however clearly the specification seems to answer them — a price, a legal
wording, anything that reaches a real customer, a release with no flag holding it back. The skipper records
these as `unavailable` and the run parks with the exact question; `.specify/cruise.stop` and `.specify/cruise.json` say how a run
stops and what it may decide.]

## What the record looks like

Every decision a run takes is appended to `specs/<feature>/decisions.md`, one entry in this shape, and the artifact the stage
owns is written in the same step. The session running `/cruise` allocates the number before a delegate
decides, so two decided at once never share one:

```markdown
## D<n> — <the question, in one line>
- **Stage:** <stage> · **Slice:** <id> · **When:** <ISO instant> · **Iteration:** <n>
- **Question:** <as the stage raised it>
- **Options:** <each, marking the one the stage recommended>
- **Decision:** <one>
- **Why:** <in the actor's terms>
- **Decided by:** host (stage recommendation) | host (standing decision D<m>) | drive-skipper (<model>) | drive-bosun | human
- **Confidence:** high | medium | low · **Would reverse if:** <the one condition>
- **Written to:** <the artifact paths the answer went into>
- **Status:** standing | overridden by D<m> | overridden by human <date>
```

Every demo the `drive-hand` delegate runs is appended to `specs/<feature>/slices/<id>/demo-log.md`, one entry in this shape, with its
evidence beside it:

```markdown
## <ISO instant> — <accepted | behaviour | implementation> · iteration <n> · drive-hand (<model>)
- **Started with:** <the literal command or URL> · **Seeded:** <what, or none>
- **Driven through:** agent-browser | <harness browser tool> | HTTP | CLI — <why, where not the first>
- **Examples:** <one line each — R1 e1: passed · R2 e1: failed, expected X, saw Y · R3 e2: unreachable, why>
- **Evidence:** <paths under demo/>
- **Feedback:** <what re-entered the ladder and at which stage, or the note for the next slice>
```

`make check-decisions` holds both files to those shapes — and every finished slice to a row in
`adversary-log.md` — and refuses an entry whose `Written to` or
`Evidence` path is not in the tree. Overrule a decision by changing its `Status` and writing the answer you
want into the artifact it names; the next iteration re-enters the ladder from that artifact. A decision that
would cost a migration to reverse is also written as an ADR at `Proposed` under `delivery/docs/adr/`, named in the
entry's `Written to`; accepting it is yours, and the run never does it.
