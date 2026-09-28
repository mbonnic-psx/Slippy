---
description: Ask the person what the tree cannot say — one question per row of the convergence map — and record each answer with its provenance
argument-hint: [axis ...]
---

# Ground

This repository adopted the delivery method around code that was already here. `delivery/docs/convergence.md` says where it stands
on each ladder a generated project sits at the top of. The survey placed every row it could from a file, and the
rows it could not are `unrecorded` — a question, not a default. This command is that question set, asked by you
of the person, one row at a time, with the answers written where the record keeps them. Two ways in, one question
set: run it on its own after `./delivery/init` and before the first slice, or let `/drive` run it — its Ground stage is this
command, for the axes the slice touches, and the ladder continues once the rows are placed. `$ARGUMENTS` names the
axes to ask about; empty means every row a person has not yet placed.

## The rules

- **One question at a time.** Ask, wait, write, confirm what was written, then the next. Never a questionnaire
  (`delivery/skills/find-gaps/SKILL.md` has the discipline).
- **Evidence and rungs first.** Before asking, show the row as it stands — the evidence the survey found and each
  rung's meaning — so the person places themselves on a ladder rather than answering a quiz.
- **Settle from the tree whatever the tree settles, and say you did.** A question you can answer by reading a
  file is not a question to put: read it, say what you found and where, and record it. Ask only what reading
  cannot reach — what a person intends, what they would ship on, what they know about how the work is done. A
  menu offered for something the tree already states asks somebody to guess at their own repository.
- **Ask how much to explain, first, and hold to the answer.** The opening question is not about this
  repository: it is whether they want each answer's consequence spelled out — what it writes, what moves,
  what it costs — or the short form. Somebody who knows this codebase and this method does not need to be
  told what webpack does, and being told anyway is how a question set becomes a wall to skim; somebody
  meeting either for the first time cannot answer safely without it. Offer both, say which you would pick
  for them and why, and take the answer as standing for the rest of the run unless they change it — which
  they may, at any question, in either direction.
- **In full, every answer on offer says what it does.** The person reads the options, not the paragraph above
  them, so each one names what it writes, what moves because of it, and what it costs. *`library`, and the
  gate holds its lint: `make verify` runs `npm run lint` here from now on, and a red one stops every change
  until it is fixed or quarantined* is an answer. *Yes — hold its lint* is a label, and a label is what gets
  picked by somebody who does not yet know what they are picking. Where two options differ only in a word,
  say what turns on that word. Never offer one whose consequence you have not stated: if you cannot say what
  an answer does, you are not ready to ask it.
- **In short, the labels stand alone — and two things never go.** The short form drops the elaboration, not
  the honesty: what you think and why, from what you read, stays (the next rule), and *I don't know* stays an
  answer on offer. What a short answer costs is a sentence away whenever they ask, and offering that once —
  *say the word and I will spell any of these out* — costs a line.
- **Say what you think, and why, from what you read.** A recommendation with its reasoning is what makes an
  answer a confirmation rather than a guess, and it is what lets somebody disagree with you on the evidence
  rather than on authority. Never a bare menu.
- **"I don't know" is one of the answers, every time, and offered as one.** It is the honest state of most
  rows in most repositories on the first pass, and a question that does not offer it is a question that
  manufactures an answer.
- **A rung is claimed only from a fact.** Here the person is the fact for what only they can know (how work is
  integrated, whether they would ship on the tests). Where the tree can contradict a rung — a release path, a
  quarantined suite, a template constitution, an application's role, no accepted ADR — the tree wins:
  `make check-convergence` fails the row, so say that instead of recording it.
- **"I don't know" stays `unrecorded`,** and is said so in the row's evidence. It is not filled in from context.
- **`detected` is the tree's reading, not the person's answer.** A row the survey placed from a file is asked
  about like an `unrecorded` one the first time; only `confirmed` and `overridden` mean a person has spoken.
- **A row a person already placed** (`confirmed`, `overridden`) is asked about again only if `$ARGUMENTS` names
  its axis; never moved silently.
- **Record, do not paraphrase.** `rung`, `provenance` `confirmed`, `evidence` in the person's own words, and
  `planned` where they name the slice that will move it. Where the answer is a fact under another key — the
  release path, an application's `kind`, a home, `why` — write it there with `confirmed` provenance; the row
  follows when `/survey` re-reads the record.

## The first question

Asked before anything about this repository, once, and answered for the rest of the run:

**Ask:** how much should each answer explain itself? *Long* spells out, for every option, what it writes to
the record, what moves because of it, and what it costs — which is what somebody meeting this repository or
this method for the first time needs in order to answer safely. *Short* gives you the reading, the
recommendation and the answers as labels, on the understanding that you already know what confirming an
application or placing a rung does. Say which you would pick for this person and why, from what you can see:
a repository whose record is entirely `unrecorded` and whose person has not met the method suggests long; a
second or third adoption by the same hands suggests short.

Then say, once: *say the word at any question and I will spell that one out* — and mean it, in either
direction, at any point.

**Write:** nothing. It is how you talk, not a fact about the repository, and it is not a row.

## The rows, as they stand

| Axis | Stands at | Provenance | Evidence |
|---|---|---|---|
| Path to production | `pipeline` | `detected` | pipeline: .github/workflows/package.yml; pipeline: .github/workflows/publish-package.yml; pipeline: .github/workflows/release.yml; pipeline: .github/workflows/verify.yml; scripted: assets/targets/aws/scripts/deploy.py; scripted: assets/targets/azure/scripts/deploy.py; scripted: tests/fixtures/adopt/converging/deploy.sh; scripted: tests/fixtures/adopt/javascript-gitlab/deploy.sh; scripted: Makefile |
| Integration | `unknown` | `unrecorded` | CI on github, gate .github/workflows/verify-delivery.yml |
| Safety net | `tests-exist` | `detected` | test recorded for slipwai |
| Structure | `named` | `detected` | slipwai: tool; not under apps/: . |
| Platform | `supported` | `detected` | in support on 2026-09-28: Python 3.11; no audit command recorded for slipwai |
| Constitution | `template` | `detected` | .specify/memory/constitution.md |
| Data | `recorded` | `detected` | schema: unmanaged |
| Infrastructure | `settled` | `detected` | home: here |
| Strategy | `recommended` | `detected` | why: Run /cruise on the factory itself to build Rust support and execution-model delegation; recommended: leave-it |

## The questions

### Path to production

**Ask:** How does a change reach production today — and is it the only way one can?

   The rungs, from the floor:

   - `unknown` — nobody has said how a change reaches production
   - `manual` — by hand: files copied, a console clicked, a package installed
   - `scripted` — somebody runs a script that deploys it
   - `pipeline` — a CI job deploys it, on some trigger
   - `one-path` — every change reaches production the same automated way, and no other
   - `pipeline-decides` — the pipeline decides releasability; artefacts immutable, rollback on demand, configuration shipped with the artefact *(a generated project sits here)*

**Write:** `release.path` in `project.json` (`pipeline`, `scripted` or `manual`) with `release.provenance` `confirmed`, and the row; `pipeline-decides` and `one-path` are claims about *every* change, so ask for the exception before recording either.

### Integration

**Ask:** How is work integrated: long-lived branches merged when a feature is done, trunk with short branches, or daily integration with CI on every commit and a build that stops on red?

   The rungs, from the floor:

   - `unknown` — nobody has said how work is integrated
   - `branches` — long-lived branches, merged when a feature is done
   - `trunk` — trunk-based: every branch lives less than a day
   - `continuous` — integrated at least daily, the build stops on red, CI runs on every commit on this forge *(a generated project sits here)*

**Write:** the row alone — nothing in the tree can say this, so the person's answer is the fact.

### Safety net

**Ask:** Do the recorded tests run green, and would you ship on them? Are they fast enough to run on every change, deterministic, and do they cover the seams a change would touch?

   The rungs, from the floor:

   - `none` — no test command is recorded for any application that was here
   - `tests-exist` — a test suite is recorded; whether it is green is not established
   - `tests-pass` — the suite is green in the gate, with no quarantine
   - `fast` — the suite is deterministic and fast enough to run on every change
   - `pinned` — the seams a slice touches are pinned by `/characterise` before they change
   - `mutation-measured` — the suite's strength is measured by mutation testing *(a generated project sits here)*

**Write:** the row; `tests-pass` and above are contradicted by a quarantined suite in the baseline, and the tree wins. A repository at `none` reaches `tests-exist` with its ecosystem's own runner and nothing else: the method never adds a mocking framework, and `/characterise` says what stands in at a seam instead.

### Structure

**Ask:** What is each buildable directory for — a service that runs somewhere, a library others import, a tool run by hand or in CI, a test suite of its own?

   The rungs, from the floor:

   - `as-found` — at least one buildable directory's role — service, library, tool, tests — is not established
   - `named` — every application is recorded as what it is
   - `laid-out` — every application lives under `apps/`, the layout a generated project has — or at a Go module's root, which is its import path and lives where it stands
   - `hexagonal` — every application declares the hexagonal layers and the import gate holds it to them
   - `typed` — every application has a type check the gate runs *(a generated project sits here)*

   The applications that were here:

   - `slipwai` at `.`: recorded as `tool` (overridden)

**Write:** `kind` on the application's record in `deployables`, with `provenance.kind` `confirmed`; the row moves when `/survey` re-reads the record.

### Platform

**Ask:** Where the tree pins no runtime version, which version does it actually run on where it is deployed — the gate's CI had to guess one and says so with *to confirm*? And of what `survey/structure.md` dates as out of support (*What it runs on*), is any of it not what actually runs, or under a vendor's paid support the table does not know? Read them each product's option from that page; each is a method slice for this row

   The rungs, from the floor:

   - `unknown` — nothing an application runs on could be read from the tree: no runtime pin, no framework version, no image
   - `inventoried` — what the applications run on is read and dated; something is out of support, or not in the support table
   - `supported` — every runtime, framework and image read is within support on the day it was dated
   - `audited` — every runtime, framework and image is in support, and every application records an `audit` command the gate can run *(a generated project sits here)*

   The applications that were here:

   - `slipwai` at `.`: runs on python 3.11 (detected)

**Write:** the runtime version in `toolchain.version` on the application's record with `provenance.toolchain` `confirmed` — it then stands through every `/survey`, which writes it into the gate's CI in place of the guess and dates it here — and a version the application cannot actually run on is a fact for `survey/running.md`, not a reason to leave the guess. A product the table does not know stays `unknown` in the evidence: say so, never fill it in. Support the table is wrong about is the row itself, `confirmed`, with the vendor's terms as `evidence`; the product's status in `platform` stays what the table says.

### Constitution

Not asked. The constitution is established by `/speckit-constitution` and held by `make check-constitution`; a person saying it is ratified does not make it so. Say where the row stands and move on.

### Data

**Ask:** Where is the database schema versioned: here, in another repository (which?), nowhere anybody can see (a DBA, a console), or is there no database?

   The rungs, from the floor:

   - `open` — where the schema is versioned is not recorded
   - `recorded` — the schema's home is recorded: `here`, `elsewhere`, `unmanaged` or `none`
   - `settled` — the schema is versioned in one place — here, in a named repository, or there is none *(a generated project sits here)*

**Write:** `database.schema` (`here`, `elsewhere` with `repository`, `unmanaged`, `none`) with `provenance` `confirmed`.

### Infrastructure

**Ask:** Where is the infrastructure this runs on described: here, in another repository (which?), nowhere (ClickOps), or is there none to describe?

   The rungs, from the floor:

   - `open` — where the infrastructure is described is not recorded
   - `recorded` — the infrastructure's home is recorded: `here`, `elsewhere`, `unmanaged` or `none`
   - `settled` — the infrastructure is described in one place — here, in a named repository, or there is none *(a generated project sits here)*

**Write:** `infrastructure.home` the same way, with `provenance` `confirmed`.

### Strategy

**Ask:** Two questions, asked apart. First: why is this work happening — what is the business trigger — in their words, not yours (a slice being built is not a trigger). Second, once `/survey` has re-made the recommendation from that: which strategy — `leave-it`, `in-place`, `modular-monolith`, `strangler-fig` or `rewrite` — with all five named, the recommendation read out as one of them with its reasons and what has to hold `before`, and *not yet* an answer. A strategy the map only *recommends* is this question, every time, never a default you decide for them

   The rungs, from the floor:

   - `open` — why this work is happening is not recorded
   - `why-recorded` — the business trigger is recorded; no strategy is recommended yet
   - `recommended` — a strategy is recommended from the trigger and this map — *leave it* included
   - `decided` — the strategy is decided and recorded as an ADR
   - `done` — the programme is finished: every retirement-ledger row reads *removed*, or the decision was to leave it *(a generated project sits here)*

**Write:** `why` in `project.json`, in their words. The strategy is an accepted ADR under `delivery/docs/adr/` with a `Strategy:` line, *leave it* included — and the word `Accepted` is the person's: you draft the ADR at `Proposed`, show it, and change its Status only after they have said, of that text, that they accept it, quoting them in the commit. Never write `Accepted` because the recommendation was not disputed, or because a slice needed the row moved.

## How each application starts

Not a row of the map, and asked before any slice changes code that was here: an application nobody has proved
starts is the floor beside the build, and `/drive`'s Pin stage refuses to change one. The first adoptions each
merged a slice that had stopped the application starting — constructor injection the container could not call —
and no suite saw it, because no suite builds the context.

**Ask:** For each application below — how is it started: the command, the port, what has to be seeded first, the
runtime it needs and the ones it cannot run on? What proves it answers — a URL, a command? Has anyone run it that
way since the method arrived, and did it start?

   The applications that were here:

   - `slipwai` at `.`: no `smoke` recorded — nobody has proved how it starts, and `/drive` refuses to change it until somebody has

**Write:** what was proven, with the date, in `delivery/survey/running.md` — the repository's own file,
which the `run-the-app` skill points to — including the run that failed and why. Then the one command that starts
the application and proves it answers, exiting non-zero when it does not, as `smoke` under the application's
`commands` in `project.json`, with `provenance.commands` `confirmed`: `make smoke` and the gate's smoke job run it
from then on, `verify` never does. An application nobody can start anywhere but production is a written `null`
with the reason in the same file — never a key left unwritten, which reads as a question still open.

## Then

1. `/survey` (`slipwai adopt --refresh`): the pages follow the record, and the strategy recommendation is
   re-made from what was said. Read the person the map's summary line and the recommendation
   `delivery/docs/change-strategy.md` now opens with.
2. `make verify` — `check-convergence` holds every row you wrote to the tree.
3. Commit as one change: what the person said, in the rows and the facts, and the pages that followed.
4. Say which rows are still `unrecorded`, and that `/drive` will ask about each before a slice that touches it.
