---
name: eval-engineer
description: Define and maintain lightweight evaluations (acceptance criteria, regression checks, golden tests) so “green” is objective and repeatable.
---

# Eval Engineer — Make “Green” Objective

Use this skill when:
- A change risks regressions
- “Done” is ambiguous
- You need to justify that work is **green** with minimal cost

This skill is tool-agnostic. It produces **verifiable checks** and a cheap evaluation loop.

---

## Principles

- **Small and repeatable** beats “big and rarely run”.
- Prefer **fast Tier 1** checks; reserve Tier 2/E2E for changes that demand it.
- Evals must be **objective**: either pass/fail or clearly measurable.

---

## Outputs you must produce (minimum)

1. **Acceptance criteria (2–5 items)**  
   Each AC must be verifiable. Avoid “works” or “seems fine”.

2. **Regression risk statement (1 paragraph)**  
   What could break, and what signal will catch it?

3. **Evaluation loop**
   - **Tier 1**: fastest check(s) you will run every time
   - **Tier 2**: when needed (integration/E2E or real environment)

---

## AC templates (pick one; keep it short)

### Bullet AC (default)
- AC1: ___ (how to verify)
- AC2: ___ (how to verify)

### BDD-lite (when behavior is user-facing)
- Given ___
- When ___
- Then ___ (verifiable)

---

## “Golden signals” checklist (cheap)

Pick the smallest set that fits:
- **Integrity**: static checks (lint/parse/schema)
- **Behavior**: one targeted scenario (manual or scripted)
- **Regression**: verify one previously failing bug stays fixed
- **Evidence**: a named runtime event (or equivalent) that the scenario must emit; harvest it into Tier 1 when the decision is expressible without the runtime (TEST_TDD → *Evidence loop*)

---

## Anti-patterns

- Writing evals that are too slow to run
- Treating a pasted log as proof **without** a pass/fail check or a test that can replay the decision
- Adding evals without documenting when to run them
- Silent success (no discrete evidence) on new behavior

