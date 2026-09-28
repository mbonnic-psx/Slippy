# Where `slipwai` deploys

This project goes to production on infrastructure this factory does not own (`target: existing`): nothing
under `infra/`, no deploy pipeline, no `make deploy`, nothing provisioned. What the factory owes it instead is
this page — where it runs, who describes that, and how a release reaches it — and the release-constraint rung
of `/drive`, which asks every slice how it reaches production and what gates its release.

## The infrastructure

Home: **`here`** (recorded as `detected`). Infrastructure code is in this repository (opentofu / terraform). The factory adds nothing to it and manages none of it: import or reference what exists — `tofu import`, data sources — and never let two places manage one resource.

## How a release reaches production

Recorded: **a pipeline deploys** (`detected`, from `pipeline: .github/workflows/package.yml`, `pipeline: .github/workflows/publish-package.yml`, `pipeline: .github/workflows/release.yml`, `pipeline: .github/workflows/verify.yml`, `scripted: assets/targets/aws/scripts/deploy.py`, `scripted: assets/targets/azure/scripts/deploy.py`, `scripted: tests/fixtures/adopt/converging/deploy.sh`, `scripted: tests/fixtures/adopt/javascript-gitlab/deploy.sh`, `scripted: Makefile`). Write it here, in the order it happens: the artefact (an image, a package, a
WAR), who or what builds it, the environments in order, who applies it, and how it is undone. A release path
nobody can write down is the first thing to fix, because every slice `/drive` finishes ends by asking for it.

## Release constraints

No flag mechanism is installed here — that comes with a managed target — so each slice decides one of three
things and writes it in its plan: releasable on merge, because everything it adds is coherent and safe to an
actor the moment it lands; held behind a toggle this repository already has, named; or a coordinated deploy,
with who does what. `/drive` asks, and `AGENTS.md` asks for the same sentence in the turn that pushes.
