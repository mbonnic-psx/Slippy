#!/usr/bin/env python3
"""Which model runs each stage of `/drive`'s ladder, for the harness this project is initialised for.

`.specify/models.json` holds the choice — a role per stage, and per harness what each role maps to — and
`scripts/agents/registry.json` says whether the harness can act on it at all. This reads both and prints the
one line `/drive` needs before a stage: the model to delegate to, or why the stage runs on the host model.
Nothing is guessed: a role with no identifier mapped, a harness the registry records no mechanism for, and a
project with no table at all are each said in words, so a stage that did not switch is a stage that says so.

    python3 scripts/agents/models.py              # the whole table, per installed harness
    python3 scripts/agents/models.py implement    # one stage, keyed by the command it runs
    python3 scripts/agents/models.py --check      # the table is well-formed; `make check-agents` runs this
    python3 scripts/agents/models.py --set implement=strong claude.fast=haiku   # change it, checked, any time
    python3 scripts/agents/models.py --set implement=local claude.local=opencode:ollama/qwen-coder-32k fallbacks.local=fast

A role may map to `<harness>:<model>` — another harness's headless command, on that harness's model. Such a
stage is not delegated through this harness's sub-agents but through `scripts/agents/delegate.py`, which starts
the other harness, holds its writes to the stage's manifest afterwards, and undoes the lot when it fails; the
role named under `fallbacks` is what the stage then reruns on. Only a type whose command scope is `any` may run
there, since a check after the fact can undo a write but not a command.

A change — by hand or with `--set` — takes effect at the next stage `/drive` runs: the table is read before every
stage and cached nowhere. `slipwai migrate` merges a newer factory's table over an edited one rather than
replacing it, so a mapped identifier survives the way every edit to a generated file does.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any


def project_root(script: Path, depth: int) -> Path:
    """The repository root: the nearest directory above this script holding `project.json`.

    This script's own tree is `<root>/scripts` in a generated project and `<root>/<layout.delivery>/scripts`
    where the method was installed beside an existing codebase (`project.json`'s `layout.delivery`), so how
    far below the root it sits is not something to count; `depth` is only the fallback for a tree with no
    manifest at all.
    """
    for candidate in script.parents:
        if (candidate / "project.json").is_file():
            return candidate
    return script.parents[depth]


ROOT = project_root(Path(__file__).resolve(), 2)
REGISTRY = Path(__file__).with_name("registry.json")
PROJECT = Path(__file__).with_name("project.py")
MODELS = ROOT / ".specify/models.json"
INTEGRATION = ROOT / ".specify/integration.json"
# Every stage the table may name, keyed by the command the stage runs. The factory writes the same list into
# `.specify/models.json`; a key outside it is a typo the check reports rather than a row `/drive` never reads.
KNOWN_STAGES = (
    "principles", "specify", "event-model", "split", "example-map", "gaps", "release-constraint", "plan", "tasks",
    "implement", "converge", "demo", "adversary", "mutation", "skipper", "hand", "bosun",
)
# A role mapped to this runs on the model running `/drive` itself: no delegation, said in as many words.
HOST = "host"
# The canonical agent types, siblings of this script's `scripts/` wherever it sits (see project.py): each declares
# the stage it runs and its command scope, which is what decides whether that stage may leave this harness.
AGENT_TYPES = Path(__file__).resolve().parents[2] / "agents"
ABSENT = f"no {MODELS.relative_to(ROOT)}: every stage runs on the host model; `slipwai migrate` writes the table"


def installed() -> list[str]:
    """The harness keys Spec Kit recorded as installed; none before `./init`."""
    if not INTEGRATION.is_file():
        return []
    state = json.loads(INTEGRATION.read_text())
    keys = state.get("installed_integrations")
    if isinstance(keys, list) and keys:
        return list(dict.fromkeys(key for key in keys if isinstance(key, str)))
    default = state.get("default_integration")
    return [default] if isinstance(default, str) else []


def registry() -> dict[str, dict[str, Any]]:
    return {entry["key"]: entry for entry in json.loads(REGISTRY.read_text())["harnesses"]}


def cross(value: object, rows: dict[str, dict[str, Any]] | None = None) -> tuple[str, str] | None:
    """(harness key, model) where an identifier names another harness — `opencode:ollama/qwen-coder-32k` — else
    None. Only a prefix the registry knows counts, so a model that carries a colon of its own (`qwen3.5:4b`)
    is still one identifier."""
    if not isinstance(value, str):
        return None
    key, separator, model = value.partition(":")
    if not separator or not model or key not in (rows if rows is not None else registry()):
        return None
    return key, model


def declared_types() -> dict[str, dict[str, str]]:
    """Each canonical type's declaration, keyed by the stage it runs; a type with no stage is left out."""
    declared: dict[str, dict[str, str]] = {}
    for path in sorted(AGENT_TYPES.glob("*.md")) if AGENT_TYPES.is_dir() else []:
        source = path.read_text()
        end = source.find("\n---\n", 4)
        if not source.startswith("---\n") or end == -1:
            continue
        fields = {key.strip(): value.strip() for key, _, value in
                  (row.partition(":") for row in source[4:end].splitlines()) if key.strip()}
        if fields.get("stage"):
            declared[fields["stage"]] = fields
    return declared


def fallback(role: str, table: dict[str, Any], harness: dict[str, Any]) -> tuple[str | None, str | None]:
    """(role, model or None) a stage on another harness reruns on when that run fails: the role `fallbacks` names
    for it, resolved on this harness, else the host model."""
    named = table.get("fallbacks", {}).get(role)
    if not isinstance(named, str):
        return None, None
    value = table.get("roles", {}).get(harness["key"], {}).get(named)
    return named, None if value in (None, HOST) else str(value)


def role_of(stage: str, table: dict[str, Any]) -> tuple[str, str]:
    """The role a stage runs under, and a note when it fell to the `default` row."""
    stages = table["stages"]
    if stage in stages:
        return str(stages[stage]), ""
    return str(stages["default"]), f"no `{stage}` row; the `default` row applies"


def resolve(stage: str, table: dict[str, Any], harness: dict[str, Any]) -> tuple[str, str | None, str]:
    """(role, model or None, why) for one stage on one harness. None is the host model, and `why` says which
    of the three reasons made it so — or, with a model, how this harness switches to it."""
    role, fell = role_of(stage, table)
    name = harness["name"]
    mapped = table.get("roles", {}).get(harness["key"])
    other = cross(mapped.get(role)) if isinstance(mapped, dict) else None
    if other is not None:
        rows = registry()
        back, model = fallback(role, table, harness)
        after = f"`{back}` → {model or 'host model'}" if back else "the host model"
        how = (f"{rows[other[0]]['name']} headless on {other[1]}, through scripts/agents/delegate.py, its writes held "
               f"to the manifest afterwards; a failed run is undone and reruns on {after}")
        return role, f"{other[0]}:{other[1]}", f"{how}; {fell}" if fell else how
    mechanism = harness.get("subagentModel")
    if not isinstance(mechanism, dict):
        return role, None, f"the registry records no way for {name} to choose a model for a sub-task"
    roles = table.get("roles", {}).get(harness["key"])
    if not isinstance(roles, dict):
        return role, None, f"no roles mapped for `{harness['key']}` in .specify/models.json"
    value = roles.get(role)
    if value is None:
        return role, None, f"no identifier mapped for `{role}` under `{harness['key']}` in .specify/models.json"
    if value == HOST:
        return role, None, f"`{role}` maps to the host model"
    how = f"{name}: {mechanism.get('how', 'see the registry')}"
    return role, str(value), f"{how}; {fell}" if fell else how


def line(stage: str, table: dict[str, Any], harness: dict[str, Any]) -> str:
    role, model, why = resolve(stage, table, harness)
    target = model if model is not None else "host model"
    return f"{stage}: {role} → {target} — {why}"


def switching(harness: dict[str, Any]) -> str:
    """How this harness gives a sub-task its model, in the spelling its identifiers take — or that it cannot."""
    mechanism = harness.get("subagentModel")
    if not isinstance(mechanism, dict):
        return f"cannot switch: the registry records no way for {harness['name']} to choose a model for a sub-task"
    return f"can switch: {mechanism.get('how')}; identifiers: {mechanism.get('identifiers')}"


def check(table: object, registry: dict[str, dict[str, Any]]) -> list[str]:
    """Everything a hand edit can break, each as one finding."""
    findings: list[str] = []
    if not isinstance(table, dict):
        return ["the table is not a JSON object"]
    stages = table.get("stages")
    if not isinstance(stages, dict) or "default" not in stages:
        return ["`stages` must be an object with a `default` row"]
    allowed = set(KNOWN_STAGES) | {"default"}
    for key, value in stages.items():
        if key not in allowed:
            findings.append(f"`stages.{key}` is not a stage of the ladder; known: {', '.join(KNOWN_STAGES)}")
        if not isinstance(value, str) or not value:
            findings.append(f"`stages.{key}` must name a role")
    named = {value for value in stages.values() if isinstance(value, str)}
    roles = table.get("roles")
    if not isinstance(roles, dict):
        return findings + ["`roles` must be an object keyed by harness"]
    for key, mapping in roles.items():
        if key not in registry:
            findings.append(f"`roles.{key}` is not a harness the registry knows")
        if not isinstance(mapping, dict):
            findings.append(f"`roles.{key}` must map each role to an identifier, `{HOST}`, or null")
            continue
        for role in sorted(named - set(mapping)):
            findings.append(f"`roles.{key}` does not say what `{role}` maps to — an identifier, `{HOST}`, or null")
        for role, value in mapping.items():
            if value is not None and (not isinstance(value, str) or not value):
                findings.append(f"`roles.{key}.{role}` must be an identifier, `{HOST}`, or null")
    return findings + check_cross(table, roles, stages, registry)


def check_cross(table: dict[str, Any], roles: dict[str, Any], stages: dict[str, Any],
                registry: dict[str, dict[str, Any]]) -> list[str]:
    """What a role mapped to another harness needs: a harness whose headless command takes a model, only stages
    whose type may run any command, and a fallback that stays on this harness."""
    findings: list[str] = []
    fallbacks = table.get("fallbacks", {})
    if not isinstance(fallbacks, dict):
        return ["`fallbacks` must map a role to the role its failed stages rerun on"]
    types = declared_types()
    for key, mapping in roles.items():
        if not isinstance(mapping, dict):
            continue
        for role, value in mapping.items():
            other = cross(value, registry)
            if other is None:
                continue
            headless = registry[other[0]].get("headless")
            if not isinstance(headless, dict) or not headless.get("modelFlag"):
                findings.append(f"`roles.{key}.{role}` names {registry[other[0]]['name']}, whose headless command the "
                                "registry records no model flag for (`headless.modelFlag`)")
            for stage in KNOWN_STAGES:
                if role_of(stage, table)[0] != role:
                    continue
                declared = types.get(stage)
                if declared is None:
                    findings.append(f"`{stage}` runs under `{role}`, which `roles.{key}` sends to another harness, but "
                                    "it has no agent type, so it is never delegated")
                elif declared.get("commands") != "any":
                    findings.append(f"`{stage}` runs under `{role}`, which `roles.{key}` sends to another harness, but "
                                    f"its type may run `{declared.get('commands')}` commands, not any — a check "
                                    "afterwards cannot undo a command; give it a role of its own")
    for role, back in fallbacks.items():
        if not isinstance(back, str) or not back:
            findings.append(f"`fallbacks.{role}` must name a role")
            continue
        for key, mapping in roles.items():
            if isinstance(mapping, dict) and cross(mapping.get(back), registry) is not None:
                findings.append(f"`fallbacks.{role}` is `{back}`, which `roles.{key}` also sends to another harness; "
                                "a fallback runs here")
    return findings


def assign(table: dict[str, Any], registry: dict[str, dict[str, Any]], assignment: str) -> str:
    """Apply one `stage=role` or `harness.role=identifier` to the table in place, and say what changed.

    A role a stage newly names is added as `null` under every harness so the table stays whole and the line
    before the stage says "no identifier mapped" until somebody maps it. A harness the registry records no
    mechanism for is refused: a role mapped for it would never be read, and a setting that does nothing is
    worse than a refusal that says why (docs/agent-harnesses.md).
    """
    key, separator, value = assignment.partition("=")
    if not separator or not key or not value:
        raise RuntimeError(f"--set takes stage=role or harness.role=identifier, not {assignment!r}")
    if key.startswith("fallbacks."):
        role = key.split(".", 1)[1]
        table.setdefault("fallbacks", {})[role] = value
        return f"fallbacks.{role} = {value}"
    if "." in key:
        harness, role = key.split(".", 1)
        entry = registry.get(harness)
        if entry is None:
            raise RuntimeError(f"`{harness}` is not a harness the registry knows; `make agents-list` names them")
        # Another harness's headless command is started by delegate.py, not by this harness's sub-agents, so a
        # harness with no sub-agent model of its own can still send a stage there.
        if cross(value, registry) is None and not isinstance(entry.get("subagentModel"), dict):
            raise RuntimeError(f"the registry records no way for {entry['name']} to choose a model for a sub-task, "
                               "so a role mapped for it would never be read")
        table.setdefault("roles", {}).setdefault(harness, {})[role] = None if value == "null" else value
        return f"roles.{harness}.{role} = {value}"
    if key not in KNOWN_STAGES and key != "default":
        raise RuntimeError(f"`{key}` is not a stage of the ladder; known: default, {', '.join(KNOWN_STAGES)}")
    table.setdefault("stages", {})[key] = value
    unmapped = [harness for harness, mapping in table.get("roles", {}).items() if value not in mapping]
    for harness in unmapped:
        table["roles"][harness][value] = None
    added = f" — `{value}` added as null under {', '.join(unmapped)}; map it with --set <harness>.{value}=<id>" \
        if unmapped else ""
    return f"stages.{key} = {value}{added}"


def reproject() -> None:
    """Rewrite the projections, because the agent files carry the model this table just changed.

    A harness that names a sub-task's model in a file (`registry.json`, `agentFile`) has that model written
    into `<dir>/drive-<stage>.md` by `scripts/agents/project.py`. Leaving them behind would mean a change made
    here takes effect at the next stage on Claude Code and never on Codex, and `make check-agents` reporting
    drift for a file nobody edited. Absent an installed integration there is nothing to write, and a failure
    here is reported rather than raised: the table is already written, and `make agents` is the retry.
    """
    if not INTEGRATION.is_file():
        return
    done = subprocess.run(["python3", str(PROJECT)], cwd=ROOT, text=True, capture_output=True)
    if done.returncode != 0:
        print(f"the projections still carry the old model — run `make agents`: {done.stderr.strip()}",
              file=sys.stderr)
        return
    print("Projections rewritten: the agent types carry the model this table names.")


def main() -> None:
    arguments = sys.argv[1:]
    registry = {entry["key"]: entry for entry in json.loads(REGISTRY.read_text())["harnesses"]}
    if not MODELS.is_file():
        print(ABSENT)
        return
    table = json.loads(MODELS.read_text())
    if "--set" in arguments:
        assignments = arguments[arguments.index("--set") + 1:]
        if not assignments:
            raise RuntimeError("--set takes stage=role or harness.role=identifier")
        changed = [assign(table, registry, assignment) for assignment in assignments]
        findings = check(table, registry)
        if findings:
            listed = "\n  - ".join(findings)
            raise RuntimeError(f"not written — the change would leave the table malformed:\n  - {listed}")
        MODELS.write_text(json.dumps(table, indent=2, ensure_ascii=False) + "\n")
        for line_ in changed:
            print(line_)
        print(f"{MODELS.relative_to(ROOT)} written; it takes effect at the next stage /drive runs. Commit it: the "
              "choice is versioned with the project.")
        reproject()
        return
    findings = check(table, registry)
    if "--check" in arguments:
        if findings:
            raise RuntimeError(f"{MODELS.relative_to(ROOT)}:\n  - " + "\n  - ".join(findings))
        print(f"check-models: {MODELS.relative_to(ROOT)} names {len(table['stages']) - 1} stage(s) and "
              f"{len(table['roles'])} harness(es)")
        return
    if findings:
        raise RuntimeError(f"{MODELS.relative_to(ROOT)} is malformed; `make check-agents` lists why")
    harnesses = [registry[key] for key in installed() if key in registry]
    stages = [argument for argument in arguments if not argument.startswith("--")]
    if not harnesses:
        print("no harness installed yet (`./init --integration <agent>` records one): every stage runs on the host "
              "model. The table, by role:")
        for stage in KNOWN_STAGES:
            print(f"  {line(stage, table, {'key': '', 'name': 'an uninitialised project', 'subagentModel': None})}"
                  .split(" — ")[0])
        return
    for harness in harnesses:
        if len(harnesses) > 1 or not stages:
            print(f"{harness['name']} ({harness['key']}):")
            print(f"  {switching(harness)}")
        for stage in stages or KNOWN_STAGES:
            print(f"  {line(stage, table, harness)}" if len(harnesses) > 1 or not stages
                  else line(stage, table, harness))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, KeyError) as error:
        print(f"models: {error}", file=sys.stderr)
        raise SystemExit(1) from None
