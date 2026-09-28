---
description: Show or change how /cruise runs /drive on its own — who decides, how it releases, what it demos with, when it parks
argument-hint: [key=value ...]
---

# Cruise settings

`.specify/cruise.json` holds how `/cruise` runs `delivery/commands/drive.md` with nobody at the wheel. `delivery/commands/cruise.md` says what
each value does at the stops it governs. This command is how the settings are read and how they change:
checked, at any time, and never by quietly running an iteration under different rules.

| Setting | Values | Default | Controls |
|---|---|---|---|
| `enabled` | `true` \| `false` | `false` | whether `/cruise` runs at all; `false` is a refusal that says so |
| `decide` | `recommended-first` \| `skipper-always` | `"recommended-first"` | who answers a product question: the host where the stage recommends an answer or a standing decision covers it and `drive-skipper` otherwise, or `drive-skipper` for every question |
| `release` | `flagged` \| `park` | `"flagged"` | the release-constraint stage: every slice continues or opens a flag seeded off, so every merge is dark; or park at the push and let a person say it is a release they want |
| `constitution` | `ratify` \| `park` | `"ratify"` | an unratified constitution: the skipper drafts and ratifies it, marked pending human review; or park |
| `hand` | `browser` \| `http` \| `cli` | `"browser"` | the top of the hand's ladder for a demo; each falls through to the next where it cannot run |
| `unblock` | `bosun` \| `park` | `"bosun"` | what a block becomes: work for `drive-bosun` first — a stub, a narrower reading, a repair — parking only at the catastrophic or when it fails; or a park at once |
| `stuck_after` | a whole number | `3` | iterations with no artifact change before the loop parks |
| `max_iterations` | a whole number or null | `null` | a budget on iterations; null is unbounded |
| `max_hours` | a whole number or null | `null` | a budget on wall time; null is unbounded |
| `poll_minutes` | a whole number | `10` | how often a parked loop looks for a reason to resume |
| `model` | a model identifier or null | `null` | the model the iteration itself runs on — the driver, and every stage `.specify/models.json` maps to `host`; null is the harness's default, which nobody at the wheel chooses |

## No argument — show the settings

```sh
python3 delivery/scripts/agents/cruise.py
```

Report every line as printed — the value and what it controls — and whether a run is enabled at all.

## Arguments — change them

```sh
python3 delivery/scripts/agents/cruise.py --set $ARGUMENTS
```

Each argument is `key=value` from the table, and several may be given at once. The script refuses a key it
does not know, a value outside the ones listed, and a number that is not one, and writes nothing then. Report
a refusal in its words; do not work around it by editing the file. Then commit `.specify/cruise.json` on its own, with a
message naming the change: it takes effect at the next iteration, and nothing already running is interrupted.
`make -f delivery/Makefile check-agents` holds the file's shape whether it was edited by hand or through this command.

## When the request is in words

"Turn it on" is `enabled=true`; "stop after tonight" is `max_hours=<n>`; "ask me before every release" is
`release=park`; "let the skipper decide everything" is `decide=skipper-always`; "drive on opus" is `model=opus`
(an identifier the harness's own model flag takes; `null` is its default); "back to the defaults" is
every key at the value the table shows. Stopping a run that is going is not a setting: it is `python3 delivery/scripts/agents/cruise.py
stop` — `touch .specify/cruise.stop`, which ends the run after the iteration in flight; `--now` ends that iteration too
— and `delivery/commands/cruise.md` says how the run ends cleanly from either.
