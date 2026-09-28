# Survey

Written by `slipwai adopt` (1.4.0.dev0) from the tree as it was; `/survey` refreshes it. Every line names the
file that said so. Experimental: see `../docs/adoption.md`.

## Builds

- `.` — python, python, from `pyproject.toml`, python 3.11; what it is for, nothing here says

Wrapped as: `slipwai` (`.`).

## Continuous integration

- `.github/workflows`

Proposed forge: `github`, from `.github/workflows`.

## How a change reaches production

- pipeline — `.github/workflows/package.yml`
- pipeline — `.github/workflows/publish-package.yml`
- pipeline — `.github/workflows/release.yml`
- pipeline — `.github/workflows/verify.yml`
- scripted — `assets/targets/aws/scripts/deploy.py`
- scripted — `assets/targets/azure/scripts/deploy.py`
- scripted — `tests/fixtures/adopt/converging/deploy.sh`
- scripted — `tests/fixtures/adopt/javascript-gitlab/deploy.sh`
- scripted — `Makefile`

Proposed: `pipeline`.

## Containers

- `.github/runner/Dockerfile`
- `assets/backing-services/docker-compose.yml`

## Infrastructure as code

- opentofu / terraform — `assets/targets/aws/bootstrap/main.tf`
- opentofu / terraform — `assets/targets/aws/bootstrap/outputs.tf`
- opentofu / terraform — `assets/targets/aws/bootstrap/variables.tf`
- opentofu / terraform — `assets/targets/aws/service/auth0.tf`
- opentofu / terraform — `assets/targets/aws/service/cognito_customers.tf`
- opentofu / terraform — `assets/targets/aws/service/cognito_staff.tf`
- opentofu / terraform — `assets/targets/aws/service/flags.tf`
- opentofu / terraform — `assets/targets/aws/service/frontend.tf`
- opentofu / terraform — `assets/targets/aws/service/ingress.tf`
- opentofu / terraform — `assets/targets/aws/service/main.tf`
- opentofu / terraform — `assets/targets/aws/service/network.tf`
- opentofu / terraform — `assets/targets/aws/service/no-auth0.tf`
- opentofu / terraform — `assets/targets/aws/service/no-frontend.tf`
- opentofu / terraform — `assets/targets/aws/service/outputs.tf`
- opentofu / terraform — `assets/targets/aws/service/rds.tf`
- opentofu / terraform — `assets/targets/aws/service/variables.tf`
- opentofu / terraform — `assets/targets/aws/service/versions.tf`
- opentofu / terraform — `assets/targets/azure/bootstrap/main.tf`
- opentofu / terraform — `assets/targets/azure/bootstrap/outputs.tf`
- opentofu / terraform — `assets/targets/azure/bootstrap/variables.tf`
- opentofu / terraform — `assets/targets/azure/service/auth0.tf`
- opentofu / terraform — `assets/targets/azure/service/entra_staff.tf`
- opentofu / terraform — `assets/targets/azure/service/flags.tf`
- opentofu / terraform — `assets/targets/azure/service/frontend.tf`
- opentofu / terraform — `assets/targets/azure/service/ingress.tf`
- opentofu / terraform — `assets/targets/azure/service/main.tf`
- opentofu / terraform — `assets/targets/azure/service/no-auth0.tf`
- opentofu / terraform — `assets/targets/azure/service/no-frontend.tf`
- opentofu / terraform — `assets/targets/azure/service/outputs.tf`
- opentofu / terraform — `assets/targets/azure/service/postgres.tf`
- opentofu / terraform — `assets/targets/azure/service/variables.tf`
- opentofu / terraform — `assets/targets/azure/service/versions.tf`

Proposed home: `here`.

## Database

Schema tools:

- none found

Drivers in dependency manifests:

- postgres — `assets/languages/java-quarkus/app/pom.xml`
- postgresql — `assets/languages/java-quarkus/app/pom.xml`
- jdbc — `assets/languages/java-quarkus/app/pom.xml`
- postgres — `assets/languages/java-spring/app/pom.xml`
- postgresql — `assets/languages/java-spring/app/pom.xml`
- jdbc — `assets/languages/java-spring/app/pom.xml`
- pgx — `assets/languages/go/modules/postgres/go.mod`

Proposed schema home: `unmanaged`.

## Also here

- root `Makefile`: yes
- `README`: yes

## Big issues that are quick wins

Each is a proposal, not a change the factory made; `/drive` offers them before the map's rows while any remain, a secret first, and `/survey` drops each as the tree stops showing it.

| Kind | Where | What | Fix |
|---|---|---|---|
| `secret-in-tree` | `assets/backing-services/java/database_url.java:10` | a connection string with a password is written in the file | rotate it now — a key in Git history is public — then read it from the environment or a secret store and purge the history (`git filter-repo`) |
| `secret-in-tree` | `assets/backing-services/java/tests/database_url_test.java:19` | a connection string with a password is written in the file | rotate it now — a key in Git history is public — then read it from the environment or a secret store and purge the history (`git filter-repo`) |
| `secret-in-tree` | `tests/test_release.py:295` | a connection string with a password is written in the file | rotate it now — a key in Git history is public — then read it from the environment or a secret store and purge the history (`git filter-repo`) |
| `secret-in-tree` | `tests/test_upgrade.py:187` | a connection string with a password is written in the file | rotate it now — a key in Git history is public — then read it from the environment or a secret store and purge the history (`git filter-repo`) |
| `no-lockfile` | `assets/frontends/react-vite/api-client/package.json` | no lockfile beside `package.json` (package-lock.json / yarn.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
| `no-lockfile` | `assets/frontends/react-vite/app/package.json` | no lockfile beside `package.json` (package-lock.json / yarn.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
| `no-lockfile` | `assets/languages/rust/app/Cargo.toml` | no lockfile beside `Cargo.toml` (Cargo.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
| `no-lockfile` | `assets/languages/rust/repository/Cargo.toml` | no lockfile beside `Cargo.toml` (Cargo.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
| `no-lockfile` | `assets/languages/typescript/app/package.json` | no lockfile beside `package.json` (package-lock.json / yarn.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
| `no-lockfile` | `assets/toolkit/scripts/event-model/package.json` | no lockfile beside `package.json` (package-lock.json / yarn.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
| `no-lockfile` | `tests/fixtures/adopt/converging/apps/shop/package.json` | no lockfile beside `package.json` (package-lock.json / yarn.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
| `no-lockfile` | `tests/fixtures/adopt/javascript-gitlab/package.json` | no lockfile beside `package.json` (package-lock.json / yarn.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
| `no-lockfile` | `tests/fixtures/adopt/javascript-service/package.json` | no lockfile beside `package.json` (package-lock.json / yarn.lock) | install once and commit the lockfile the package manager writes; without it every build resolves versions afresh and no two are the same |
