# Running slipwai

What is proven about running each application that existed before the delivery method did — the command, the
port, what has to be seeded first, the runtime it actually needs and the ones it cannot run on — written by
whoever proved it, with the date. This file is the repository's own: `slipwai migrate` and `/survey` never
rewrite it, and `skills/run-the-app/SKILL.md` points here. Record what was proven, not what a README promises,
and record the run that failed too: a runtime the gate compiles on and the application cannot load is the kind
of fact that is only ever learned once if it is written down.

## `.` (python)

A command-line tool, not a server: no port, nothing to seed, nothing listening afterwards. Proven 2026-09-28 by
drive-bosun (/cruise iteration 2, feature `001-rust-cargo-adopt`, stage Pin), from the repository root, with the
checkout's own launcher (`./slipwai`, which puts `src/` on `PYTHONPATH` and runs `python3 -m slipwai`; no install
step is needed for it):

```sh
./slipwai --version && ./slipwai adopt --next
```

- `./slipwai --version` printed `slipwai 1.4.0.dev0` and exited 0.
- `./slipwai adopt --next` imports the CLI and the adoption path, reads `project.json` and the tree, printed the
  adoption sequence (`done` / `now` / `then` lines, ending with the pointer to `delivery/docs/adoption.md`) and
  exited 0 in about 0.1 s. It writes nothing: `git status --porcelain` was identical before and after the run. An
  unknown flag exits 2, so a broken start is a non-zero exit, not a quiet pass.
- It depends on the working directory: it answers only from a repository whose `project.json` records an
  adoption, which this one does — run it from the root.
- Runtime it ran on: `python3 --version` → Python 3.14.4 (`/usr/bin/python3`, WSL2 Linux). That is the only
  interpreter on this machine; `requires-python` says `>=3.11` and `toolchain.version` says 3.11, but no run on
  3.11, 3.12 or 3.13 has been proven here (the only other `python3.12` on the path is a dangling link). Nothing is
  yet known that it cannot run on.

This is recorded as `commands.smoke` in `project.json`. `delivery/Makefile`'s `smoke` target and the gate's CI
smoke job are generated from that record and were not regenerated in the same run (they are control files a
/cruise run may not change), so until a person regenerates them `make -f delivery/Makefile smoke` still says no
smoke is recorded — see decision D6 in `specs/001-rust-cargo-adopt/decisions.md`.
