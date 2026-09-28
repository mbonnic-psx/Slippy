---
description: Evaluate test effectiveness with mutation testing
argument-hint: [changed-production-paths]
---

# Mutation

Read `delivery/skills/mutation-testing/SKILL.md`. Target changed production code and use the ecosystem's mutation tool when the
project has configured it. Mutation tooling is intentionally not part of the mandatory repository gate: if it
is absent, report the exact setup decision needed instead of pretending mutations ran. Classify survivors,
add tests only for meaningful behavioural gaps, then finish with `make verify`.
