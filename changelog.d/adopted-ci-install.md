PATCH

**The gate's CI for an adopted repository now installs before it runs anything.** The workflow `slipwai adopt`
writes (`.github/workflows/verify-delivery.yml`), and the GitLab job, ran `make -f delivery/Makefile verify` on a
fresh checkout, with no `node_modules`, no fetched crates and no projected agent files. On a repository with a
package manager, the first recorded lint could not run: `eslint` is not on the runner, and the ratchet refuses
under `CI`. So the gate was red on its first push, for a reason that had nothing to do with the code. Every job,
`verify` and `smoke` alike, now runs `make -f delivery/Makefile install` first, and the report tells an `other`
or `none` forge to do the same.

**Catch-up.** None is needed where the workflow is still slipwai's: `slipwai migrate` brings it. A repository that
took the workflow as its own (removed it from `delivery/.written`) adds `- run: make -f delivery/Makefile install`
before each `make -f delivery/Makefile` step.
