PATCH

**`/drive` checks a stage on another harness for style as well as behaviour, and records whether it showed RED.**
The verify command `/drive` hands `scripts/agents/delegate.py` is now the scoped tests and the scoped lint and
format check, so a local model's style slips are undone at once instead of being found at the push. After a kept
run, `/drive` reads its log for the new tests failing before they passed and records `RED observed` or `RED not
observed` on the stage line and `red=` in the benchmark, a new signal `benchmark.py end` accepts. It is recorded
and never enforced.

**Catch-up.** `slipwai migrate` brings the new `/drive` text and `benchmark.py`.
