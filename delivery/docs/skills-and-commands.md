# Skills and commands

Skills under `delivery/skills/` are active working guidance owned by this project. **52 of them shipped
here**, and every one is about something this project can do:

`acceptance-review`, `adversarial-testing`, `api-design`, `architecture-decisions`, `bff-design`, `bff-entry-points`, `characterisation-tests`, `ci-debugging`, `cli-design`, `codebase-design`, `debugging`, `diagrams`, `domain-driven-design`, `double-check`, `evaluate-existing-solutions`, `event-modeling`, `event-sourcing`, `expectations`, `find-gaps`, `find-skills`, `finding-seams`, `folder-structure`, `front-end-testing`, `frontend-design`, `functional`, `global-event-model`, `hexagonal-architecture`, `improve-codebase-architecture`, `mutation-testing`, `observability`, `planning`, `production-parity-skill-builder`, `react-testing`, `reduce-system-complexity`, `refactoring`, `run-the-app`, `secure-oauth-oidc`, `specification`, `stack-pull-requests`, `story-splitting`, `storyboard`, `structure-codebase`, `tdd`, `teach-me`, `technical-writing`, `test-design-reviewer`, `testing`, `twelve-factor`, `typescript-strict`, `ubiquitous-language`, `web-interface-guidelines`, `wtf`

`run-the-app` is written from this project's own answers rather than taken from the catalogue. The rest of
them carry examples in every language this project's services are written in, one labelled block per
language where there are several, and a skill whose examples have not been translated yet says so under its
title rather than being withheld — guidance in the wrong language still reads, and that is a different
question from guidance about something that is not here.

Nothing was withheld: this project has an answer for every capability the catalogue knows how to be about.

`make check-agents` names a skill that is here and no longer justified, so a project that drops its browser
app is told rather than left carrying the pages about one.

Commands are adapted to this repository's  backend toolchain, `none` frontend, and
`standard` workflow:

- `/drive` — `delivery/commands/drive.md`
- `/where-are-we` — `delivery/commands/where-are-we.md`
- `/whats-next` — `delivery/commands/whats-next.md`
- `/gaps` — `delivery/commands/gaps.md`
- `/adversary` — `delivery/commands/adversary.md`
- `/mutation` — `delivery/commands/mutation.md`
- `/constitution-coverage` — `delivery/commands/constitution-coverage.md`
- `/model-delegation-settings` — `delivery/commands/model-delegation-settings.md`
- `/drive-settings` — `delivery/commands/drive-settings.md`
- `/benchmark` — `delivery/commands/benchmark.md`
- `/cruise` — `delivery/commands/cruise.md`
- `/cruise-settings` — `delivery/commands/cruise-settings.md`
- `/cruise-status` — `delivery/commands/cruise-status.md`
- `/cruise-stop` — `delivery/commands/cruise-stop.md`
- `/cruise-tell` — `delivery/commands/cruise-tell.md`
- `/cruise-watch` — `delivery/commands/cruise-watch.md`
- `/add-service` — `delivery/commands/add-service.md`
- `/add-frontend` — `delivery/commands/add-frontend.md`
- `/catch-up` — `delivery/commands/catch-up.md`

Agent types under `delivery/agents/` are the fifth thing projected. Each stage `/drive` sends to a fresh context has
one — `drive-gaps`, `drive-tasks`, `drive-implement`, `drive-converge`, `drive-adversary`, `drive-mutation`, and
`drive-slice` for a whole slice — carrying that stage's standing brief, the model `.specify/models.json` resolves for it, and what its delegate may write and
run. Three more, `drive-skipper`, `drive-hand` and `drive-bosun`, are `/cruise`'s product owner, actor and
unblocker, met only when that command runs the ladder on its own. `make agents` renders each into the installed harness's own agent file so the harness holds the scope
it can hold; `delivery/docs/agent-harnesses.md` says what each one can.

Run `./delivery/init --integration <agent>` to install Spec Kit's scripts, templates, workflow, and agent commands.
Those generated files are owned by Spec Kit rather than the scaffolding factory. After native Spec Kit
initialization, `./delivery/init` projects the project-owned catalogue into the selected agent's native locations.
For example, Cursor receives skills and command wrappers under `.cursor/skills/`, while Codex receives them
under `.agents/skills/`. Keep editing the canonical `delivery/skills/`, `delivery/commands/` and `delivery/agents/` files and rerun
`./delivery/init` to refresh the agent projections.
