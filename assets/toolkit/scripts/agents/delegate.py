#!/usr/bin/env python3
"""Run one stage of `/drive` on another harness, and hold what it writes to the stage's manifest afterwards.

`.specify/models.json` may map a role to `<harness>:<model>` — `opencode:ollama/qwen-coder-32k` — which no
sub-agent of the harness running `/drive` can start. This does: it runs that harness's verified headless command
(`scripts/agents/registry.json`, `headless`) on that model, with the stage's type as the standing brief and the
task the session hands it, then compares the working tree with where it started. The other harness may not be
able to hold a write scope, so the scope is held here, after the fact: a run that wrote outside the files it was
allowed, changed nothing, failed its verify command, exited non-zero or ran out of time is undone — its commits
and its edits alike, back to exactly where it started — and says so, so the session reruns the stage on the
fallback `python3 scripts/agents/models.py <stage>` names. Only a type whose command scope is `any` gets here
(`models.py --check`): a write can be undone, a command cannot.

    python3 scripts/agents/delegate.py implement --brief brief.md --allow apps/service/src/todo.py \\
        --allow apps/service/tests/ --verify "make -C apps/service test"

`--allow` takes a file, or a directory ending in `/`. The last line is `delegate: done — …` or
`delegate: failed — …`, and the exit status agrees. What the other harness printed is kept under
`.specify/delegations/`, which is ignored. Run it alone: it reads every change in the tree while it runs as the
delegate's, so a sibling writing beside it would be undone with it.
"""

from __future__ import annotations

import json
import os
import shlex
import shutil
import signal
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import models  # noqa: E402 — after the path that makes this script's sibling importable

ROOT = models.ROOT
LOGS = ROOT / ".specify/delegations"
# The runner's own files change while a delegate runs — it writes what the session prints — and none is the
# delegate's doing. `.specify/delegations/` is this script's.
OWN = (".specify/cruise", ".specify/delegations/", "specs/cruise-checkpoint.md")
TIMEOUT = 1800


def git(*arguments: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(["git", *arguments], cwd=ROOT, capture_output=True, check=check)


def paths(output: bytes) -> set[str]:
    return {part.decode() for part in output.split(b"\0") if part}


def content(path: str) -> bytes | None:
    file = ROOT / path
    return file.read_bytes() if file.is_file() else None


def at(commit: str, path: str) -> bytes | None:
    shown = git("show", f"{commit}:{path}", check=False)
    return shown.stdout if shown.returncode == 0 else None


def snapshot() -> tuple[str, dict[str, bytes | None]]:
    """The commit the tree stands on, and every path that already differs from it, with what it held."""
    head = git("rev-parse", "HEAD").stdout.decode().strip()
    dirty = paths(git("diff", "--name-only", "-z", "HEAD").stdout)
    dirty |= paths(git("ls-files", "--others", "--exclude-standard", "-z").stdout)
    return head, {path: content(path) for path in dirty}


def changed(start: str, before: dict[str, bytes | None]) -> list[str]:
    """Every path whose content now differs from where the run started, whether committed or not."""
    candidates = set(before)
    candidates |= paths(git("diff", "--name-only", "-z", start).stdout)
    candidates |= paths(git("ls-files", "--others", "--exclude-standard", "-z").stdout)
    return sorted(path for path in candidates if not path.startswith(OWN)
                  and content(path) != (before[path] if path in before else at(start, path)))


def allowed(path: str, allow: list[str]) -> bool:
    return any(path == entry or (entry.endswith("/") and path.startswith(entry)) for entry in allow)


def undo(start: str, before: dict[str, bytes | None], touched: list[str]) -> None:
    """Back to exactly where the run started: its commits dropped, every path it touched as it was."""
    if git("rev-parse", "HEAD").stdout.decode().strip() != start:
        git("reset", "-q", "--soft", start)
    for path in touched:
        held = before[path] if path in before else at(start, path)
        file = ROOT / path
        if held is None:
            file.unlink(missing_ok=True)
        else:
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_bytes(held)
        git("reset", "-q", start, "--", path, check=False)


def arguments_of(argv: list[str]) -> tuple[str, Path, list[str], str | None, float]:
    if not argv or argv[0].startswith("-"):
        raise RuntimeError("usage: delegate.py <stage> --brief FILE --allow PATH [--allow PATH …] [--verify CMD] "
                           "[--timeout SECONDS]")
    stage, brief, allow, verify, timeout = argv[0], None, [], None, float(TIMEOUT)
    rest = iter(argv[1:])
    for flag in rest:
        value = next(rest, None)
        if value is None:
            raise RuntimeError(f"{flag} takes a value")
        if flag == "--brief":
            brief = Path(value)
        elif flag == "--allow":
            allow.append(value.removeprefix("./"))
        elif flag == "--verify":
            verify = value
        elif flag == "--timeout":
            timeout = float(value)
        else:
            raise RuntimeError(f"unknown option {flag}")
    if brief is None or not allow:
        raise RuntimeError("--brief and at least one --allow are required: the task, and the files it may write")
    return stage, brief, allow, verify, timeout


def target(stage: str) -> tuple[dict, str, str, str]:
    """The other harness's row, the model, the fallback as the session should read it, and the type's brief."""
    rows = models.registry()
    host = next((rows[key] for key in models.installed() if key in rows), None)
    if host is None or not models.MODELS.is_file():
        raise RuntimeError("no installed harness or no .specify/models.json: nothing maps this stage anywhere")
    table = json.loads(models.MODELS.read_text())
    role, model, why = models.resolve(stage, table, host)
    other = models.cross(model, rows)
    if other is None:
        raise RuntimeError(f"`{stage}` does not map to another harness here — {models.line(stage, table, host)}")
    declared = models.declared_types().get(stage)
    if declared is None or declared.get("commands") != "any":
        raise RuntimeError(f"`{stage}`'s type may not run on another harness: only a type whose commands are `any` "
                           "may (`python3 scripts/agents/models.py --check`)")
    back, back_model = models.fallback(role, table, host)
    source = (models.AGENT_TYPES / f"{declared['name']}.md").read_text()
    body = source[source.find("\n---\n", 4) + 5:].lstrip("\n")
    return rows[other[0]], other[1], f"`{back}` → {back_model or 'host model'}" if back else "the host model", body


def command(harness: dict, model: str, prompt: str) -> str:
    headless = harness.get("headless")
    if not isinstance(headless, dict) or not headless.get("modelFlag"):
        raise RuntimeError(f"the registry records no headless model flag for {harness['name']}")
    binary = shlex.split(str(headless["command"]))[0]
    if shutil.which(binary) is None:
        raise RuntimeError(f"`{binary}` is not on PATH, so {harness['name']} cannot run this stage")
    flag = str(headless["modelFlag"]).replace("{model}", shlex.quote(model))
    permissions = f"{headless.get('permissions', '')} {flag}".strip()
    return str(headless["command"]).replace("{permissions}", permissions).replace("{prompt}", shlex.quote(prompt))


def prompt_for(body: str, task: str, allow: list[str]) -> str:
    listed = "\n".join(f"- `{entry}`" for entry in allow)
    return (f"{body.rstrip()}\n\n---\n\n# This delegation\n\n{task.strip()}\n\n## The files you may write\n\n"
            f"{listed}\n\nNothing else. A write anywhere else undoes this whole run, commits included.\n")


def run(line: str, log: Path, timeout: float) -> str | None:
    """Run the harness in its own process group, its output into the log; None, or why it failed."""
    environment = {key: value for key, value in os.environ.items()
                   if key not in ("CRUISE_RUNNER", "CRUISE_ITERATION")}
    with log.open("wb") as out, subprocess.Popen(line, shell=True, cwd=ROOT, stdin=subprocess.DEVNULL, stdout=out,
                                                 stderr=subprocess.STDOUT, env=environment,
                                                 start_new_session=True) as process:
        try:
            status = process.wait(timeout=timeout)
        except BaseException as error:
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            if isinstance(error, subprocess.TimeoutExpired):
                return f"it ran past {timeout:.0f}s"
            raise
    return None if status == 0 else f"the harness exited {status}"


def main() -> None:
    stage, brief, allow, verify, timeout = arguments_of(sys.argv[1:])
    harness, model, back, body = target(stage)
    line = command(harness, model, prompt_for(body, brief.read_text(), allow))
    LOGS.mkdir(parents=True, exist_ok=True)
    log = LOGS / f"{stage}-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}.log"
    who = f"{harness['key']}:{model}"
    start, before = snapshot()
    began = time.monotonic()
    failure = run(line, log, timeout)
    touched = changed(start, before)
    outside = [path for path in touched if not allowed(path, allow)]
    if failure is None and outside:
        failure = f"it wrote outside its manifest: {', '.join(outside)}"
    if failure is None and not touched:
        failure = "it changed nothing"
    if failure is None and verify:
        # One stream, in the order it was written, so the last line is the verdict and not a stray install line.
        checked = subprocess.run(verify, shell=True, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                 text=True)
        if checked.returncode != 0:
            tail = checked.stdout.strip().splitlines()[-1:]
            failure = f"`{verify}` failed" + (f": {tail[0][:300]}" if tail else "")
    took = f"{time.monotonic() - began:.0f}s"
    relative = log.relative_to(ROOT)
    if failure is not None:
        undo(start, before, touched)
        print(f"delegate: failed — {stage} · model: {who} · {took}: {failure}. Undone, back to where it started; "
              f"rerun it on {back}. Its output: {relative}")
        raise SystemExit(1)
    print(f"delegate: done — {stage} · model: {who} · delegated, fresh context · {took} · "
          f"{len(touched)} file(s): {', '.join(touched)}. Its output: {relative}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, KeyError, subprocess.CalledProcessError) as error:
        print(f"delegate: {error}", file=sys.stderr)
        raise SystemExit(2) from None
