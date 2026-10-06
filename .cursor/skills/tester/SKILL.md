---
name: tester
description: Black-box tests, test-first for core logic, and continuous test runs. Use when adding or changing tests or app logic. Run your project’s test command after changes; keep suite green.
---

# Tester — Project

Use this skill when writing or running tests, or when touching app logic or new behavior. **First action when adding new behavior:** Read this skill and [TEST_TDD.md](../TEST_TDD.md), then write a **failing** test at the appropriate tier(s) **before** production code.

---

## Role

- **Black-box tests:** Assert on **behavior** (public API: inputs and outputs). Do not depend on implementation details.
- **Test-first (tiers):** When [TEST_PLAN.md](../../../TEST_PLAN.md) defines **Tier 1** and **Tier 2**, use red → green at each tier that covers the change (fast feedback first when both apply; browser-only work may start at Tier 2). See TEST_TDD.md.
- **TDD loop:** (1) Tier 1 red/green if logic is covered by Tier 1. (2) Tier 2 red/green if integration or E2E is required. (3) Document if needed. (4) Run your **merge-ready** command from AGENT_HANDOFF.md before merge when your project defines one. If the stack **compiles**, that gate must include every documented compile (test/Debug **and** Release) and increment the local +BUILD counter. Non-compiling stacks skip the counter.
- **Integration chain:** With the unit test, add or extend a test that runs this piece together with the shipped behaviors it touches, through real interfaces. A green unit test alone is not the gate. See [integration-chain.mdc](../../rules/integration-chain.mdc).
- **Continuous:** Run your project test command after each small step. Keep the suite green. The command must include the chain suite, not only the new test file.
- **Evidence loop:** New behavior writes named events to the documented evidence file (TEST_TDD → *Evidence loop* + TEST_PLAN path). After smoke or a pasted log, harvest into a Tier 1 test. Do not treat logs as green without a pass/fail check. Debugger-only Trace is not evidence.

## Source of truth

- **What to test:** TEST_TDD.md.
- **Test plan (two tiers):** TEST_PLAN.md.
- **Always-on rule:** [.cursor/rules/testing.mdc](../../rules/testing.mdc)
