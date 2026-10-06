# TEST_TDD — Project

## How to test

- **Black-box:** Assert on behavior (public API: inputs and outputs). Do not depend on implementation details. See [tester/SKILL.md](tester/SKILL.md).
- **Continuous:** Run your project’s test command after adding or changing logic or tests; keep the suite green.
- **Tiers ([TEST_PLAN.md](../../TEST_PLAN.md)):** When defined, **Tier 1** is fast feedback; **Tier 2** is integration or E2E. Validate at every tier that applies to the change.

---

## TDD when TEST_PLAN defines Tier 1 and Tier 2

**Default:** Do not merge production changes until the right tier(s) have **failing test → passing test** for the behavior you are adding or changing.

### Tier 1

Use for logic covered by your fast test command (unit, headless, mocked APIs — whatever TEST_PLAN.md says).

1. **Red** — Add or extend a test that describes the new behavior and fails with the current code.
2. **Green** — Implement until the Tier 1 command passes.

### Tier 2

Use when behavior must hold in a real runtime (browser, device, network, DB — whatever TEST_PLAN.md says).

1. **Red** — Add or extend an integration or E2E test that fails until the feature exists.
2. **Green** — Implement until the Tier 2 command passes.

**When both apply:** Usually Tier 1 first, then Tier 2. Pure integration-only changes may start at Tier 2; add Tier 1 later if you extract testable logic.

### Exceptions

- Docs-only, config-only, or comment-only changes.
- Trivial one-line fixes with no behavior change (still run your merge-ready command if the project uses one).
- Pure refactors preserving behavior: keep tests green.

Never leave failing tests on the default branch.

### Integration chain (with the unit test, not instead of it)

Isolated red/green proves the piece. It does not prove shipped behaviors **1..X** still work together.

1. Name the chain this change sits in (who calls it, who it calls, what runs after it).
2. **Red** — Add or extend a test that runs those behaviors through their real interfaces and fails if any step breaks.
3. **Green** — Implement until that chain test and the piece’s own test both pass.
4. Run the suite that contains the chain in the merge-ready command. Do not run only the new file.

A mock of the neighbor you just changed is not this test. See [integration-chain.mdc](../rules/integration-chain.mdc).

---

## Evidence loop (logs → tests → CI)

Field/runtime evidence is how this template stays honest when CI cannot run the real environment.

1. **Emit** — Each new behavior ships discrete, structured events (stable name + fields you would assert on). Lifecycle and decisions yes; per-tick traces no. Write them to the **documented evidence file** (TEST_PLAN.md). Debugger-only Trace/Debug is not evidence. Document the names and the path in TEST_PLAN.md.
2. **Verify** — Human/Tier 2 uses those events in that file (plus the documented install/run step) as the checklist. A missing expected line is a fail. “Nothing visible” is not proof the code ran unless the file also has the named line.
3. **Harvest** — After a find or a PASS with a useful trace: extract pure inputs → outputs. Add or extend a **Tier 1** test named after the scenario. Keep the evidence line as the Tier 2 checklist item.
4. **CI** — Merge-ready / CI runs that Tier 1 test every time. Do not leave “we’ll catch it next smoke” as the only net.

Anti-patterns: per-frame spam; logs with no test that can replay the decision; treating a pasted log as green without a pass/fail check; calling a ship a PASS when neither human nor agent can open the evidence file.

---

## Merge-ready

Document your **merge-ready** or **CI** command in **AGENT_HANDOFF.md** and run it before merge when your team uses that gate.

### Compile counter (compiling stacks only)

If this repo **compiles**, merge-ready must **run and count** every compile configuration it documents (usually test/Debug **and** Release). Do not treat `test` as covering the Release/production compile.

- **Do:** gitignored local counter (e.g. `build_number.txt`, seed `build_number.txt.example` = `0`); increment on a real compile; skip design-time/IntelliSense; inject `+N` into the binary informational version when the stack supports it.
- **Do not:** commit the counter; write `+N` into tracked public version files; increment on IDE design-time builds.
- **Skip** when nothing compiles (scripts, bookmarklets, twitchurl*, DJ library tools, docs-only). Tests/docs are the gate.
