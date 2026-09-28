---
description: Show or change how /drive delegates implementation — the boundary a delegate is handed and the cycle it runs
argument-hint: [delegate=story|rule|task] [cycle=rule|example]
---

# Drive settings

`.specify/drive.json` holds two settings: `delegate`, how much one `drive-implement` delegate is handed — every rule of one
user story, one rule, or one task — and `cycle`, how many RED tests one RED-GREEN-REFACTOR cycle opens with — a
rule's examples together, or one at a time. `delivery/commands/drive.md`, *How implementation is delegated*, says what
each means and which veto overrides it. This command is how the settings are read and how they change: checked,
at any time, and never by quietly handing a delegate something else.

## No argument — show the settings

```sh
python3 delivery/scripts/agents/drive.py
```

Report both lines as printed — the value, what it means, and the veto that can override it on a slice.

## Arguments — change them

```sh
python3 delivery/scripts/agents/drive.py --set $ARGUMENTS
```

Each argument is `delegate=story|rule|task` or `cycle=rule|example`, and both may be given at once. The script
refuses a value it does not know and refuses `cycle=story` with the reason — it is the batch Principle V
prohibits — and writes nothing then. Report a refusal in its words; do not work around it by editing the file.
Then commit `.specify/drive.json` on its own, with a message naming the change: the choice is versioned with the project,
and it takes effect at the next implementation stage `/drive` runs. Nothing already running is interrupted.

## When the request is in words

"One delegate per task" is `delegate=task`; "one test at a time" is `cycle=example`; "back to the defaults" is
`delegate=story cycle=rule`. A request for one rule at a time is `delegate=rule`, and where the map is
rule-owned it hands the same thing as `task`. A request to batch a whole story's tests before implementing is
the one thing this cannot do — say why, and offer `cycle=rule`. `make -f delivery/Makefile check-agents` holds the file's
shape whether it was edited by hand or through this command.
