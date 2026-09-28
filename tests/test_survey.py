"""The survey `slipwai adopt` starts from: what an existing repository's tree says about itself.

Every fact here is a proposal the person confirms, so the properties worth gating are that each is *read*
rather than guessed — the language from the manifest that starts the build, the commands from that
ecosystem's own tools and only where its configuration says they apply, the toolchain pin from the file
that carries it — that each names its evidence, that a build owning the builds beneath it is one root and
not several, and that what cannot be read is reported absent: a written no for a target the ecosystem has
no answer to, `none` for a schema nothing here versions and no driver talks to.

The fixtures are written by the tests, in the shape real repositories have, because a description of a
tree is not a tree.
"""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from slipwai.ecosystems import TARGETS
from slipwai.survey import Root, buildable, survey


def write(root: Path, files: dict[str, str]) -> Path:
    for relative, content in files.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
    return root


class SurveyTest(unittest.TestCase):
    def only(self, roots: tuple[Root, ...]) -> Root:
        self.assertEqual(len(roots), 1, f"exactly one candidate expected, got {[r.path for r in roots]}")
        return roots[0]

    def test_an_empty_tree_says_so(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            found = survey(Path(directory))
            self.assertEqual(found.roots, ())
            self.assertEqual(found.schema_home, "none")
            self.assertEqual(found.infrastructure_home, "unmanaged")
            self.assertFalse(found.git)
            self.assertFalse(found.makefile)

    def test_a_typescript_repository_is_read_from_its_package_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "package.json": json.dumps({
                    "name": "shop", "scripts": {"lint": "eslint .", "test": "vitest run", "build": "tsc"},
                    "devDependencies": {"typescript": "^5", "vitest": "^2"}, "dependencies": {"pg": "^8"},
                    "engines": {"node": ">=20"},
                }),
                "package-lock.json": "{}", "tsconfig.json": "{}", ".nvmrc": "v20.11\n",
                "Dockerfile": "FROM node:20\n", ".github/workflows/ci.yml": "on: push\n",
                "infra/main.tf": 'resource "aws_s3_bucket" "b" {}\n', "README.md": "# shop\n", "Makefile": "all:\n",
                "prisma/schema.prisma": "datasource db {}\n",
            })
            (root / ".git").mkdir()
            found = survey(root)
            self.assertEqual(len(found.roots), 1)
            shop = found.roots[0].found
            self.assertEqual((found.roots[0].path, shop.ecosystem, shop.language), (".", "node", "typescript"))
            self.assertEqual(shop.evidence, "package.json")
            self.assertEqual(tuple(shop.commands), TARGETS, "every target is answered, in the Makefile's order")
            self.assertEqual(shop.commands["install"], "npm ci")
            self.assertEqual(shop.commands["lint"], "npm run lint")
            self.assertEqual(shop.commands["test"], "npm run test")
            self.assertEqual(
                shop.commands["typecheck"], "npm exec -- tsc --noEmit --skipLibCheck",
                "TypeScript with no script of its own gets tsc — and `--skipLibCheck`, because a bare run is "
                "red on dependencies that ship disagreeing types, which the project cannot fix and the "
                "ratchet would baseline as a blanket excuse for the whole check",
            )
            self.assertEqual(shop.commands["audit"], "npm audit --audit-level=critical")
            for target in ("integration", "adversarial", "mutation"):
                self.assertIsNone(shop.commands[target], f"{target} has no answer here, and none is invented")
            self.assertEqual(shop.toolchain, {"kind": "node", "version": "20.11"}, ".nvmrc wins over engines")
            self.assertEqual(found.ci, (".github/workflows",))
            self.assertEqual(found.containers, ("Dockerfile",))
            self.assertEqual(found.infrastructure, (("opentofu / terraform", "infra/main.tf"),))
            self.assertEqual(found.schema_tools, (("prisma", "prisma/schema.prisma"),))
            self.assertIn(("pg", "package.json"), found.drivers)
            self.assertEqual((found.schema_home, found.infrastructure_home), ("here", "here"))
            self.assertTrue(found.git and found.makefile and found.readme)
            self.assertEqual(found.roots[0].name("Shop Front"), "shop-front")

    def test_a_python_service_answers_only_what_its_configuration_says(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "pyproject.toml": (
                    '[project]\nname = "ledger"\nrequires-python = ">=3.12"\n'
                    '[tool.ruff]\nline-length = 100\n[tool.mypy]\nstrict = true\n'
                ),
                "requirements.txt": "flask==3.0\npsycopg[binary]==3.1\n",
                "tests/test_x.py": "def test_x():\n    pass\n",
                "alembic.ini": "[alembic]\n",
            })
            found = survey(root)
            ledger = found.roots[0].found
            self.assertEqual(
                (ledger.ecosystem, ledger.language, ledger.evidence), ("python", "python", "pyproject.toml")
            )
            self.assertEqual(ledger.commands["install"], "python3 -m pip install -r requirements.txt")
            self.assertEqual(ledger.commands["lint"], "python3 -m ruff check .")
            self.assertEqual(ledger.commands["typecheck"], "python3 -m mypy .")
            self.assertEqual(ledger.commands["test"], "python3 -m pytest")
            self.assertEqual(ledger.commands["audit"], "pip-audit -r requirements.txt")
            self.assertEqual(ledger.toolchain, {"kind": "python", "version": "3.12"})
            self.assertEqual(found.schema_tools, (("alembic", "alembic.ini"),))
            self.assertIn(("psycopg", "requirements.txt"), found.drivers)
            # Without a linter configured there is no lint command, not a guessed one.
            bare = write(Path(directory) / "bare", {"setup.py": "from setuptools import setup\nsetup()\n"})
            plain = survey(bare).roots[0].found
            self.assertIsNone(plain.commands["lint"])
            self.assertIsNone(plain.commands["typecheck"])
            self.assertIsNone(plain.commands["test"])
            self.assertEqual(plain.commands["install"], "python3 -m pip install -e .")

    def test_a_maven_war_names_its_packaging_and_the_tools_its_pom_declares(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "pom.xml": (
                    "<project><packaging>war</packaging><properties><maven.compiler.release>17"
                    "</maven.compiler.release></properties><build><plugins>"
                    "<plugin><artifactId>maven-checkstyle-plugin</artifactId></plugin>"
                    "<plugin><artifactId>maven-failsafe-plugin</artifactId></plugin></plugins></build></project>"
                ),
                "mvnw": "#!/bin/sh\n",
                "src/main/resources/db/migration/V1__init.sql": "create table t (id int);\n",
            })
            found = survey(root)
            legacy = found.roots[0].found
            self.assertEqual((legacy.ecosystem, legacy.language, legacy.packaging), ("maven", "java", "war"))
            self.assertEqual(legacy.commands["test"], "./mvnw -B -q test")
            self.assertEqual(legacy.commands["lint"], "./mvnw -B -q -DskipTests checkstyle:check")
            self.assertEqual(legacy.commands["integration"], "./mvnw -B -q failsafe:integration-test failsafe:verify")
            self.assertIsNone(legacy.commands["adversarial"], "no JUnit 5 declared: `groups` would be a Category")
            jupiter = write(Path(directory) / "jupiter", {
                "pom.xml": "<project><dependencies><dependency><artifactId>junit-jupiter</artifactId></dependency>"
                           "</dependencies></project>",
            })
            self.assertEqual(survey(jupiter).roots[0].found.commands["adversarial"],
                             "./mvnw -B -q test -Dgroups=adversarial -DfailIfNoTests=false",
                             "the recipe every generated Java service runs, once the pom declares JUnit 5")
            # The wrapper form whether or not the wrapper is there: `adopt` writes it where it is missing.
            bare = write(Path(directory) / "bare", {"pom.xml": "<project/>"})
            self.assertEqual(survey(bare).roots[0].found.commands["test"], "./mvnw -B -q test")
            nested = write(Path(directory) / "nested", {"services/api/build.gradle": "plugins { id 'java' }\n"})
            self.assertEqual(
                survey(nested).roots[0].found.commands["test"], "cd services/api && ./gradlew -q test"
            )
            self.assertIsNone(legacy.commands["audit"])
            self.assertEqual(legacy.toolchain, {"kind": "java", "version": "17"})
            self.assertEqual(found.schema_tools, (("flyway", "src/main/resources/db/migration/V1__init.sql"),))
            self.assertEqual(found.schema_home, "here")

    def test_a_dotnet_solution_is_one_root_and_a_bare_project_gives_its_framework(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "Ledger.sln": "Microsoft Visual Studio Solution File\n",
                "global.json": '{"sdk": {"version": "8.0.100"}}',
                "src/Ledger.Api/Ledger.Api.csproj": (
                    "<Project><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>"
                ),
                "src/Ledger.Api/Migrations/20240101_Init.Designer.cs": "// ef\n",
                "src/Ledger.Tests/Ledger.Tests.csproj": "<Project></Project>",
            })
            found = survey(root)
            self.assertEqual([root.path for root in found.roots], ["."], "the solution owns the projects under it")
            ledger = found.roots[0].found
            self.assertEqual(
                (ledger.ecosystem, ledger.language, ledger.evidence), ("dotnet", "csharp", "Ledger.sln")
            )
            self.assertEqual(ledger.commands["test"], "dotnet test Ledger.sln --no-build")
            self.assertEqual(ledger.commands["lint"], "dotnet format Ledger.sln --verify-no-changes")
            self.assertEqual(ledger.toolchain, {"kind": "dotnet", "version": "8.0.100"})
            self.assertEqual(found.schema_tools[0][0], "entity framework")
            bare = write(Path(directory) / "bare", {
                "Api.csproj": (
                    "<Project><PropertyGroup><TargetFramework>net6.0</TargetFramework></PropertyGroup></Project>"
                ),
            })
            self.assertEqual(survey(bare).roots[0].found.toolchain, {"kind": "dotnet", "version": "6.0"})

    def test_a_repository_with_several_builds_reports_each_once_and_none_inside_an_owner(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "package.json": json.dumps({"workspaces": ["web/*"], "scripts": {"test": "npm run test --workspaces"}}),
                "pnpm-lock.yaml": "", "web/admin/package.json": json.dumps({"name": "admin"}),
                "services/billing/go.mod": "module billing\n\ngo 1.22\n",
                "services/reports/composer.json": json.dumps({"require": {"php": "^8.2", "doctrine/orm": "^3"}}),
                "services/reports/phpunit.xml": "<phpunit/>",
                "legacy/Gemfile": "source 'https://rubygems.org'\n", "legacy/.ruby-version": "3.3.0\n",
                "legacy/spec/a_spec.rb": "", "legacy/.rubocop.yml": "",
                "tools/build.gradle.kts": (
                    'plugins { kotlin("jvm") }\n'
                    'java { toolchain { languageVersion.set(JavaLanguageVersion.of(21)) } }\n'
                ),
                "tools/gradlew": "#!/bin/sh\n", "tools/settings.gradle.kts": "",
                "node_modules/left/package.json": "{}", "vendor/x/composer.json": "{}",
            })
            found = survey(root)
            by_path = {root.path: root.found for root in found.roots}
            self.assertEqual(
                list(by_path), [".", "legacy", "services/billing", "services/reports", "tools"],
                "the workspace owns web/admin; dependency directories are never surveyed",
            )
            self.assertEqual(by_path["."].commands["install"], "pnpm install --frozen-lockfile")
            self.assertEqual(by_path["."].commands["test"], "pnpm run test")
            self.assertEqual(by_path["services/billing"].commands["test"], "cd services/billing && go test ./...")
            self.assertEqual(by_path["services/billing"].toolchain, {"kind": "go", "version": "1.22"})
            reports = by_path["services/reports"]
            self.assertEqual(reports.language, "php")
            self.assertEqual(
                reports.commands["install"], "composer install --no-interaction --working-dir=services/reports"
            )
            self.assertEqual(reports.commands["test"], "cd services/reports && vendor/bin/phpunit")
            self.assertEqual(reports.toolchain, {"kind": "php", "version": "8.2"})
            legacy = by_path["legacy"]
            self.assertEqual((legacy.language, legacy.toolchain["version"]), ("ruby", "3.3.0"))
            self.assertEqual(legacy.commands["test"], "cd legacy && bundle exec rspec")
            self.assertEqual(legacy.commands["lint"], "cd legacy && bundle exec rubocop")
            tools = by_path["tools"]
            self.assertEqual((tools.ecosystem, tools.language), ("gradle", "kotlin"))
            self.assertEqual(tools.commands["test"], "cd tools && ./gradlew -q test")
            self.assertEqual(tools.toolchain, {"kind": "java", "version": "21"})
            self.assertEqual(found.languages, ["javascript", "ruby", "go", "php", "kotlin"])
            self.assertIn(("doctrine", "services/reports/composer.json"), found.drivers)
            self.assertEqual(
                found.schema_home, "unmanaged", "a driver and no schema tool: talked to, versioned elsewhere or nowhere"
            )
            self.assertEqual(Root("services/billing", by_path["services/billing"]).name("x"), "billing")

    def test_a_directory_holding_a_cargo_toml_is_proposed_as_rust_built_by_cargo_with_its_commands(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "Cargo.toml": '[package]\nname = "ledger"\nversion = "0.1.0"\nedition = "2021"\n',
                "Cargo.lock": "", "src/lib.rs": "",
                "rust-toolchain.toml": '[toolchain]\nchannel = "1.79.0"\n',
            })
            found = survey(root)
            self.assertEqual([r.path for r in found.roots], ["."], "one crate is one candidate")
            crate = found.roots[0].found
            self.assertEqual((crate.language, crate.ecosystem, crate.evidence), ("rust", "cargo", "Cargo.toml"))
            self.assertEqual(crate.commands["install"], "cargo fetch --locked")
            self.assertEqual(crate.commands["typecheck"], "cargo check --all-targets")
            self.assertEqual(crate.commands["lint"], "cargo clippy --all-targets -- -D warnings && cargo fmt --check")
            self.assertEqual(crate.commands["test"], "cargo test")
            for target in ("integration", "adversarial", "audit", "mutation"):
                self.assertIn(target, crate.commands)
                self.assertIsNone(crate.commands[target], f"{target} is a written no")
            self.assertEqual(crate.toolchain, {"kind": "rust", "version": ""}, "a pin in the tree is a later slice's")
            self.assertIsNone(crate.packaging)
            self.assertIn("rust", found.languages)
            self.assertIsNone(found.roots[0].role, "nothing beside the crate says what it is for")

    def test_a_crate_in_a_subdirectory_has_every_command_prefixed_once(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"crates/ledger/Cargo.toml": '[package]\nname = "ledger"\n'})
            found = self.only(survey(root).roots)
            self.assertEqual((found.path, found.found.evidence), ("crates/ledger", "crates/ledger/Cargo.toml"))
            commands = found.found.commands
            self.assertEqual(commands["install"], "cd crates/ledger && cargo fetch --locked")
            self.assertEqual(commands["typecheck"], "cd crates/ledger && cargo check --all-targets")
            self.assertEqual(
                commands["lint"], "cd crates/ledger && cargo clippy --all-targets -- -D warnings && cargo fmt --check"
            )
            self.assertEqual(commands["test"], "cd crates/ledger && cargo test")

    def test_a_malformed_cargo_toml_and_a_missing_lockfile_are_still_cargo(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"Cargo.toml": "[package\nname = \n"})
            found = self.only(survey(root).roots)
            self.assertEqual(found.found.ecosystem, "cargo")
            self.assertEqual(found.found.commands["install"], "cargo fetch --locked")

    def test_a_cargo_toml_in_a_directory_that_is_never_surveyed_is_not_proposed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "target/debug/build/x/Cargo.toml": "[package]\n", "vendor/dep/Cargo.toml": "[package]\n",
                "fixtures/ledger/Cargo.toml": "[package]\n",
            })
            self.assertEqual(buildable(root, frozenset({"fixtures/ledger"})), ())

    def test_a_package_json_beside_a_cargo_toml_is_reported_once_as_node(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "Cargo.toml": '[package]\nname = "native"\n', "package.json": json.dumps({"name": "native"}),
            })
            found = self.only(survey(root).roots)
            self.assertEqual((found.found.ecosystem, found.found.evidence), ("node", "package.json"))

    def test_a_fuzz_crate_under_a_root_crate_is_a_candidate_of_its_own(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "Cargo.toml": '[package]\nname = "ledger"\n', "fuzz/Cargo.toml": '[package]\nname = "ledger-fuzz"\n',
            })
            found = survey(root)
            self.assertEqual([r.path for r in found.roots], [".", "fuzz"])
            self.assertEqual(found.roots[1].found.commands["test"], "cd fuzz && cargo test")

    def test_a_dockerfile_beside_a_crate_makes_it_a_service(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {"Cargo.toml": '[package]\nname = "api"\n', "Dockerfile": "FROM rust\n"})
            found = self.only(survey(root).roots)
            self.assertEqual((found.role, found.role_evidence), ("service", "Dockerfile"))

    def test_the_forge_the_release_path_and_what_a_directory_is_are_read_only_where_a_file_says(self) -> None:
        """Three facts adoption used to assume: which forge runs CI, how a change reaches production, and what
        each buildable directory is for. Each is proposed from a file that says so, with the file, and where
        no file says, nothing is proposed — the record then says `unrecorded`, never a default."""
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "package.json": json.dumps({
                    "name": "orders", "private": True,
                    "scripts": {"start": "node src/server.js", "test": "node --test"},
                }),
                "src/server.js": "",
                ".gitlab-ci.yml": "stages: [test, deploy]\ndeploy:\n  script: [./deploy.sh]\n",
                "deploy.sh": "#!/bin/sh\n",
                "Makefile": "deploy:\n\t@echo ship\n",
                "tools/cli/package.json": json.dumps({"name": "cli", "bin": {"cli": "index.js"}}),
                "packages/money/package.json": json.dumps({"name": "money", "main": "index.js"}),
                "e2e/package.json": json.dumps({"name": "e2e", "scripts": {"test": "node --test"}}),
                "svc/go.mod": "module example.com/svc\n\ngo 1.24\n", "svc/main.go": "package main\n",
            })
            found = survey(root)
            by_path = {root_.path: root_ for root_ in found.roots}
            self.assertEqual(found.forge, ("gitlab", ".gitlab-ci.yml"))
            self.assertEqual(found.release_path, "pipeline", "a CI job that deploys outranks a script somebody runs")
            self.assertEqual(
                set(found.release_evidence),
                {("pipeline", ".gitlab-ci.yml"), ("scripted", "deploy.sh"), ("scripted", "Makefile")},
            )
            self.assertEqual((by_path["."].role, by_path["."].role_evidence), ("service", "package.json"))
            self.assertEqual(by_path["tools/cli"].role, "tool")
            self.assertEqual(by_path["packages/money"].role, "library")
            self.assertEqual(by_path["e2e"].role, "tests")
            self.assertEqual((by_path["svc"].role, by_path["svc"].role_evidence), ("service", "svc/main.go"))
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "package.json": json.dumps({"name": "x", "private": True}),
                ".git/config": (
                    '[core]\n\tbare = false\n[remote "origin"]\n\turl = git@github.com:acme/x.git\n'
                    "\tfetch = +refs/heads/*:refs/remotes/origin/*\n"
                ),
            })
            found = survey(root)
            self.assertEqual(found.forge, ("github", "remote origin at github.com"), "no CI file: the remote's host")
            self.assertIsNone(found.release_path, "nothing says how a change reaches production: nothing proposed")
            self.assertEqual(found.release_evidence, ())
            self.assertIsNone(found.roots[0].role, "a package.json with no start, bin or main says nothing of its role")
            bare = write(Path(directory) / "bare", {"package.json": "{}"})
            self.assertEqual(survey(bare).forge, ("none", "no CI configuration in the tree"))

    def test_an_ant_build_is_read_from_its_build_xml_and_the_jars_it_commits(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            targets = ("-init", "compile", "compile-test", "test", "jar", "clean")
            impl = "".join(f'<target name="{t}"/>' for t in targets)
            root = write(Path(directory), {
                "build.xml": '<project name="H" default="default"><import file="nbproject/build-impl.xml"/></project>',
                "nbproject/build-impl.xml": f"<project>{impl}</project>",
                "nbproject/project.properties": "javac.source=1.8\njavac.target=1.8\nsrc.dir=src\ntest.src.dir=test\n",
                "nbproject/project.xml": "<project><type>org.netbeans.modules.java.j2seproject</type></project>",
                "lib/mysql-connector-java-5.1.23-bin.jar": "PK", "src/Login.java": "class Login {}\n",
            })
            found = survey(root)
            legacy = found.roots[0].found
            self.assertEqual((legacy.ecosystem, legacy.language, legacy.evidence), ("ant", "java", "build.xml"))
            self.assertEqual(legacy.toolchain, {"kind": "java", "version": "8"})
            self.assertEqual(legacy.commands["typecheck"], "ant -q compile")
            self.assertIsNone(legacy.commands["test"], "a test target with no test directory is a written no")
            self.assertIsNone(legacy.commands["install"], "the jars are committed; nothing fetches them")
            self.assertIsNone(legacy.packaging)
            self.assertEqual(found.drivers, (("mysql", "lib/mysql-connector-java-5.1.23-bin.jar"),))
            # The jars are the build's classpath, not an archive tracked by mistake: no quick win says so.
            self.assertEqual([w["kind"] for w in found.quick_wins], [])
            # With a test directory the target is recorded; a NetBeans web project packages a WAR.
            write(root, {"test/LoginTest.java": "class LoginTest {}\n",
                         "nbproject/project.xml": "<project><type>org.netbeans.modules.web.project</type></project>"})
            again = survey(root).roots[0].found
            self.assertEqual((again.commands["test"], again.packaging), ("ant -q test", "war"))
            # A `pom.xml` beside a leftover `build.xml` is a Maven build.
            write(root, {"pom.xml": "<project/>"})
            self.assertEqual(survey(root).roots[0].found.ecosystem, "maven")
