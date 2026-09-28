---
description: Drive one slice through planning, implementation, and an actor-visible demo
argument-hint: [slice-id-or-feature]
---

# Drive

Deliver one small vertical slice under `AGENTS.md`. Once the ladder below has produced it, resolve
the requested feature or the active directory recorded in `.specify/feature.json`.

## Enter at the first incomplete stage

Read artifacts from disk rather than conversation memory and walk this ladder from the top. The entry stage
is the first one whose artifact is missing, empty, or still a placeholder — **including the stages upstream
of the slice loop**. State the entry stage and the evidence that selected it before changing anything, then
run that stage and every stage after it. Never rerun a completed stage merely to check. Where `.codegraph/` is in
the tree, a caller or blast-radius question is one index call — `delivery/scripts/codegraph callers <symbol>`, or
`codegraph_explore` — and not a text search; grep is for words in documents.

**The checkout goes stale the way conversation memory does, so check the branch before the artifacts.**
Every signal the ladder reads — a slice's `status`, whether `examples.md` or `tasks.md` exists, the slice
graph — is a property of this commit, and a branch behind trunk reads exactly like a project where the work
was never done: a `/drive` fifty-seven commits behind wrote a second example map for a slice that had
shipped. So fetch and compare first — `git fetch`, then `git log --oneline HEAD..@{u}`, or against
`origin/main` where the branch has no upstream. Behind by anything, stop and say so rather than deriving:
the artifacts about to be read are not the project's current ones. The evidence line names the branch, its
head and its distance from trunk in the same breath as the stage. Where the fetch could not run — no remote,
or a remote this environment cannot reach — the line says *could not verify this checkout is current*, and
that never reads as *current*.

1. **Ground** — this repository adopted the method around code that was already here, so before any
   principle is ratified the map says where it stands: `delivery/docs/convergence.md` exists and `make check-convergence` is
   green (`adopt` wrote both; `/survey` redraws the page from `project.json`). No map, no Principles. Read
   the rows and name the ones this slice touches — a slice through the repository root
   touches at least *Safety net*, *Structure* and *Strategy* — adding to or changing what was here is
   what a strategy is about, and a strategy the map only *recommends* is a question, never a default; a slice
   that changes how a change reaches production touches *Path to production*. A row whose provenance is
   `unrecorded` — or `detected`, the tree's reading and nobody's answer — on an axis the slice touches is a
   question for the person **before anything else**: the release path first of all, since a slice with no
   known path to production cannot be called releasable. **Asking is this stage's work, not a stop:** run
   `/ground` here, inside `/drive`, for the axes the slice touches — or with no argument, the first time, for
   every row a person has not placed — one row at a time, the evidence and the rungs shown first, each answer
   written where the record keeps it (`release`, `ci`, a deployable's `kind`, `why`, or the `convergence` row
   itself, provenance `confirmed`), then `/survey` so the record is followed. The stage is done when the rows
   the slice touches are `confirmed` or `overridden`, or the person has said they cannot place them; a row
   nobody can answer today stays as it is and is said so in the slice's specification — never filled in from context
   — and an ADR's `Accepted` is the person's word, never yours. Then continue down the ladder in the same run.
2. **Principles** — `.specify/memory/constitution.md` is ratified rather than absent, unfilled, or
   still the template `./delivery/init` installed, and `make check-constitution` passes. A passing gate alone is not
   this stage done: the gate lets the untouched template through so the first push can deploy, and says so
   (`nothing drafted yet`). Otherwise run `/speckit-constitution`, then `/constitution-coverage` for
   whatever the gate still reports missing.
3. **Product specification** — `specs/<feature>/spec.md` describes the product this slice belongs to.
   Otherwise run `/speckit-specify`, then `/gaps` over what it promises.
4. **Split** — the work is ordered vertical slices rather than one undivided outcome. Otherwise run
   `/story-splitting`.
5. **Slice gaps** — the slice's acceptance criteria in `specs/<feature>/spec.md`
   records a gaps review for this slice: the criteria and states it added, or a `Gaps reviewed` note saying
   what was checked. Otherwise run `/gaps` over it. A missing state is a paper edit here and a rewritten
   test later.
6. **Release constraint** — this project deploys to infrastructure the factory does not manage;
   `delivery/docs/deployment.md` records where, and how a release reaches it. Before the plan, decide and write down
   how this slice reaches production and what gates its release: releasable on merge because everything it
   adds is coherent and safe to an actor the moment it lands; held behind a toggle this repository already
   has, named; or a coordinated deploy, with who does what. Recommend the answer with its reason rather than
   asking an open question. If `delivery/docs/deployment.md` cannot yet say how a release reaches production, that is
   not a gap to work around — it is the first slice.
7. **Plan and tasks** — `specs/<feature>/slices/<id>/plan.md` and `tasks.md` exist. Otherwise run the
   installed Spec Kit plan and tasks commands — after making the canonical paths they resolve to into links.
   Those commands write `specs/<feature>/plan.md`, `research.md`, `data-model.md`, `quickstart.md` and
   `tasks.md`, one slot per feature, so before running them: `mkdir -p specs/<feature>/slices/<id>` and, for
   each of the five, `ln -sfn slices/<id>/<name> specs/<feature>/<name>`. The commands then write through the
   links, the record lives under `slices/<id>/` from the day it is planned, and every later stage reads it
   there. After each command, `ls -l specs/<feature>/`: a regular file where a link was is a harness that
   replaced the link, and the file is moved under `slices/<id>/` and the link remade before anything else.
   The links are ignored by git and never committed. The plan's *Structure Decision* names the service the
   slice's code lives in;
   with more than one service in `project.json`'s `deployables`, that is a choice made against each
   service's recorded `purpose`, never the first service by default — and a slice no purpose covers is a
   product decision to ask. What `research.md` states about a dependency's behaviour — a default, a limit, a
   version's requirement — cites the artefact it was read from: the library's documentation at the pinned
   version, its source, a run against it. A statement with no citation reads *assumed*, and a plan does not
   rest on it. It also names the bounded context inside that service, and this is where contexts are
   found in a project without an event model: read the specification's vocabulary the way
   `delivery/skills/domain-driven-design/resources/bounded-contexts.md` describes under *The Language Test* — the
   same word meaning two things, qualifiers creeping in ("billing customer", "shipping customer"), rules
   that change for different reasons. Two vocabularies are two bounded contexts: record them on the
   service, with the user, as `slipwai describe-service <name> --context <context>` (once per context), and
   put each context's code under its own `src/<context>/` behind a `public` module — `make check-imports` keeps them apart from then on. One
   vocabulary is one context, and saying so is the whole decision. Neither is a reason for a new service;
   `delivery/docs/architecture.md`, *Bounded contexts*, says what is. In this repository the decided strategy governs the home too. Under an accepted `strangler-fig`,
   a product slice's *Structure Decision* names a deployable that is **not** one that was here (under
   the repository root) — a service `add-service` made beside them, current from day one,
   whose first capability moves through `/strangle` and writes `delivery/retirement.md` — or it quotes the owner's written
   exception for this slice, in their words. "Confirmed at plan time" by the plan alone is not a decision:
   a strangler that lands every slice in the old home is the old home with a new label, and
   `check-convergence` says so while the ledger stays empty.
8. **Pin** — the slice changes code that existed before the method did (under
   the repository root) only once the current behaviour at the seam it changes is
   recorded: `delivery/survey/pinned.md` carries a row for each behaviour `plan.md` says this slice changes, with the tests
   that pin it and the command that runs them. Otherwise run `/characterise <behaviour>` for each, one at a
   time — it refuses "everything", and so does this stage. Code the factory generated needs no pin: its
   tests are the pin. A slice that touches no code that was here passes this stage by saying so, and a slice
   that reaches a seam nothing can observe deterministically records the smallest seam it introduced in the
   same ledger. What stands in at the seam is a fake written in the test tree, never a mocking framework this
   stage would have to add — `/characterise` says why. This is the *pinned* rung of the safety-net axis,
   held per slice rather than claimed once. **And before any pin, the application has to start.** Where
   `delivery/survey/running.md` still reads *Not yet proven* for the application the slice changes, or its record in
   `project.json` has no `smoke` command, this stage refuses the slice and says so: proving the run path and
   recording `smoke` is the programme's step right after the build, and it goes first — the one exception is
   the slice that proves it. A suite that never builds the context cannot see a constructor the container
   cannot call, and slices have shipped that way, converged and green, with an application that no longer
   started.
9. **Implementation** — tasks remain unchecked. Run the installed Spec Kit implement command. In this repository new code beside what was here (under
   the repository root) is tested the way `AGENTS.md`, *Delivery method*, says: what
   stands in at a seam is a fake written in the test tree, never a mocking framework added for the purpose —
   Mockito, Moq, gomock, `unittest.mock` are the same last resort in every language, and "what should I add?"
   is never answered with one. The runner is the one the application records; where that is out of support
   (JUnit 3 or 4, nose, a runner nobody maintains) new tests use the ecosystem's *current* framework and keep
   the old tests running beside them — JUnit 5 through its vintage engine, pytest running `unittest` as it is
   — as a slice on the map's Platform row, never the next-oldest version chosen because the tree's vintage
   makes it usual.
10. **Convergence** — `tasks.md` records a converged verdict for the current commit and has no
   unchecked convergence task. Otherwise run the installed Spec Kit converge command, implement whatever it
   appends, and repeat — until it reports converged, or until the loop reaches its bound, whichever is first.
   Record that verdict under a `## Convergence` heading so this stage is not re-run, then `/gaps` over the
   slice diff. The verdict names each constitution principle the diff touches — a MUST about money, time,
   identity, a boundary — with the file and line that satisfies it; "no constitution obligation unmet" as one
   sentence is not a verdict, and a slice has shipped a float in a monetary column under exactly that
   sentence. Converge is append-only — its one write is new tasks — so repeating it is safe. A Spec Kit
   install with no converge command is a skip with a stated reason, not a stop.

   **The loop has a bound, because its exit condition is the judgement of the thing being looped.** Every
   other rung ends on something an outside reader can evaluate — a file exists, tasks are ticked, a gate is
   green. This one ends on the opinion of a fresh strong model asked to find what is missing, and asked that,
   it will find something: at any level there is a level further out. So: at most two passes by default —
   one to find the work, one to confirm it closed. A pass beyond that is a decision this session takes and
   says why, never what this wording produces — with one exception the bound does not hold against: **an
   open `CRITICAL` finding re-opens the loop however many passes have run**, because a slice does not go to
   its demo carrying one. When the bound is reached, whatever is still open and not `CRITICAL` is appended
   as Phase 4 tasks, the verdict says the loop stopped at its bound, and the slice goes to its demo: the
   actor's feedback is better evidence about whether the slice is right than a third reading of the same
   diff. Only a `CRITICAL` or `HIGH` finding re-opens the loop at all, and only a `CRITICAL` re-opens it past
   the bound; grade every appended task, and a `MEDIUM` or `LOW` rides along with the next pass or lands in
   Phase 4 rather than costing a pass-and-fix cycle of its own. Hand the **first** pass the level list and ask it to account for each —
   domain, use case, delivery adapter, screen, published contract — because four passes on one slice each looked exactly one level further out than the
   last, and asked for all of them at once they are one pass. Give each pass a stated budget in its brief and
   take what it has found when it reaches it: an incomplete verdict with three findings is worth more than a
   complete one nobody waited for.

   **After every pass — including one that was stopped — the tree is clean before anything else runs.**
   Converge proves a finding by mutating the code and restoring it, and a pass stopped mid-mutation leaves
   the mutation in place with nothing announcing it; the next thing on this ladder is the demo, which would
   show the actor the mutation. `git status` is two seconds against that. And **a finding from a pass that
   did not finish is a lead, not a finding**: a stop notification's last line is a fragment of work in
   progress, and what was in progress was checking. Re-run its reproduction before it is written into
   `tasks.md`, a waiver or anything outside this session — one mutation, one test run, one restore — and
   until then write it as *a stopped pass believed X; verify before acting*. One such line became a HIGH
   task, a waiver and a published finding within the hour, and the mutant died when somebody re-ran it.

   **Each appended task closes the class, not the instance it was found at.** Where a finding sits on a
   repeated surface — a field in a parser, a row in a route table, one screen of a pair, one array cap in a
   decoder, one column of a field table — the task's GREEN names the sweep rather than the example ("every
   field this parser validates", "both screens of the crossing", "every array this decoder bounds"), and the
   verdict records the sweep that was performed and what it found. A pass that validates one field and leaves
   its neighbour is a pass the next one repeats: six of them closing a sibling each is the same work as one
   closing the surface, at six times the price, and it is what writing the task as the example produces.
   Where the sweep is genuinely larger than the slice, say so in the verdict and leave a task naming the
   rest — that is a scope decision recorded, not a sibling found again next pass. A slice that touched how an application starts — constructors, dependency injection,
   configuration, module registration, the build — has converged only once that application has started with
   the change in place: `make smoke` where a `smoke` command is recorded, the command `delivery/survey/running.md`
   holds otherwise, and a converged verdict that rests on the suite alone is not one. A runtime or a
   framework that moved a major version moves on the Platform row, as a slice of its own and never inside
   another — `/survey` reports a version that moved without the row planning it as a disagreement, and the
   slice that moved it is not converged until the row, or an ADR, owns the move. Then, because this repository is converging on what a generated one has, hold
   the map to what the slice did: run `make check-convergence` and read `delivery/docs/convergence.md` against the diff. A rung this
   slice reached — a quarantined suite now green, a role now recorded, a release script now run by CI on
   every commit — flips its row: edit that row in `project.json`'s `convergence` (`rung`, `evidence` naming
   what established it, `provenance` `confirmed`, `planned` cleared), run `/survey` so the page follows, and
   `make ratchet-tighten` where fewer findings remain than the baseline records. A rung the slice did not
   reach stays where it is; the map is never moved to match a hope, and `make verify` fails a row the tree
   contradicts. Then offer the next slice. **An open `CRITICAL` in `specs/<feature>/adversary-log.md` comes
   first**, ahead of every product and method slice, until it is fixed or a person has deferred it in that row
   with their name and reason: one adoption pinned a defect that leaked one customer's account to another as a
   failing-if-fixed test and then spent a slice on an icon. Then the next method slice,
   from the programme `delivery/docs/change-strategy.md` carries (*The programme*), top first: a quick win — a secret in the tree the same day — then a build below the
   floor (Ant with its jars committed, to Maven or Gradle: first whatever the strategy), then an application
   nobody has proved starts (its run path written and `smoke` recorded: before any slice changes code that was
   here), then a product out of
   support, then a rung of the ladder or a tool the ecosystem has and nothing here runs, each paced as the
   decided strategy says; and, with the programme empty, the lowest map row still below its target with nothing
   `planned`. What is offered is written into the row's `planned` (or the step's evidence named in the slice) and
   handed to `/story-splitting` as a method slice for the split to place among the product slices — never ahead
   of all of them by default, a secret and an open `CRITICAL` excepted. A step that needs a decision nobody has
   made (a release path,
   a change strategy — the programme says *decide it first*) is offered as the question, not as work;
   `/survey` derives the programme again, so a step done is gone rather than ticked. Demo feedback that is
   neither a thing an actor does nor a rung of the map — an icon, a colour, a label — is a task in the next
   slice, never a slice with acceptance criteria of its own.
11. **Demo** — the actor-visible path is ready to show.

Being invoked before any of this exists is a valid start, not an error: it means the entry stage is near the
top of the ladder. Step back to that stage and say so rather than reporting that the request came too early.
Never invent a principle, specification, event, command, stream, or slice to skip a stage — a missing
artifact is work to do with the user, not a gap to fill from context. A stage needing a real product
decision is a stop.

## Who runs each stage

`.specify/models.json` says which model each stage runs on, by role: `strong` where a stage decides what to
build or whether it was built, `fast` where the input is already fully specified on paper — a plan into
tasks, `examples.md` into tests and code, a mutation run. Before running a stage, read its line:

```sh
python3 delivery/scripts/agents/models.py implement   # keyed by the command the stage runs; `make -f delivery/Makefile models` prints them all
```

The line chooses the model; it does not require that model to inherit this session's context. Prefer a fresh
sub-agent whenever the stage can get all of its inputs from artifacts on disk. Six stages are exactly that,
and each is a **named agent type** this project carries in `delivery/agents/`, projected into the installed harness by
`make -f delivery/Makefile agents` with its model and as much of its scope as that harness can enforce:

| Stage | Type | Writes | Runs |
|---|---|---|---|
| `gaps` | `drive-gaps` | nothing | anything that reads |
| `tasks` | `drive-tasks` | only the slice's `tasks.md` | anything that reads, plus the installed tasks command |
| `implement` | `drive-implement` | the files its manifest names | anything |
| `converge` | `drive-converge` | the files its manifest names | anything |
| `adversary` | `drive-adversary` | nothing | anything that reads |
| `mutation` | `drive-mutation` | only the report it produces | anything |
| `skipper` | `drive-skipper` | nothing | anything that reads |
| `hand` | `drive-hand` | only the report it produces | anything |
| `bosun` | `drive-bosun` | the files its manifest names | anything |

`gaps` runs twice and only the pass after implementation is delegated: the pre-planning one may have to ask a
product question, which is the whole reason a stage stays here. Any other conversational stage stays here too,
and has no type for that reason. There is a seventh type, `drive-slice`, for a whole slice rather than a stage:
*Running ready slices concurrently* is where it is delegated, and it reads this section from inside its own
worktree to choose a model for each stage it then runs. The last three rows, `skipper`, `hand` and `bosun`,
are `/cruise`'s: the product owner, the actor and the one who gets a blocked run moving, delegated only when that command is running this ladder on its
own (`delivery/commands/cruise.md`). Under `/drive` alone they run nothing; a person is the owner and the actor.

Delegate to the type by name. The type is the standing brief, so the call adds only the task, its contract and
the file manifest — it never describes the role again or restates the scope, and it does not give the delegate
conclusions. It may give it a map: which precedent to copy, which decision in `research.md` governs, which
helper already exists — a file and a section, which the delegate then opens and reads for itself. Naming where
a fact lives is a pointer and costs a sentence; asserting what it says is a conclusion, and a brief that asked
the delegate to read a decision itself and report what it says has caught what a brief that summarised it got
wrong. Where a brief offers a delegate more than one way of working, every permission is written into each
mode that has it, even at the price of a repeated paragraph: a fresh delegate reads a silence conservatively,
and the conservative reading is the expensive one. The page each type is written on is `delivery/docs/delegated-agent-safety.md`, the standing boundary every
delegation is held to: reference it, restate none of it (`AGENTS.md`, *Delegated agents*).

**A delegate does not inherit this session's code-index connection, and needs none.** Where `AGENTS.md`
carries the CodeGraph extension block, three of the types above — `drive-converge`, `drive-gaps` and
`drive-adversary` — are exploration-heavy, and *what does this code not yet do* is a blast-radius question the
index answers. Delegate them as the table says: every delegate's shell has `delivery/scripts/codegraph`, the pinned CLI
through `npx`, and on Claude Code the server's tool is loaded at the delegate's start, so its brief's first
route to the index is one it has. It names the route that answered, and falls back to text search only when
`delivery/scripts/codegraph` says there is no route. Never pass the parent conversation merely to carry the connection.

Delegate both when the line names another model and when the same strong model can run in a fresh context.
On a harness whose agent file names a model (`delivery/scripts/agents/registry.json`, `agentFile`) the type already
carries the one the table resolved; everywhere else set it explicitly through the mechanism the registry
names, and never accept that mechanism's implicit default. If the harness cannot start a fresh sub-agent on
the selected model, run the stage here and say why. Then read its result from disk the way every stage is
read. Either way the stage says, in one line, which type ran it, which model, whether it was delegated and
whether its context was fresh (`drive-implement · model: sonnet · delegated, fresh context` ·
`drive-adversary · model: host · delegated, fresh context` · `model: host, current context — harness cannot
delegate`). Those lines make type, model and context measurable; a stage that switched any of them silently
cannot be compared with one that did not. Nothing about what a stage produces changes with who runs it — the
artifacts, gates and stops are the same — and a sub-agent that meets a product decision hands the
question back here rather than answering it.

**A line may name another harness** — `implement: local → opencode:ollama/qwen-coder-32k — …`. No sub-agent of
this harness can start that, so the stage runs through `python3 delivery/scripts/agents/delegate.py <stage> --brief <file>
--allow <path> … --verify "<scoped tests> && <scoped lint and format check>"`. Write the brief to a file — the
task, its contract and the map, exactly as for any delegate — and give one `--allow` per manifest entry, a
directory ending in `/`. The verify command is the stage's quickest relevant tests and the lint and format check
the service's gate runs, both scoped to the manifest's files: a model that is not this one slips on style as
well as behaviour, and a slip caught here is undone now rather than found at the push. Run it in the background where this harness can, since it may take many minutes, and alone: it
reads every change in the tree while it runs as the delegate's, so never beside a concurrent sibling or a second
one. It holds the write scope itself, after the run, because the other harness may not: a run that wrote
outside its manifest, changed nothing, failed its verify command, exited non-zero or ran out of time is undone,
commits included, to exactly where it started. Its last line is `delegate: done — …`, which is the stage line,
or `delegate: failed — …`, which names the fallback: rerun the stage there as an ordinary delegation and say both
(`drive-implement · model: opencode:ollama/qwen-coder-32k failed (wrote outside its manifest), rerun on sonnet ·
delegated, fresh context`) — how often the first half happens is what the mapping is measured by. A run that
is kept is not taken at its word on RED: read its log, whose path is on that last line, for a failing run of the
new tests before the passing one, and add `RED observed` or `RED not observed` to the stage line and
`red=observed` or `red=not-observed` to its benchmark entry. It is recorded, not enforced — a log shows what
was printed, not the order the work was done in — so the count is a measure of the mapping, never a reason to
undo a run whose tests pass. Only a stage whose type may run any command is ever sent there; `models.py --check`
refuses a table that would send another.

A stage is not always one delegate. Before delegating implementation, read `tasks.md` for its `[P]` markers and
its *Parallel opportunities* section: the tasks command writes both, and they are the plan for what may run
alongside what — written by one half of this workflow to be read here, not decoration. Every unchecked `[P]`
task whose files are disjoint from the batch already running is a concurrent sibling, delegated in the same turn
with a manifest of its own — and so is a task with no marker whose manifest shares no file with the batch. The
marker is the tasks command's reading of production-code contention, and it under-reports: one slice's list
marked one pair concurrent, said of the rest "none, by construction", and left two pairs that shared no file
to run in sequence. The manifests are the artifact; read them, and let only an overlap with a running
sibling's files, or what the section rules out, keep a task waiting. How many rules one delegate is handed
— a task, a rule or a user story — and how many RED tests each cycle opens with are `.specify/drive.json`'s
two settings (*How implementation is delegated* below), said in the stage line and put on the record. What the
section rules out stays sequential whatever the markers seem to allow — a RED-GREEN-REFACTOR increment starts
from a green, committed suite, and two of them at once is the batched-tests anti-pattern with a `[P]` on it.
The siblings are `drive-implement` delegates, and that type is where the rule they cannot infer for
themselves already lives: **no concurrent delegate writes `tasks.md`**. It is the one file every sibling would
otherwise contend for, so each reports which task it finished and this session ticks the checkbox.

When several delegates form one batch, report once when the batch completes rather than once per delegate.
Verify their claims by spot-checking the recorded reproduction or RED evidence; do not repeat each complete
investigation in the host context. Do not re-investigate. A delegate that was stopped has filed nothing:
everything in its stop notification is a lead, never a result, and a lead is re-run before it is written
anywhere outside this session. An adversary pass is the same kind of batch:
disjoint-manifest seams of one pass are concurrent `drive-adversary` siblings in one turn, and the host
writes the log after the batch rather than re-attacking. It records the type and the explicit model that
ran each seam in `specs/<feature>/adversary-log.md`, especially when seams use different models.

The table is the project owner's to change at any point, and it is read before every stage rather than once,
so a change takes effect at the next stage — and rewrites the agent types, which carry the model on every
harness that reads one from a file: `/model-delegation-settings implement=strong claude.fast=haiku` edits it checked
(`delivery/commands/model-delegation-settings.md`, over `python3 delivery/scripts/agents/models.py --set`), or edit the file by hand and let
`make check-agents` hold the shape. When the owner asks for a different model at a stage, `/model-delegation-settings` is the
change — not a note, and not a switch made silently in the delegation. Commit the file: the choice is versioned with the project, and `slipwai
migrate` merges a newer factory's table over it rather than replacing it.

## What each stage costs

Every stage is recorded, so a change to a prompt, a skill or the layout can be compared on what it did to cost
and quality rather than on impressions. `delivery/scripts/agents/benchmark.py` keeps one record per slice,
`specs/<feature>/slices/<id>/benchmark.json`, beside the slice's other artifacts; the stages above the slice loop
go to `specs/<feature>/benchmark.json`, since they are the feature's cost and not the next slice's. Before a
stage, open its entry:

```sh
python3 delivery/scripts/agents/benchmark.py start specs/<feature>/slices/<id> implement
```

After it, close the entry with the signals that stage owes, and nothing else:

```sh
python3 delivery/scripts/agents/benchmark.py end specs/<feature>/slices/<id> implement verify_failures=1
```

| Stage | `end` takes |
|---|---|
| `gaps` | `gaps=N` — criteria or states added before the plan; findings traced after converge |
| `implement` | `verify_failures=N` — red `make verify` runs during the stage |
| `implement` | `delegate=…` and `cycle=…` — how the stage was delegated and driven (*How implementation is delegated*) |
| `implement` | `split=N` — groups the delegate fanned out into; `0` where it did not |
| `adversary` | `findings=N` — findings the pass recorded in the log; a recorded skip is `0` |
| `adversary` | `seams=N` — delegates spawned; a recorded skip is `0` |
| `mutation` | `mutation_score=…` — copied from the tool's own line, in its own units |
| `demo` | `outcome=accepted`, `outcome=behaviour` or `outcome=implementation` — what the feedback changed |
| `any` | `model=…` only when the record shows no transcript was read and the stage said which model ran it |
| `any` | `agent=…` the same way: only when the transcript attributed the delegate to no type and the stage delegated to one |

Everything else is read, not asked: wall time; tokens by model from the harness's own transcript between the two
moments — Claude Code's and Codex's today; anywhere else the record says `null` and why — whether the stage was
delegated, and to which agent type where the transcript names one (Claude Code attributes every sub-agent line
to the type that ran it, so an adversary pass on this slice is comparable with the same type on another); the tasks a converge pass appended; how many converge passes there were; a stage re-entered after
implementation. A delegated stage is started and ended here, by the host: the sub-agent's transcript is found from
this session's. A number the script could not read is `null` with its reason and stays that way — never fill one
in, and never pass `model=` when the record already names one. Start and end must bracket the work itself. Two
brackets open at once are told apart: a line goes to the innermost bracket covering it, and a delegate's lines to
the bracket whose stage owns the type that ran them, so a skipper round during implementation costs the skipper
and not the implementers. An entry a session leaves open is cut off — by the `/cruise` runner when the iteration
ends, and by the next `start` in the same record — with the reason, its tokens read from the transcript it left,
and no signals: nothing will close it truthfully afterwards. `make -f delivery/Makefile
check-benchmark`, in `make -f delivery/Makefile verify`, warns of an entry still open, a slice the ladder calls done with no
record or an unclosed one, and a feature with done slices and no record above the slice loop — warns, never fails,
because a bracket missed cannot be taken afterwards. Close
`adversary` after its findings are triaged and before any fix, passing `findings=N` and `seams=N`; each
failing test and fix belongs to a new
`implement` entry. Bracket `demo` around the actor's session, not the note afterwards, and `mutation` around the
run. A same-moment start and end is reported as `unbracketed`, not `0s`, and makes the slice wall a floor.

When the slice is archived, close its record:

```sh
python3 delivery/scripts/agents/benchmark.py close specs/<feature>/slices/<id>   # adds tasks, files and lines, prints the aggregate
```

Closing also redraws `specs/<feature>/benchmark.md`, the overview `/benchmark` writes on demand; `make -f delivery/Makefile
benchmark` prints the same aggregate at any time — one row per slice: cost, converge passes, gaps, mutation score,
findings, demo outcome, rework, shape. Commit the record and the page with the slice. Nothing on either goes on the
demo board: cost is the team's, and the board is the actor's.

### How implementation is delegated

Two settings in `.specify/drive.json` decide how this ladder hands implementation to `drive-implement` and how each
delegate drives what it is handed. Read them before every implementation stage — `python3 delivery/scripts/agents/drive.py` — say
both in the stage line beside the model (`drive-implement · model: sonnet · delegated, fresh context ·
story/rule`), and change them only through `/drive-settings`, never silently inside a delegation.

**`delegate`** — how much one delegate is handed. `story`: every rule of one user story (`[US<n>]` on the
tasks), each rule its own cycle in one context — the boundary that stops each fresh delegate re-reading the
same plan, map, precedent and test file per rule, which was most of a slice's implementation wall where it
was measured. `rule`: one rule with its examples. `task`: one task as the tasks stage cut it — the finest
boundary, and the most independent checking of what this session asserted. Whatever the boundary, the
delegate may fan its work out to sub-delegates over disjoint files under the constraints its brief carries,
and a sub-delegate is handed whole cycles, never part of one.

**`cycle`** — how many RED tests a RED-GREEN-REFACTOR cycle opens with. `rule`: a rule's examples written
together, each observed failing for its own stated reason, stub-first so none fails on a build, then the
smallest code that passes them. `example`: one at a time. There is always a cycle; what this setting loosens
is one test per cycle. A story is never a cycle unit: every rule of a story red before any is implemented is
the batch Principle V prohibits, and `delivery/scripts/agents/drive.py` refuses it.

The defaults are `story` and `rule`. Two vetoes override them, written as vetoes
because a preference is what the next edit simplifies away: tasks that carry no story tag are delegated per
`rule`, and a map that does not number its rules is delegated per `task` and driven per `example`, since
there is no agreed rule boundary to cut on. `example` is also the right per-slice choice for a rule where one
assertion at a time is worth the cycles — money, authorisation, anything the constitution names as a MUST —
and saying so in that slice's delegation brief is allowed; changing the default is `/drive-settings`.

**Parallelism is this session's duty at every boundary.** Siblings whose manifests are disjoint — stories,
rules or tasks — run concurrently in the same turn, derived from the manifests rather than from a `[P]`
marker alone (*Who runs each stage*), and a delegate fans out inside its boundary the same way. What is never
parallel is one cycle: a RED-GREEN-REFACTOR increment starts from a green, committed suite.

The delegate reports the boundary it was given, the cycle unit it ran, and whether it fanned out and into how
many groups; this session passes them to the record as `delegate=`, `cycle=` and `split=N` on the implement
entry, so a wall time says what it was a wall time of. `make -f delivery/Makefile benchmark` shows them beside each slice.

## Once inside the slice

Start the slice from a green `make verify`. During implementation, take one RED-GREEN-REFACTOR increment per
task — one rule of the example map with its examples, where the map numbers its rules — run only the quickest
relevant tests in the same file or area, commit that increment locally, and keep
task checkboxes truthful. A local commit is not a push: it does not run the full gate and it does not start
CI. Do not push increment commits until the actor has accepted the demo. Before an increment that changes a
shared function, ask `codegraph_explore` what calls it and what the change reaches — loaded by name where the
harness defers it — and name those callers in the delegate's manifest; a project without `.codegraph/` answers
with a text search and says so.

When the tasks are done, converge, then stop at the actor-visible demo from the unpushed slice branch. After
acceptance — and only then — a project that has adopted CodeGraph runs `codegraph sync`, then the full
`make verify`, then the first push of those increment commits (and the merge that lands them on trunk).
That push is the integration boundary. A claim of `slice/<id>` at the start of the slice may still push a
lock ref from `main`; that is not the implementation.

### What the demo stop has to contain

The stop is a pause for feedback, so it opens with where the product stands and ends with the thing the
actor uses and a question only they can answer.

**It opens with the progress board.** The actor should never have to ask how many slices there are or how
far along the work is: that question, asked, means the board was missing — and between demos, `/where-are-we`
draws the same board on demand. Seven parts, in this order, every
line in the actor's vocabulary rather than a slice id or a test name:

- ✅ **Works now** — every accepted slice, one line each, as the thing the actor can do
- 🆕 **New in this demo** — what this slice added: the thing about to be shown
- ⬜ **Still to come** — the remaining slices of the split, in order and by name, headed by one line of
  counts: `N of M slices accepted`
- ⚠️ **Not working yet** — every deliberate stub and every release constraint still in force, named as
  such, so a hole reads as "not yet" and not as a fault the actor has to find
- 🔀 **Ready (parallel)** — every slice that can start now: not done, every `depends_on` done, and not
  pre-empted by an open `CRITICAL` — in two groups, *claimed* (a `slice/<id>` branch on the forge, by whom
  and how long ago) and *unclaimed*, so another session knows which sibling is free
- ➡️ **Next (this session)** — the slices this `/drive` will take: every unclaimed ready slice whose
  contract is settled, concurrently; or the earliest ready slice in split order where the harness cannot
  delegate, unless the user picks another ready one
- ⛔ **Blocked** — remaining slices waiting on unmet `depends_on`, or an open `CRITICAL` ahead of them

The board is derived from artifacts, not memory, the way the entry stage is:
the ordered split and its `## Slice graph`, the register at `specs/<feature>/slices/README.md` — a row
marks a slice accepted; a slice with `plan.md` under `slices/<id>/` and no row is in flight —
and the forge's `slice/<id>` branches (`git ls-remote --heads origin 'slice/*'`), which are the claims — read, never assumed: where that command fails the board says the claims could not be read, and shows no slice as unclaimed on the strength of a failed read.

The task count is not on it. Tasks stay in `tasks.md` for whoever is doing the work; the board counts
slices, because a slice is a thing the actor can use and a task is not, and fifty-eight tasks over two
slices reads as fifty-eight features to somebody who did not write them.

**It ends with the thing the actor uses.** The final turn states, in its own words:

- the literal command or URL that runs it — the exact text to paste, not a description of where to look
- the seed data it needs, or that it needs none
- the result to expect, in the actor's vocabulary rather than a test name

**When "the thing the actor uses" is a long-lived process** — a dev server, a container — starting it to
verify the command works and then stopping it once verification passes is not a demo, it is a test you ran
alone. The actor's first action is opening what you just closed. Start it in the background, verify it, and
leave it running past the end of the turn; say plainly that it is still up rather than letting the actor
discover a dead port.

**When the slice touched how the application starts** — constructors, dependency injection, configuration,
module registration, the build — the demo is the application starting with the change in place, by the command
`run-the-app` records (`make smoke` where one is recorded), and never the suite passing in its place: a suite
that never builds the context cannot see a constructor the container cannot call, and slices have shipped that
way, converged and green, with an application that no longer started.

Then it asks directly what using it revealed. A summary of what was built, a compliance table, or a green
gate report is evidence *for* a demo and never the demo itself: it hands the actor nothing to use and asks
them nothing. **A turn that reaches the demo without asking that question has not paused for feedback — it
has stopped**, and from the outside those look identical until the slice sits idle waiting for an
invocation nobody knew was needed.


### After acceptance, and after Phase 4 clears

After acceptance, run `/adversary`, which decides whether the slice changed attack surface or closed the
split and records the attack or the skip — `make check-decisions` holds every done slice to that row. Close the
adversary benchmark entry after its findings are triaged; implement confirmed defects through failing tests,
each in an `implement` entry. Then run `/mutation`, then `make verify`. `delivery/commands/adversary.md` owns the
trigger table; do not spawn before it is in the log. It records that decision in
`specs/<feature>/adversary-log.md` either way, so do not make it here. The order is not arbitrary: the pass
adds tests, and mutation measures whatever exists when it runs. Stop earlier only for a product decision or
unavailable input.

Once that evidence is clean, **continue on the same run**: mark the finished slice done —
a row in the register at `specs/<feature>/slices/README.md` — and confirm its
`plan.md`, `research.md`, `data-model.md`, `quickstart.md` and `tasks.md` are under
`specs/<feature>/slices/<id>/` with their relative links pointing at what they cite (they have been since it
was planned; a regular file still at the feature root is moved there now, and the canonical links dropped).
Then select the next slice from the **ready** set and re-enter the ladder at whichever stage that slice's
own artifacts require, which is usually its example map or its plan rather than the top. Do not wait to be
invoked again.

**Ready-set selection** (which slices may start now, and whether this session takes one or all of them):

1. Read the slice graph — `## Slice graph` in the split — and which slices are **done**:
   a row in the register at `specs/<feature>/slices/README.md`. A slice with `plan.md` under `slices/<id>/` and no such mark is in
   flight, not done.
2. An open `CRITICAL` in `specs/<feature>/adversary-log.md` is the next slice, ahead of every ready
   product or method slice.
3. Otherwise **ready** = not done, every `depends_on` done.
4. If ready is empty, stop — the split is exhausted or every remaining slice is blocked.
5. If ready has one slice, claim it and take it.
6. If ready has several, **name the full ready set**, split into *claimed* and *unclaimed* by the
   forge's `slice/<id>` branches, and run every unclaimed one whose contract is settled concurrently, as
   *Running ready slices concurrently* says. Where the harness cannot delegate, claim the earliest in the
   ordered split for this session and leave the rest named, so another session can claim a sibling. Do not
   stop merely to choose among them unless the user asks.

### Running ready slices concurrently

Slices that share only a written contract — a route, a schema, a port — are independent: neither
has to be built first, and each is tested against the contract rather than against the other.
So one session need not take one slice at a time. Fan out over the ready set when, and only when, the
four things below hold; the reference implementations of this method parallelise exactly this way, and
these are the primitives they add.

**The contract is settled.** A ready slice runs alongside its siblings only when what they share is
written down: the entries it adds to `specs/<feature>/contracts/` exist, and its acceptance criteria in
`spec.md` have had their gaps review. That is the contract a concurrent sibling builds against. A ready
slice whose surface is still being decided is worked here first, never delegated alongside the others.

**Each slice is claimed.** A claim is a `slice/<id>` branch on the forge — nothing in the model changes,
so `check-model` learns nothing new. Claim by pushing the branch and expecting it not to exist:

```sh
git push --force-with-lease=refs/heads/slice/<id>: origin HEAD:refs/heads/slice/<id>
```

Rejected means another session got there first: skip to the next unclaimed ready slice, never retry,
never error. Read the claims back with `git ls-remote --heads origin 'slice/*'`; the board shows 🔀
*Ready* split into *claimed by* and *unclaimed*. A claim whose last commit is days old is reported as
stale, never silently taken — its owner may be mid-slice. Without a remote, the local branch is the claim,
and say so. A remote that is configured and cannot be reached is the other case, and it looks like success:
`ls-remote` fails, every slice reads as unclaimed, and that is exactly the answer that lets two sessions take
one slice. A failed read is reported as *claims could not be read*, never as *unclaimed*, and no slice is
claimed on the strength of it.

**Each slice has its own worktree and one delegate.** For every unclaimed ready slice whose contract is
settled, in the same turn: claim it, give it a worktree (`git worktree add ../<project>-<id> slice/<id>`;
on Claude Code the Agent tool's `isolation: worktree` makes one), and delegate the slice's ladder — its
example map through its converged verdict — to one fresh `drive-slice` delegate (`delivery/agents/drive-slice.md`,
the standing brief) with a manifest naming the worktree, the slice's block of the model, its `examples.md`,
and the shared-surface rule below. That type takes no stage's model, because *Who runs each stage* still
chooses one stage by stage inside the delegate, where `[P]` tasks still fan out to `drive-implement`: the
two levels nest. Inside one slice the stages stay strictly sequential. A delegate that meets a
product question stops its slice with the question recorded in its `plan.md` and hands it here — a blocked
slice is marked blocked, never guessed past.

**The shared-surface rule**, which the delegates cannot infer and `make -f delivery/Makefile check-slice-scope` holds on
every `slice/<id>` branch: a slice's commits touch its own `specs/<feature>/slices/<id>/`, the feature's
cumulative artifacts (`spec.md`, `story-split.md`, `contracts/`, `checklists/`, `adversary-log.md`, and
`decisions.md`, where `/cruise` records a decision taken during the slice), its own block of `model.yaml`
with the committed canvas `model.drawio` regenerated from it (`make -f delivery/Makefile model-drawio` — `check-drawio`
holds the canvas to the model, so a slice that advanced its block cannot pass `verify` without it), the
mockups, the code and tests of the service that owns it — one bounded context where the service holds
several — the context's events module *additively*, **new** migration files named by a timestamp
(`date -u +%Y%m%d%H%M`, so two slices never mint the same name), a **new** ADR under `delivery/docs/adr/` at
`Proposed` (never an edit to one that stands), and the composition root. Nothing else —
`Makefile`, `project.json`, package manifests and locks, `delivery/scripts/`, `delivery/skills/`, `delivery/agents/`, the other docs —
is a slice's to write: a delegate that needs one of them hands the need back here, and it lands on `main`
before the fan-out or between merges. The canonical slot at the feature root is a link, never committed.

**Demo on the slice branch, then verify, then push.** As delegates report converged, demo each slice from
its unpushed worktree in split order — never from `main`, never by pushing increment commits first. A
claim may already have pushed a lock ref from `main`; leave the increment commits local until the actor
accepts. After acceptance: `codegraph sync` if the project has adopted a code index, `make -f delivery/Makefile verify`
green, then push the slice's commits and merge into `main` in split order — never in finishing order.
That is the first implementation push, and it is what starts CI. The composition root and the cumulative
artifacts are where two merges meet, and split order is what makes those resolutions predictable;
regenerate the Mermaid diagrams (`make -f delivery/Makefile model`) after a merge, never in a branch, and the canvas
(`make -f delivery/Makefile model-drawio`) after each merge as well, taking both sides' blocks. No slice's Phase 4 runs until its demo is
accepted, and a sibling's demo never waits on another's Phase 4. Phase 4 itself — adversary, mutation,
`make -f delivery/Makefile verify`, marking the slice done — runs here, on `main`, one slice at a time. Delete the
`slice/<id>` branch once its Phase 4 clears: the claim is spent.

**Where the harness cannot delegate**, run one slice at a time here — claim the earliest in split order,
name the rest — and say so in the line that says which model ran (`harness cannot delegate`).

The stops are a required product decision, an input that is genuinely unavailable, a split with no ready
slice left in it, and the next slice's own demo. **A slice having finished is not one of them.**

Demo feedback re-enters the ladder at the stage that owns the change, which may sit well above the slice
loop. Re-derive the entry stage from artifacts instead of assuming the loop resumes where it paused.

