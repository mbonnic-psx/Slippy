"""Big issues that are quick wins: what the survey sees in a tree that is cheap to fix and expensive to leave.

What this gates is that each finding is read off the tree — a secret shaped like one or keyed like one, IDE and
build output tracked, a dependency source over plain HTTP, a lockfile the package manager would write, an archive
tracked — with the file that shows it and the fix, and never the value; that a reference is not a secret; that
the survey carries them, the record and the page say them, the report says them first, and a refresh drops what
the tree stops showing. Experimental, with the rest of adoption (experimental).
"""
from __future__ import annotations

import json
import os
import resource
import subprocess
import tempfile
import unittest
from pathlib import Path

from test_adopt import git, repository, slipwai
from test_survey import write

from slipwai.assets import ROOT
from slipwai.ecosystems import read
from slipwai.quick_wins import KINDS, missing_lockfiles, quick_wins
from slipwai.survey import survey

# What a hostile tree's reads may cost, held by the kernel: were a device read without end again, the run fails
# here with a MemoryError instead of the host's OOM killer taking every other process with it (issue #13).
CEILING = 2 * 1024 ** 3
KEY = "SG." + "a" * 22 + "." + "b" * 43
FILES = {
    "pom.xml": "<project><repositories><repository><id>x</id><url>http://repo.example.com/m2</url></repository>"
               "</repositories><url>http://example.com</url></project>\n",
    "src/main/java/Mail.java": f'class Mail {{ static final String KEY = "{KEY}"; String password = password; }}\n',
    "src/main/webapp/WEB-INF/spring-security.xml": '<user name="admin" password="pass" authorities="ROLE_ADMIN" />\n',
    # Spring's bean XML: the key in one attribute, the value in the next, or in a child — the form the second real
    # adoption's data source used, which the keyed pattern walked past.
    "src/main/webapp/WEB-INF/mvc-dispatcher-servlet.xml": (
        '<bean id="dataSource">\n  <property name="username" value="root"/>\n'
        '  <property name="password" value="s3cretpw"/>\n  <property name="url"><value>jdbc:x</value></property>\n'
        '  <property name="password"><value>${DB_PASSWORD}</value></property>\n</bean>\n'
    ),
    "src/main/resources/db.properties": "db.url=jdbc:mysql://localhost/shop\ndb.password=${DB_PASSWORD}\n",
    "web/package.json": '{"name": "web", "private": true, "scripts": {"test": "node --test"}}\n',
    ".idea/workspace.xml": "<project/>\n", "Shop.iml": "<module/>\n", "lib/legacy.jar": "PK\x03\x04",
    "mvnw": "MVNW_PASSWORD='' ;;\n",
    "tools/$(id)/go.mod": "module x\n",
}


class QuickWinsTest(unittest.TestCase):
    def test_each_kind_is_read_off_the_tree_with_the_file_and_the_fix_and_never_the_value(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "shop", FILES)
            found = {(f.kind, f.where): f for f in quick_wins(repo, [], set(), "delivery")}
            self.assertEqual({k for k, _ in found}, set(KINDS), "every kind fires once on this tree")
            secrets = {where: f for (kind, where), f in found.items() if kind == "secret-in-tree"}
            security = "src/main/webapp/WEB-INF/spring-security.xml:1"
            datasource = "src/main/webapp/WEB-INF/mvc-dispatcher-servlet.xml:3"
            self.assertEqual(set(secrets), {"src/main/java/Mail.java:1", security, datasource},
                             "the property form is read; its placeholder form on line 5 is not a secret")
            self.assertIn("`password` has a literal value", secrets[datasource].what)
            self.assertIn("a SendGrid key is written", secrets["src/main/java/Mail.java:1"].what)
            self.assertIn("`password` has a literal value", secrets[security].what)
            self.assertTrue(all("rotate" in f.fix for f in secrets.values()))
            everything = json.dumps([f.record() for f in found.values()])
            self.assertNotIn(KEY, everything, "the value is never repeated")
            self.assertNotIn("pass\"", everything)
            self.assertNotIn("s3cretpw", everything)
            # A placeholder, a reference and the wrapper's own empty default are not secrets.
            self.assertNotIn(("secret-in-tree", "src/main/resources/db.properties:2"), found)
            self.assertNotIn(("secret-in-tree", "mvnw:1"), found)
            noise = [f for (kind, _), f in found.items() if kind == "ide-or-build-output-tracked"]
            self.assertEqual({f.where for f in noise}, {".idea/workspace.xml", "Shop.iml"})
            self.assertIn("`.gitignore`", noise[0].fix)
            self.assertEqual(found[("insecure-dependency-source", "pom.xml")].what,
                             "a dependency repository is fetched over plain HTTP")
            self.assertIn("no lockfile beside `package.json`", found["no-lockfile", "web/package.json"].what)
            self.assertEqual(found["archive-tracked", "lib/legacy.jar"].what,
                             "1 binary archive(s) under version control")
            self.assertIn("would read as code", found["unsafe-path", "tools/$(id)"].what)

            # The survey carries them; a tree with nothing has none.
            self.assertEqual(len(survey(repo).quick_wins), len(found))
            clean = repository(Path(directory), "clean", {"package.json": '{"name": "c", "private": true}\n',
                                                         "package-lock.json": "{}\n"})
            self.assertEqual(quick_wins(clean, [], set(), "delivery"), [])

    def test_adopt_says_them_first_the_page_carries_them_and_a_refresh_drops_what_is_fixed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "shop", {
                "package.json": '{"name": "shop", "private": true, "scripts": {"test": "node --test"}}\n',
                "test/a.test.js": "test('a', () => {});\n", ".idea/x.xml": "<project/>\n",
            })
            result = slipwai(repo, "adopt", "--yes")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("Quick wins: 2 big issue(s) that are cheap to fix — ide-or-build-output-tracked at "
                          ".idea/x.xml; no-lockfile at package.json", result.stdout)
            document = json.loads((repo / "project.json").read_text())
            self.assertEqual([w["kind"] for w in document["survey"]["quickWins"]],
                             ["ide-or-build-output-tracked", "no-lockfile"])
            page = (repo / "delivery/survey/survey.md").read_text()
            self.assertIn("## Big issues that are quick wins", page)
            self.assertIn("| `no-lockfile` | `package.json` |", page)
            essay = (repo / "delivery/docs/change-strategy.md").read_text()
            self.assertIn("### The programme", essay)
            self.assertIn("| 1 | IntelliJ's `.idea/` is under version control —", essay)
            self.assertIn("now — a slice of its own whatever the strategy", essay)
            drive = (repo / "delivery/commands/drive.md").read_text()
            self.assertIn("from the programme `delivery/docs/change-strategy.md`", drive)
            # Fixed in the tree, gone from the record and the page after /survey — nothing to tick.
            subprocess.run(["git", "rm", "-rq", ".idea"], cwd=repo, check=True, capture_output=True)
            (repo / "package-lock.json").write_text("{}\n")
            subprocess.run(["git", "add", "-A"], cwd=repo, check=True)
            subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@local", "commit", "-qm", "fixed"], cwd=repo,
                           check=True)
            refreshed = slipwai(repo, "adopt", "--refresh")
            self.assertEqual(refreshed.returncode, 0, refreshed.stderr)
            self.assertIn("quick wins: 2 fixed since the last survey; 0 remain", refreshed.stdout)
            document = json.loads((repo / "project.json").read_text())
            self.assertEqual(document["survey"]["quickWins"], [])
            self.assertIn("None the survey can see", (repo / "delivery/survey/survey.md").read_text())

    def test_the_jars_under_an_ant_build_are_its_classpath_and_not_a_quick_win(self) -> None:
        from slipwai.quick_wins import archives_tracked
        ant = ["build.xml", "lib/mysql-connector-java-5.1.23-bin.jar", "jcalendar-1.4.jar", "src/Login.java"]
        self.assertEqual(archives_tracked(ant), [])
        nested = ["apps/desk/build.xml", "apps/desk/lib/a.jar", "apps/web/lib/b.jar", "vendor.zip"]
        self.assertEqual([f.where for f in archives_tracked(nested)], ["apps/web/lib/b.jar (+1 more)"])


if __name__ == "__main__":
    unittest.main()


@unittest.skipUnless(Path("/dev/zero").exists(), "needs a device that never ends")
class HostileFilesTest(unittest.TestCase):
    """A tracked symlink to `/dev/zero` reports a size of 0 and never ends: on 2026-09-28 the secrets scan passed
    it as empty, read it until the machine ran out of memory, and took WSL down with it (issue #13)."""

    def test_read_says_nothing_for_what_is_not_a_regular_file_of_bounded_size(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            here = Path(directory)
            (here / "zero").symlink_to("/dev/zero")
            (here / "gone").symlink_to(here / "nowhere")
            os.mkfifo(here / "pipe")
            (here / "room").mkdir()
            (here / "door.xml").symlink_to(here / "room")
            (here / "big.txt").write_text("x" * 11)
            (here / "ok.txt").write_text("password = hunter22\n")
            self.addCleanup(resource.setrlimit, resource.RLIMIT_AS, resource.getrlimit(resource.RLIMIT_AS))
            mapped = int(Path("/proc/self/statm").read_text().split()[0]) * resource.getpagesize()
            resource.setrlimit(resource.RLIMIT_AS, (mapped + CEILING, resource.getrlimit(resource.RLIMIT_AS)[1]))
            self.assertEqual(read(here / "zero"), "", "a device is not read, however small it says it is")
            self.assertEqual(read(here / "gone"), "")
            self.assertEqual(read(here / "pipe"), "", "a FIFO nobody writes to neither blocks the open nor the read")
            self.assertEqual(read(here / "door.xml"), "", "a link to a directory is no text either")
            self.assertEqual(read(here / "big.txt", 10), "", "one byte past the limit is too big")
            self.assertEqual(read(here / "big.txt", 11), "x" * 11)
            self.assertEqual(read(here / "ok.txt"), "password = hunter22\n")

    def test_adopt_survives_tracked_symlinks_to_a_device_that_never_ends(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "hostile", {
                "package.json": '{"name": "shop", "private": true, "scripts": {"test": "node --test"}}\n',
                "package-lock.json": "{}\n", "test/a.test.js": "test('a', () => {});\n",
            })
            # A config suffix, a source suffix and a manifest name — each reached by a different read — and a
            # broken link, whose `stat()` the old size check raised on.
            for link in ("devz/Cargo.toml", ".env", "src/Main.java", "config/app.yml"):
                (repo / link).parent.mkdir(parents=True, exist_ok=True)
                (repo / link).symlink_to("/dev/zero")
            (repo / "config/old.properties").symlink_to(repo / "config/removed.properties")
            git(repo, "add", "-A")
            git(repo, "-c", "user.name=t", "-c", "user.email=t@local", "commit", "-q", "-m", "hostile")
            result = subprocess.run(
                [str(ROOT / "slipwai"), "adopt", "--yes"], cwd=repo, text=True, capture_output=True,
                stdin=subprocess.DEVNULL, timeout=300,
                preexec_fn=lambda: resource.setrlimit(resource.RLIMIT_AS, (CEILING, CEILING)),
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn("secret-in-tree", result.stdout)

    def test_a_directory_named_to_be_read_as_code_is_not_surveyed_and_is_said(self) -> None:
        """A directory name reaches `cd <dir> && …` and `delivery/Makefile`: a newline there ends the line and makes
        `$(shell …)` top-level make, run while make parses; a `;` or `$(…)` reaches `sh -c`. Such a directory is
        never proposed, and the report names it escaped (GHSA-3fpx-wg55-c4qj)."""
        newline, semicolon = "a\n$(shell touch PWNED_MAKE)\n#", "x;touch PWNED_SH;y"
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "hostile", {
                "package.json": '{"name": "shop", "private": true, "scripts": {"test": "node --test"}}\n',
                "package-lock.json": "{}\n", "test/a.test.js": "test('a', () => {});\n",
                f"{newline}/go.mod": "module a\n\ngo 1.22\n", f"{semicolon}/go.mod": "module x\n\ngo 1.22\n",
                "sp ace/go.mod": "module s\n\ngo 1.22\n", "ok-name_1.2/go.mod": "module ok\n\ngo 1.22\n",
            })
            self.assertEqual([r.path for r in survey(repo).roots], [".", "ok-name_1.2"])
            result = slipwai(repo, "adopt", "--yes")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("unsafe-path", result.stdout)
            wins = json.loads((repo / "project.json").read_text())["survey"]["quickWins"]
            unsafe = [w for w in wins if w["kind"] == "unsafe-path"]
            self.assertEqual(len(unsafe), 1)
            self.assertNotIn("\n", unsafe[0]["where"], "the name is written escaped, never as it is")
            makefile = (repo / "delivery/Makefile").read_text()
            self.assertNotIn("PWNED", makefile)
            self.assertNotIn("sp ace", makefile)
            make = subprocess.run(["make", "-f", "delivery/Makefile", "help"], cwd=repo, capture_output=True,
                                  text=True, timeout=120)
            self.assertEqual(make.returncode, 0, make.stderr)
            self.assertEqual(sorted(p.name for p in repo.glob("PWNED*")), [])

    def test_a_path_recorded_before_the_rule_is_refused_rather_than_written(self) -> None:
        """A repository adopted before the rule may already record such a path; reading it back refuses, naming it
        escaped, instead of writing it into `delivery/Makefile` again."""
        with tempfile.TemporaryDirectory() as directory:
            repo = repository(Path(directory), "shop", {
                "package.json": '{"name": "shop", "private": true, "scripts": {"test": "node --test"}}\n',
                "package-lock.json": "{}\n", "tools/go.mod": "module t\n\ngo 1.22\n",
            })
            self.assertEqual(slipwai(repo, "adopt", "--yes").returncode, 0)
            document = json.loads((repo / "project.json").read_text())
            document["deployables"]["tools"]["path"] = "x\n$(shell touch PWNED)\n#"
            (repo / "project.json").write_text(json.dumps(document, indent=2) + "\n")
            refreshed = slipwai(repo, "adopt", "--refresh")
            self.assertNotEqual(refreshed.returncode, 0)
            self.assertIn("deployable 'tools' is at `x\\n$(shell touch PWNED)\\n#`", refreshed.stderr)
            self.assertNotIn("PWNED", (repo / "delivery/Makefile").read_text())
            self.assertEqual(list(repo.glob("PWNED*")), [])


class MissingLockfilesTest(unittest.TestCase):
    """`missing_lockfiles` at its boundary, over a tree on disk and the paths Git would list."""

    def reported(self, files: dict[str, str]) -> list[str]:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), files)
            return [f.where for f in missing_lockfiles(root, sorted(files))]

    def test_a_plain_crate_with_no_cargo_lock_beside_it_is_reported(self) -> None:
        self.assertEqual(self.reported({"Cargo.toml": '[package]\nname = "a"\n'}), ["Cargo.toml"])

    def test_a_package_json_with_no_lock_is_reported(self) -> None:
        self.assertEqual(self.reported({"web/package.json": "{}\n"}), ["web/package.json"])

    def test_a_member_below_a_workspace_root_with_a_lock_is_reported_for_want_of_one_beside_it(self) -> None:
        self.assertEqual(self.reported({
            "Cargo.toml": '[workspace]\nmembers = ["a"]\n', "Cargo.lock": "",
            "a/Cargo.toml": '[package]\nname = "a"\n',
        }), ["a/Cargo.toml"])
