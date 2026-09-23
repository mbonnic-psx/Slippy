MINOR

**A stage of `/drive` can run on another harness: Claude Code on your subscription can send implementation to
a local model through opencode.** `.specify/models.json` takes `<harness>:<model>` as a role's identifier —
`claude.local=opencode:ollama/qwen-coder-32k` — and a new `fallbacks` map naming the role a failed run reruns
on. The new `scripts/agents/delegate.py` runs such a stage with the other harness's verified headless command
and model flag (`headless.modelFlag`, recorded for opencode, Qwen Code and Claude Code), then holds the write
scope itself. A run that wrote outside its manifest, changed nothing, failed its verify command, exited
non-zero or timed out is undone, commits included, to exactly where it started. `/drive` then reruns the stage
on the fallback and reports both. Only stages whose type may run any command can go there, and
`models.py --check` refuses a table that sends any other. `docs/agent-harnesses.md` has the recipe.

**Catch-up.** `slipwai migrate` brings the new script, the registry, the `/drive` text and the
`.specify/delegations/` line in `.gitignore`. Nothing changes until a role is mapped to another harness.
