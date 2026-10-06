---
name: green-and-clean
description: Operating model for disciplined agent work: stay on scope, avoid guessing, decompose work into verifiable chunks, and keep context lean and durable.
---

# Green and Clean — Operating Model

This skill enforces a strict operating model:

- **Green**: Work is verifiably correct and meets explicit acceptance criteria.
- **Clean**: Context is curated; durable state lives in tracked docs, not chat.

Use this skill at the start of any non-trivial task, when scope feels unclear, when you suspect drift, and before declaring work done.

---

## Non‑negotiables

- **No guessing**: if required inputs are missing (paths, branch, VM name, expected behavior), STOP and ask for them.
- **No drift**: do not expand scope beyond the current feature/phase.
- **No hidden state**: durable decisions must be written into tracked docs (not only chat, not only local handoff notes).
- **No “done-ish”**: work is only complete when acceptance criteria and validation are satisfied.

---

## Pre‑flight (before doing work)

Confirm each item explicitly:

1. **Objective**: one sentence describing what “done” looks like.
2. **Scope**:
   - **In scope**:
   - **Out of scope**:
3. **Inputs present**: identify any required info; if any are missing, **ask**.
4. **Acceptance criteria** (must be verifiable):
   - AC1:
   - AC2:
5. **Validation tier**: which checks/tests will be run (minimum Tier 1 where defined).
6. **Plan size**:
   - If the work is more than ~3 meaningful steps, **decompose** into bite-sized tasks with acceptance criteria.

---

## In‑flight discipline (while working)

At natural checkpoints (every 1–3 edits / tool calls), run this quick audit:

- **On track**: is the current step directly contributing to the objective?
- **Clarity**: am I relying on assumptions? If yes, pause and ask.
- **Decomposition**: has the task become multi-stage? If yes, split into verifiable subtasks.
- **Context hygiene**:
  - Prefer referencing tracked docs over repeating long histories.
  - Avoid copying logs unless they directly affect a decision.

Drift response:
- If new work is discovered, **record it as “Next”** instead of expanding the current scope unless the user explicitly reprioritizes.

---

## Post‑flight (before declaring complete)

1. **Acceptance criteria met**: demonstrate how each AC is satisfied.
2. **Validation executed**: note what was run (and what was not).
3. **Docs updated**: if behavior/process changed, update the appropriate tracked docs:
   - `AGENT_HANDOFF.md`, `PM_PLAN.md`, `TEST_PLAN.md`, README (as applicable)
4. **Handoff readiness**: if ending a session, follow `.cursor/rules/handoff-checklist.mdc` and write a concise handoff note using the session summarizer skill.
