# How the delivery method was installed here

> **Experimental.** Brownfield adoption is new and will change shape while real repositories teach it what it got wrong: the files under the delivery directory, the facts `project.json` records and the questions `adopt` asks may change in a MINOR release, and `slipwai migrate` brings each change here with a note saying what to do. Every place this reaches you says so until it stops being true. What surprised you — a detection that was wrong, a gate that went red, a sentence this page should have had — belongs on the public issue tracker.

`slipwai` existed before this method did. `slipwai adopt` surveyed the tree, proposed what it found, and
wrote the method's material under `delivery/` beside the code — nothing of the repository's own was
written over, and `project.json` records every fact with where it came from: `detected` from the tree,
`confirmed` by the person who accepted it, `overridden` by the person who changed it.

**Why:** Run /cruise on the factory itself to build Rust support and execution-model delegation


## What was wrapped

### `.` — `slipwai`, a tool in python (python, python 3.11)

The factory: generates and adopts repositories that carry the delivery method

Provenance: language `detected`, commands `overridden`, kind `overridden`, purpose `overridden`.

| Target | Command |
|---|---|
| `install` | `make install` |
| `typecheck` | `make typecheck` |
| `lint` | `make lint check-structure` |
| `test` | `make test` |
| `integration` | *none recorded — a written no; the target passes and says so* |
| `adversarial` | *none recorded — a written no; the target passes and says so* |
| `audit` | *none recorded — a written no; the target passes and says so* |
| `mutation` | *none recorded — a written no; the target passes and says so* |

`python` is a language this factory generates, so `add-service --language python` can put a generated service beside the existing one, with every axis and gate a generated service has.

## Where the rest lives

**The database schema** is not versioned anywhere this survey could see: applied by hand, by a DBA or a console. The first step of the data or infrastructure work is a versioned baseline, described before it is imported (`detected`).

**The deployment infrastructure** is versioned in this repository (`opentofu / terraform`) (`detected`).

Neither is something the factory manages here: `adopt` owns no environment and applies nothing. Where either
is `here`, the rule is import or reference — never manage a resource in two places. Where it is `elsewhere`,
the other repository is the contract. Where it is `unmanaged`, the finding is written down and the first step
is named.

## What runs

`make -f delivery/Makefile verify` is the gate: the method's own checks — agent projections, Spec Kit, the constitution floor,
the import rule over anything the factory generates — and then every wrapped application's recorded commands,
in the order above. A target recorded `null` passes with a line saying so; it is a written no, not a gap to
fill by guessing. `lint` and `typecheck` run through the ratchet (`delivery/scripts/ratchet.py`): the
first local run records the findings that are there into `delivery/baseline.json` — commit it — and
from then on the gate fails only on a finding that is new; `make ratchet-tighten` re-records what is left once
somebody has looked. `test` runs through the same ratchet with one difference: a suite that is red on day one
stops the first run and says so — a red suite is a fact a person reads, never one the gate records behind them —
and `make ratchet-tighten`, once they have, records it as **quarantined**: the gate passes on the state it
recorded and says so every run, until the suite is green and `ratchet-tighten` clears it. A new failure is still
a failure, whether the output names a file at a position or the runner names the test (`--- FAIL: TestX`,
`not ok 3 - adds`, `FAILED tests/test_a.py::test_b`, Surefire's `[ERROR]   ShopTest.adds`). `smoke` is the one
command the gate cannot compose from the eight: the application started and proved to answer, recorded as
`commands.smoke` once `delivery/survey/running.md` says how (`/ground` asks; `null` is a written no with
its reason there). `make -f delivery/Makefile smoke` runs it, `make -f delivery/Makefile ci` and the gate's own smoke job run it in CI, and `verify`
never does, since it needs what the application needs. A Maven or Gradle build runs through its wrapper
(`./mvnw`, `./gradlew`): where the repository had none, `adopt` wrote one beside the build file, so the machine
needs a JDK and no Maven or Gradle of its own. Any other recorded command whose tool is not on the machine
running it cannot be baselined — the ratchet fails, names the tool, and records nothing until it is installed or
`project.json` records what this machine does run. A suite recorded as `test-full` is too slow for the gate and
has a target of its own. The same `verify` runs in CI from `.github/workflows/verify-delivery.yml` on GitHub Actions (`overridden`), beside whatever CI this repository already had.

## How a change reaches production

**A pipeline deploys** (`detected`, from `pipeline: .github/workflows/package.yml`, `pipeline: .github/workflows/publish-package.yml`, `pipeline: .github/workflows/release.yml`, `pipeline: .github/workflows/verify.yml`, `scripted: assets/targets/aws/scripts/deploy.py`, `scripted: assets/targets/azure/scripts/deploy.py`, `scripted: tests/fixtures/adopt/converging/deploy.sh`, `scripted: tests/fixtures/adopt/javascript-gitlab/deploy.sh`, `scripted: Makefile`). Which environments it passes through, what gates each, and how a release is undone belong in `delivery/docs/deployment.md`; `/drive` asks every slice how it reaches production.

## Next

1. `./delivery/init` — first, as in any project the factory made. It installs Spec Kit and asks which
   coding agent to project the skills and commands into (`./delivery/init --integration claude` names it
   outright; `python3 delivery/scripts/agents/project.py --list` shows every agent it knows).
   `./delivery/init --extension codegraph` indexes the code, which is what makes a legacy codebase safe
   for an agent to work in; the two flags go together in one run.
2. `/ground`, in the agent — the question set the tree could not answer: one row of
   `delivery/docs/convergence.md` at a time, the evidence and the rungs shown first, each answer written
   with `confirmed` provenance; then `/survey` so the pages and the strategy recommendation follow. What
   the tree could not say, or `--yes` left `unrecorded`, is settled here rather than one slice at a time —
   or skip straight to `/drive`, whose Ground stage runs the same questions for the rows a slice touches.
3. `make -f delivery/Makefile verify` — green on day one is the promise for a linter or type checker that arrived after the
   code, and the first run records the ratchet baseline: commit `delivery/baseline.json` with what `init`
   wrote. A test suite that is red on day one is the one exception: the run stops and says so, and
   `make -f delivery/Makefile ratchet-tighten` quarantines it once you have read the failures. Anything else not green is the
   repository's own command failing, and `project.json` is where to correct what the survey got wrong.
4. Offer the targets as the repository's own: add `-include delivery/Makefile` to the root `Makefile`,
   and `make verify` is one word again.
5. Read `delivery/docs/convergence.md`: where this repository stands on every ladder a generated project
   sits at the top of, what is planned to move each row, and what nobody has established yet; and
   `delivery/survey/structure.md`, the architecture view — where anything starts, what depends on what,
   where change happens — which says what moves the Structure row and where `/strangle` would cut. Then
   `delivery/docs/getting-started.md` and `delivery/docs/architecture.md`, then `/drive`,
   whose ladder here begins at *Ground* (no map, no principles), pins wrapped code before *Implementation*,
   and holds the map to each slice at *Convergence* before offering the next row as a method slice.
   Three more commands are this adoption's own: `/characterise` pins the current behaviour of the code a slice is about
   to change, at the seam where it can be observed (`delivery/survey/pinned.md` is the ledger) — with
   fakes written in the test tree, never a mocking framework it would have to add — and
   `/survey` runs `slipwai adopt --refresh` to reconcile a fresh survey with what `project.json` records.
   How each application is *run* is written, once proven, in `delivery/survey/running.md` — the
   repository's own file, which the `run-the-app` skill points to and the factory never rewrites.
   When the code is ready to move, `delivery/docs/change-strategy.md` opens with the strategy `why` and
   the map recommend for this repository — *leave it* included — then the three strategies and the order to
   change in; an accepted ADR with a `Strategy:` line is the decision, and `/strangle` moves one capability
   at a time only under a decision that says `strangler-fig`, writing `delivery/retirement.md` as it goes.
6. Merge this adoption alone, before the first slice, and from then on one pull request per slice — a slice's
   after-acceptance commits ride in its own PR. A reviewer can read a slice; nobody reads five at once.
7. Later, `slipwai migrate` brings a newer factory's material here as one merge; the files it wrote are listed
   in `delivery/.written`, and nothing else is ever touched.

Profiles, axes and everything else the factory offers a generated project are described in
`delivery/docs/`; `typescript, python, go, rust, java-quarkus, java-spring` are the backends it can add beside what is here.
