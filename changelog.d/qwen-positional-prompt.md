PATCH

**The runner drives Qwen Code with a positional prompt, not the deprecated `-p`.** Qwen Code 0.24.0 marks
`-p`/`--prompt` as deprecated and to be removed, in favour of the positional prompt, which is one-shot by
default. The registry's headless command for `qwen` is now `qwen {permissions} {prompt}`, so `/cruise` and
every other headless run keep working when the flag goes.

**Catch-up.** `slipwai migrate` brings the new registry.
