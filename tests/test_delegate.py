"""A stage of `/drive` sent to another harness — a local model through opencode — with its writes held afterwards."""
from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

from support import FactoryTestCase
from test_stage_models import installed, models

# Stands in for `opencode`: records how it was called, then does what the test asks of it.
FAKE = """#!/bin/sh
printf '%s\\n' "$@" > "$FAKE_DIR/args"
{behaviour}
"""


def delegate(repo: Path, fake: Path, *arguments: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["python3", "scripts/agents/delegate.py", *arguments], cwd=repo, text=True, capture_output=True,
        env={**os.environ, "PATH": f"{fake}:{os.environ['PATH']}", "FAKE_DIR": str(fake)},
    )


def git(repo: Path, *arguments: str) -> str:
    return subprocess.run(["git", *arguments], cwd=repo, text=True, capture_output=True, check=True).stdout.strip()


class DelegateTest(FactoryTestCase):
    def test_a_role_may_name_another_harness_and_only_a_stage_that_may_run_anything_goes_there(self) -> None:
        """`claude.local=opencode:<model>` sends a role's stages to opencode's headless command. A stage whose type
        is held to a narrower command scope is refused there, since a check afterwards cannot undo a command; so is
        a fallback that is itself on another harness. The line says where the stage runs and what it reruns on, and
        the harness's own agent file carries the fallback, the model it runs when the other harness fails."""
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "route", "standard", "python")
            installed(repo, "claude")
            done = models(repo, "--set", "implement=local", "claude.local=opencode:ollama/qwen-coder-32k",
                          "fallbacks.local=fast")
            self.assertEqual(done.returncode, 0, done.stderr)
            self.assertIn("fallbacks.local = fast", done.stdout)
            line = models(repo, "implement").stdout
            self.assertTrue(line.startswith("implement: local → opencode:ollama/qwen-coder-32k — opencode headless on "
                                            "ollama/qwen-coder-32k, through scripts/agents/delegate.py"), line)
            self.assertIn("a failed run is undone and reruns on `fast` → sonnet", line)
            self.assertEqual(models(repo, "--check").returncode, 0)
            projected = (repo / ".claude/agents/drive-implement.md").read_text()
            self.assertIn("model: sonnet", projected)
            self.assertNotIn("opencode:", projected.split("---")[1])

            written = (repo / ".specify/models.json").read_text()
            tasks = models(repo, "--set", "tasks=local")
            self.assertEqual(tasks.returncode, 1)
            self.assertIn("`tasks` runs under `local`, which `roles.claude` sends to another harness, but its type may "
                          "run `tasks-command` commands, not any", tasks.stderr)
            chained = models(repo, "--set", "fallbacks.fast=local")
            self.assertEqual(chained.returncode, 1)
            self.assertIn("`fallbacks.fast` is `local`, which `roles.claude` also sends to another harness",
                          chained.stderr)
            flagless = models(repo, "--set", "claude.local=amp:some-model")
            self.assertEqual(flagless.returncode, 1)
            self.assertIn("names Amp, whose headless command the registry records no model flag for", flagless.stderr)
            self.assertEqual((repo / ".specify/models.json").read_text(), written)

    def test_a_run_is_kept_when_it_stays_in_its_manifest_and_undone_to_where_it_started_when_it_does_not(self) -> None:
        """The other harness may not hold a write scope, so the scope is held after the run. A run that wrote
        only its manifest is kept, commits and all, and says so on the stage line. One that wrote outside it,
        changed nothing, failed its verify command or exited non-zero is undone — its commits dropped and every
        path it touched back as it was, including a change this session had not committed yet — and names the
        fallback to rerun on."""
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "held", "standard", "python")
            installed(repo, "claude")
            models(repo, "--set", "implement=local", "claude.local=opencode:ollama/qwen-coder-32k",
                   "fallbacks.local=fast")
            git(repo, "add", "-A")
            git(repo, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "route implement to opencode")
            fake = Path(directory) / "bin"
            fake.mkdir()
            brief = Path(directory) / "brief.md"
            brief.write_text("Add `todo.py` with a function `add(title)`.\n")
            (repo / "README.md").write_text("an edit this session has not committed\n")
            start = git(repo, "rev-parse", "HEAD")

            def behave(script: str) -> None:
                (fake / "opencode").write_text(FAKE.replace("{behaviour}", script))
                (fake / "opencode").chmod(0o755)

            commit = 'git -c user.name=q -c user.email=q@q commit -qm "{0}"'
            behave("mkdir -p src && echo 'def add(title): ...' > src/todo.py && echo ok > AGENTS.md && "
                   "git add src AGENTS.md && " + commit.format("increment"))
            outside = delegate(repo, fake, "implement", "--brief", str(brief), "--allow", "src/")
            self.assertEqual(outside.returncode, 1, outside.stderr)
            self.assertIn("delegate: failed — implement · model: opencode:ollama/qwen-coder-32k", outside.stdout)
            self.assertIn("it wrote outside its manifest: AGENTS.md. Undone, back to where it started; rerun it on "
                          "`fast` → sonnet", outside.stdout)
            self.assertEqual(git(repo, "rev-parse", "HEAD"), start)
            self.assertFalse((repo / "src/todo.py").exists())
            self.assertNotEqual((repo / "AGENTS.md").read_text(), "ok\n")
            self.assertEqual((repo / "README.md").read_text(), "an edit this session has not committed\n")
            self.assertEqual(git(repo, "status", "--porcelain"), "M README.md")
            flags, _, prompt = (fake / "args").read_text().partition("# drive-implement")
            self.assertEqual(flags.split(), ["run", "-m", "ollama/qwen-coder-32k"])
            self.assertIn("Add `todo.py` with a function `add(title)`.", prompt)
            self.assertIn("## The files you may write\n\n- `src/`\n\nNothing else.", prompt)

            behave("true")
            idle = delegate(repo, fake, "implement", "--brief", str(brief), "--allow", "src/")
            self.assertEqual(idle.returncode, 1)
            self.assertIn("it changed nothing", idle.stdout)
            behave("mkdir -p src && echo x > src/todo.py; exit 3")
            crashed = delegate(repo, fake, "implement", "--brief", str(brief), "--allow", "src/")
            self.assertIn("the harness exited 3", crashed.stdout)
            self.assertFalse((repo / "src/todo.py").exists())
            behave("mkdir -p src && echo x > src/todo.py")
            red = delegate(repo, fake, "implement", "--brief", str(brief), "--allow", "src/", "--verify",
                           "echo 1 test failed; exit 1")
            self.assertIn("`echo 1 test failed; exit 1` failed: 1 test failed", red.stdout)
            self.assertFalse((repo / "src/todo.py").exists())

            behave("mkdir -p src && echo 'def add(title): ...' > src/todo.py && git add src && "
                   + commit.format("increment"))
            kept = delegate(repo, fake, "implement", "--brief", str(brief), "--allow", "src/todo.py",
                            "--verify", "true")
            self.assertEqual(kept.returncode, 0, kept.stdout + kept.stderr)
            self.assertIn("delegate: done — implement · model: opencode:ollama/qwen-coder-32k · delegated, fresh "
                          "context", kept.stdout)
            self.assertIn("1 file(s): src/todo.py. Its output: .specify/delegations/implement-", kept.stdout)
            self.assertEqual(git(repo, "log", "-1", "--format=%s"), "increment")
            self.assertEqual(git(repo, "status", "--porcelain"), "M README.md")

            unrouted = delegate(repo, fake, "tasks", "--brief", str(brief), "--allow", "src/")
            self.assertEqual(unrouted.returncode, 2)
            self.assertIn("`tasks` does not map to another harness here", unrouted.stderr)
            self.assertIn(".specify/delegations/", (repo / ".gitignore").read_text())
            self.assertTrue(json.loads((repo / ".specify/models.json").read_text())["fallbacks"])

    def test_drive_checks_style_with_the_tests_and_records_red_without_enforcing_it(self) -> None:
        """A model that is not the host's slips on style as well as behaviour, so the verify command `/drive` hands
        a delegation is the scoped tests and the scoped lint and format check. Whether a kept run showed its tests
        failing before they passed is read from its log and recorded, on the stage line and in the benchmark,
        but never undoes a green run: a log shows what was printed, not the order the work was done in."""
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "red", "standard", "python")
            drive = (repo / "commands/drive.md").read_text()
            section = drive.split("**A line may name another harness**")[1].split("A stage is not always one")[0]
            self.assertIn('--verify "<scoped tests> && <scoped lint and format check>"', section)
            self.assertIn("a slip caught here is undone now rather than found at the push", section)
            self.assertIn("add `RED observed` or `RED not observed` to the stage line", section)
            self.assertIn("It is recorded, not enforced", section)
            feature = repo / "specs/001-red/slices/S1"
            feature.mkdir(parents=True)
            bench = ["python3", "scripts/agents/benchmark.py"]
            subprocess.run([*bench, "start", str(feature), "implement"], cwd=repo, check=True, capture_output=True)
            ended = subprocess.run([*bench, "end", str(feature), "implement", "red=not-observed",
                                    "model=opencode:ollama/qwen3-coder-30b-32k"], cwd=repo, text=True,
                                   capture_output=True)
            self.assertEqual(ended.returncode, 0, ended.stderr)
            record = json.loads((feature / "benchmark.json").read_text())
            self.assertEqual(record["stages"][-1]["signals"]["red"], "not-observed")
