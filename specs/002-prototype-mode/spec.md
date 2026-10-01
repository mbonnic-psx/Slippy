# Feature Specification: Prototype mode — a fenced, throwaway prototype product people can use while the spec is written

**Feature Branch**: `002-prototype-mode`

**Created**: 2026-10-01

**Status**: Draft

**Input**: User description: "A prototype mode in Slippy: while we are running through specs in the beginning, take the
PRD and start vibe coding without worrying about security concerns because it is just a prototype, and give the product
people a space to work with it — a simple localhost or a Claude Artifact. As questions for the spec are answered, the
prototype is updated. It also lets UI/UX be designed in tandem with building the production code." Decisions taken with
the requester on 2026-10-01: the prototype is throwaway; a private Claude Artifact is the default home and localhost the
upgrade; it applies to generated and adopted repositories alike; it works under `/drive` and `/cruise`. Precedent:
Convene's `E3p · event-emails-prototype` slice, built by hand on a local demo branch and ported behind a flag.

## User Scenarios & Testing *(mandatory)*

Three actors meet this feature:

- **The product person** — a product manager, designer or stakeholder who owns what the product should do but does not
  read code. They open the prototype, click through it, and say what is wrong with it.
- **The maintainer** — whoever runs `slipwai generate` or works in a repository it made or adopted, and runs `/drive` or
  `/cruise` there. They turn prototype mode on and own the repository's gate.
- **The run** — `/drive` or `/cruise` acting on the maintainer's behalf: it builds and updates the prototype, and under
  `/cruise` its skipper answers product questions the way it already does for every other stage.

Today the ladder goes from a specification straight to a split and then to production slices. The first thing a
product person can click is the first slice's demo, days in, built to production standard — so a wrong idea about the
screens is found after it has been built properly, and UI/UX design waits on code. Prototype mode puts a clickable,
deliberately disposable version of the product in front of product people while the specification is still being
written, and fences it off so that what makes it fast — no security, no tests, no real data — can never reach
production.

### User Story 1 - A product person clicks through a prototype built from the spec (Priority: P1)

The maintainer turns prototype mode on and runs `/drive` (or `/cruise`) on a feature whose `spec.md` exists but whose
split does not. Before the split, the run builds a prototype from what the specification and its PRD describe: every
user story the spec names as something an actor does becomes a screen or a step the product person can click through,
using made-up sample data. It is published as a private Claude Artifact and the run hands back its link. The product
person opens the link with nothing installed and walks the journeys. Every screen carries a visible banner saying it is
a prototype and that nothing in it is saved or sent.

**Why this priority**: It is the whole value: a product person can use the idea before anyone has built it properly.
Every other story improves or protects this one.

**Independent Test**: In a generated project with prototype mode on and a `spec.md` of three user stories, run the
prototype stage; open the published link and check that each user story is reachable as a clickable journey, that every
screen shows the banner, and that the prototype's source is under the prototype folder and nowhere else.

**Acceptance Scenarios**:

1. **Given** prototype mode is on and a feature has a `spec.md` but no split, **When** the run reaches the prototype
   stage, **Then** it builds a prototype under the project's prototype folder, publishes it as a private Claude
   Artifact, and records the link and the spec revision it was built from.
2. **Given** a published prototype, **When** the product person opens it, **Then** every user story of the spec is
   reachable as a journey, and every screen shows the "Prototype — nothing here is saved or sent" banner.
3. **Given** the prototype, **When** any of its screens shows people, accounts, money or messages, **Then** what it shows
   is sample data held inside the prototype folder, never data read from a real system.
4. **Given** prototype mode is off — the default for every project generated or adopted before this feature —
   **When** the run reaches the point where the stage would sit, **Then** nothing is built and nothing about the ladder
   has changed.
5. **Given** a published prototype, **When** the run republishes it after a change, **Then** the link stays the same.

---

### User Story 2 - The fence: nothing in the prototype can reach production (Priority: P2)

Because the prototype is built without security, tests or review, the repository's gate holds a fence around it rather
than holding it to the production standard. Production code that imports anything from the prototype folder fails the
gate. A credential or secret anywhere in the prototype folder fails the gate. The production build, image and deploy
never include the prototype folder. Inside the fence, the gate asks nothing else of the prototype: no coverage, no
mutation score, no lint strictness.

**Why this priority**: The promise "do not worry about security, it is just a prototype" is only true if a prototype
can never become production by accident. Without the fence, prototype mode is a way to ship unreviewed code.

**Independent Test**: In a generated project with prototype mode on, add one production import of a prototype module
and run `make verify`: it fails and names the import. Remove it and add a string shaped like a credential in the
prototype folder: it fails and names the file. Remove that: it passes, although the prototype has no tests.

**Acceptance Scenarios**:

1. **Given** production code that imports a module from the prototype folder, **When** `make verify` runs, **Then** it
   fails, naming the importing file and the imported prototype path.
2. **Given** a credential-shaped value anywhere under the prototype folder, **When** `make verify` runs, **Then** it
   fails, naming the file.
3. **Given** a prototype with no tests, untyped code and lint findings, **When** `make verify` runs, **Then** none of
   those fail the gate.
4. **Given** a project with prototype mode on, **When** its production build, image or deploy artifact is produced,
   **Then** it contains nothing from the prototype folder.
5. **Given** a prototype that calls out to any network address other than its own host, **When** the fence check runs,
   **Then** it fails, naming the call.

---

### User Story 3 - Answers and reactions flow back into the spec, and the prototype follows (Priority: P3)

The product person reacts to the prototype — a comment on the Artifact, or a message the maintainer passes on through
`/cruise-tell`. Each reaction is read as a gap in the specification: the run turns it into a question or a new
criterion in `spec.md`, logs the answer in `decisions.md`, and then updates the prototype to match the specification as
it now reads. The prototype never runs ahead of the specification: a change seen in the prototype is always a change
written in `spec.md` first. Under `/cruise`, the skipper answers a reaction's question the way it answers any other,
and the reaction's thread on the Artifact gets a reply saying what was decided and where it was written.

**Why this priority**: It is what makes the prototype more than a mock-up: it is how product people shape the
specification without writing it, and how the specification stays the one source of truth.

**Independent Test**: Publish a prototype, leave one comment on it that contradicts a criterion, and run the next
iteration; check that `spec.md` changed, a `decisions.md` entry names the comment, the republished prototype shows the
change, and the comment has a reply.

**Acceptance Scenarios**:

1. **Given** a comment on the published prototype, **When** the run next reaches the prototype stage, **Then** the
   comment becomes a gap: a question answered in `decisions.md` and, where it changes what the product does, a
   criterion added or changed in `spec.md`.
2. **Given** a specification that changed since the prototype was last published, **When** the run reaches the
   prototype stage, **Then** the prototype is rebuilt from the current specification and republished to the same link.
3. **Given** a reaction that would change behaviour no criterion in `spec.md` describes, **When** the run handles it,
   **Then** the criterion is written before the prototype changes — never the other way round.
4. **Given** a comment answered by the run, **When** the product person reopens the prototype, **Then** the comment's
   thread carries a reply naming the decision and the criterion it changed.
5. **Given** `/cruise`, **When** a reaction raises a question the stage cannot answer from the specification, the
   constitution or a standing decision, **Then** it goes to the skipper and its entry says `Decided by: drive-skipper`
   — so a person can see which answers to the product person were the machine's.

---

### User Story 4 - The prototype moves to localhost when it outgrows an Artifact (Priority: P4)

When the prototype needs the project's real design system, more screens than a single page holds comfortably, or a
browser-driven walk-through, the maintainer upgrades it to run locally with `make prototype`. The same source, the same
banner and the same fence apply; the Artifact is left as it last was, with a note on it pointing to the local version.

**Why this priority**: The Artifact is the zero-setup default and covers most early prototypes; localhost matters for
the minority that need the real frontend stack.

**Independent Test**: In a generated project with a frontend, upgrade a prototype to local and run `make prototype`;
open the printed address and check the banner and the journeys; check the gate's fence still applies.

**Acceptance Scenarios**:

1. **Given** a project with prototype mode on, **When** the maintainer runs `make prototype`, **Then** the prototype is
   served on a local address that is printed, and each screen shows the banner.
2. **Given** a project generated with a frontend, **When** the prototype runs locally, **Then** it can use that
   frontend's components and styles, while still living only under the prototype folder.
3. **Given** a harness that cannot publish a Claude Artifact, **When** the run reaches the prototype stage, **Then** it
   falls back to the local prototype and says so, rather than skipping the stage.
4. **Given** a prototype upgraded to local, **When** its Artifact is next republished, **Then** the Artifact says where
   the current version is and stops being updated.

---

### User Story 5 - Accepted prototype screens become the design reference for each slice (Priority: P5)

Once the split exists, each slice that has a screen names the prototype screens it replaces. The product person's
acceptance of a prototype screen makes it that slice's design reference: the slice's demo is compared against it, and a
difference is either a behaviour the specification changed since (and says so) or feedback. As each production slice
lands, the prototype screen it replaces is marked replaced and links to the real one; journeys keep their route names,
so a product person's path stays where it was. When every screen is replaced, the prototype is retired: its folder is
deleted in one commit and the link says it has been retired.

**Why this priority**: It is what lets UI/UX be designed in tandem with production code rather than ahead of it or
after it — but it only matters once the earlier stories exist and slices are being built.

**Independent Test**: With a split whose first slice names one prototype screen, carry the slice to its demo; check the
demo stop names the reference screen, that after acceptance the prototype marks that screen replaced, and that a
second, final slice retires the prototype.

**Acceptance Scenarios**:

1. **Given** a slice with a screen, **When** it is split, **Then** its row names the prototype screens it replaces, or
   says none.
2. **Given** a prototype screen accepted by the product person, **When** the slice that replaces it reaches its demo,
   **Then** the demo stop shows the reference screen beside the real one.
3. **Given** a slice accepted, **When** its screens are in production, **Then** the prototype marks each replaced screen
   as replaced, with a pointer to the real one.
4. **Given** every prototype screen replaced, **When** the last slice is accepted, **Then** the prototype folder is
   removed in one commit and the published link says the prototype is retired.
5. **Given** `/cruise`, **When** a prototype screen would be accepted as a design reference, **Then** it is recorded as
   `accepted-by: drive-skipper — pending human review` until a product person accepts it.
6. **Given** prototype code a team wants to keep, **When** they choose to port it, **Then** the port is a slice of its
   own, built test-first to the production standard like any other — the fence is never lifted to let prototype code
   through.

### Edge Cases

- **A product with no screens** (an API, a CLI, a library): the prototype is a clickable walk-through of what the actor
  does and sees — requests and responses, commands and their output — rather than application screens. Prototype mode
  still applies.
- **The prototype folder's name is already taken** in an adopted repository by something that is not a prototype: the
  run refuses to write into it, says so, and asks for another location, which is recorded in `project.json`.
- **A spec too thin to prototype** — fewer than one user story with an actor and an action: the stage says what is
  missing and does not build an empty prototype.
- **A comment that asks for something out of scope**: it becomes a decision entry saying it is out of scope, with a
  reply on the thread; the prototype does not change.
- **Two comments that contradict each other**: they are one question, answered once, and both threads get the reply.
- **A comment on a prototype built from an older spec revision**: it is read against the current specification; where
  the current spec already answers it, the reply says so.
- **Someone pastes real customer data into the prototype**: the fence's secret check does not catch every kind of
  personal data, so the prototype's page states that it must hold sample data only; a value the check does recognise
  (an email address list, a card number shape) fails the gate.
- **The Artifact link is shared beyond the product people**: it is private by default and is shared only by a person;
  the run never widens who can see it.
- **The Artifact service is unreachable mid-run**: the stage falls back to the local prototype, says so, and does not
  stop the ladder.
- **The prototype is still live when the feature is abandoned**: retiring it is the maintainer's call; nothing deletes a
  published link without a person asking.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Prototype mode MUST be a per-project setting, off by default and off for every project generated or
  adopted before it existed, so that every existing answer means what it meant; turning it on MUST change nothing but
  adding the prototype stage and the fence.
- **FR-002**: With prototype mode on, the ladder MUST offer a prototype stage after the product specification and before
  the split; the stage MUST be skippable per feature and MUST NOT block the split.
- **FR-003**: The prototype stage MUST build a prototype from the feature's `spec.md` (and the PRD it names, if any), in
  which every user story with an actor and an action is reachable as a journey.
- **FR-004**: Every screen or step of a prototype MUST show a visible banner stating it is a prototype and that nothing
  is saved or sent.
- **FR-005**: A prototype MUST hold only sample data kept inside its own folder, MUST NOT read from or write to any real
  system, and MUST NOT call any network address other than the host serving it.
- **FR-006**: The prototype MUST be published by default as a private Claude Artifact; republishing MUST keep the same
  link; the run MUST record the link and the spec revision it was built from.
- **FR-007**: `make prototype` MUST serve the same prototype locally, printing its address, and the stage MUST fall back
  to it where an Artifact cannot be published.
- **FR-008**: The repository's gate MUST fail when production code imports from the prototype folder, when a
  credential-shaped value appears under it, or when prototype code calls an external network address — naming the file
  in each case.
- **FR-009**: The gate MUST NOT hold prototype code to the production standard (tests, coverage, mutation, lint), and
  the production build, image and deploy MUST exclude the prototype folder.
- **FR-010**: A reaction to the prototype (an Artifact comment, or a message passed on through `/cruise-tell`) MUST be
  handled as a gap in the specification: answered in `decisions.md`, written into `spec.md` where it changes behaviour,
  and replied to on its thread where it came from one.
- **FR-011**: The prototype MUST NOT show behaviour the specification does not describe: a change MUST be written in
  `spec.md` before the prototype reflects it.
- **FR-012**: Under `/cruise`, the prototype stage MUST run with nobody at the wheel by the existing stop table and
  skipper protocol; a decision that answers a product person MUST say who decided it; and a design reference accepted
  by the machine MUST be marked pending human review.
- **FR-013**: A slice with a screen MUST name, in the split, the prototype screens it replaces; its demo stop MUST show
  the accepted reference screen; and on acceptance the prototype MUST mark each replaced screen as replaced.
- **FR-014**: When every prototype screen is replaced, the prototype folder MUST be removed in one commit and the
  published link MUST say the prototype is retired; a published link MUST NOT be deleted unless a person asks.
- **FR-015**: Prototype code MUST NOT be promoted into production code except through a slice of its own built to the
  production standard.
- **FR-016**: Prototype mode MUST work the same way in a generated project and in an adopted repository; in an adopted
  repository the prototype folder's location MUST be recorded in `project.json`, and an existing folder that is not a
  prototype MUST NOT be written into.
- **FR-017**: Turning prototype mode on twice, or re-running the prototype stage on an unchanged specification, MUST
  change nothing, with a test proving the second run is a no-op.
- **FR-018**: The feature MUST ship as experimental under the written exemption in `AGENTS.md`: the command's `--help`,
  the first line of its report, the top of its page under `docs/` and of the page it writes into a project, and its
  changelog lines MUST say so.
- **FR-019**: The change MUST carry a changelog fragment claiming MINOR, saying what prototype mode is, that it is off
  unless turned on, and what turning it on asks of an existing repository.

### Key Entities

- **Prototype**: the clickable, throwaway version of a feature — its source under the prototype folder, its sample data,
  its published link, the spec revision it was last built from, and whether it is live, local-only, or retired.
- **Prototype screen**: one screen or step of a journey; it has a route name, the user story it shows, and a state —
  draft, accepted (by whom), or replaced (by which slice).
- **Reaction**: a product person's comment or message about the prototype; it belongs to a screen where it names one,
  and is resolved by a decision entry and, where behaviour changes, a criterion.
- **Fence**: the rules the gate holds at the prototype folder's boundary — no import across it, no credential inside
  it, no outbound call from it, no inclusion in a production artifact.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From a specification with three user stories, a product person can open and click through a prototype of
  all three within 30 minutes of the maintainer starting the prototype stage, with nothing installed.
- **SC-002**: 100% of production imports of prototype code, credential-shaped values under the prototype folder and
  outbound calls from prototype code planted in the test fixtures fail the gate; 0 fail it when absent.
- **SC-003**: Every reaction left on a prototype is answered — a decision entry and a reply — by the end of the next run
  iteration, and none changes the prototype without a matching change to `spec.md`.
- **SC-004**: Every project generated or adopted before this feature generates and migrates exactly as before, and its
  gate is unchanged, until prototype mode is turned on.
- **SC-005**: In a feature built with prototype mode on, every slice with a screen names its reference prototype
  screens, and its demo is judged against them.
- **SC-006**: No production build, image or deploy artifact of a project with prototype mode on contains any file from
  the prototype folder.

## Assumptions

- **Throwaway by default.** The prototype is evidence for the specification, not the first draft of production;
  production is rebuilt test-first (decided with the requester, 2026-10-01).
- **The Artifact is the default home, localhost the upgrade** (decided with the requester, 2026-10-01). Publishing an
  Artifact needs a harness that can; elsewhere the local prototype is the only home.
- **Both workflows.** Generated projects and adopted repositories get the same feature (decided with the requester,
  2026-10-01). The prototype folder is `prototype/` at the repository root unless `project.json` records another place.
- **The fence is the safety argument.** "No security concerns" holds because of what the gate enforces at the folder's
  boundary, not because of a convention; anything the fence cannot detect (personal data in sample data, for example)
  is stated on the prototype's page as the product people's rule.
- **One prototype per feature.** A feature under `specs/<feature>/` has at most one live prototype; several features
  may each have one.
- **The constitution's security MUSTs are about production code.** The prototype folder sits outside production code by
  construction, which is why the fence, and not the security principles, governs it; nothing in this feature loosens a
  MUST about production code.
- **Experimental** under the `AGENTS.md` exemption until a later MINOR names it stable, so what it offers may change in a
  MINOR while it settles.
- **Out of scope:** hosting the prototype anywhere public, sign-in for product people, analytics on how the prototype is
  used, porting prototype code automatically, and prototypes of features whose specification does not yet exist.
