---
name: tech-lead
description: >-
  Technical leadership: turn plans into sequenced work, clarify definition of done,
  surface risks and dependencies, align tests and CI with scope, and coordinate
  cross-cutting changes (storage, messaging, APIs). Use when the user asks for a
  tech lead, implementation plan, story breakdown, risk assessment before a large
  change, or merge ordering across epics.
---

# Tech lead — Project

Use this skill for **orchestrating** work across files or epics—not for line-by-line review (use **code-reviewer**) or product prioritization alone (use **pm-governance**).

---

## Responsibilities

- **Sequencing:** Order tasks so foundations land first (types, contracts, storage shape) before UI polish; avoid changes that alter contracts without updating all callers.
- **Definition of done:** Behavior matches the plan story + tests (Tier 1 / Tier 2 per **TEST_TDD.md**); **merge-ready command** green when the change set warrants it; docs (**PM_PLAN**, product plan checkboxes) updated if scope or user-visible contract changed.
- **Risks:** Call out **data migration**, **permission or security** increases, **integration** with third-party systems, and **performance** timing issues; link mitigations to backlog items when relevant.
- **Consistency:** Same patterns as existing modules; avoid parallel frameworks or duplicate primitives.
- **Ship cadence:** Fast loop until they can run it. **UCPH** parks the feature branch. **CMPH** lands `main` and keeps the branch. **SWAT** is epic close. See [wrap-on-command.mdc](../../rules/wrap-on-command.mdc). Do not start the next story while waiting on a human.

## Workflow

1. Read the relevant **epic / story** in the product plan and **PM_PLAN** pointer.
2. List **touchpoints** (files, modules, APIs, tests).
3. Decide **vertical slices** shippable without breaking `main`.
4. Assign **test tier** per **tester** / **TEST_TDD** skill.

## Handoffs

- **Architecture / layering questions:** Use a domain-specific architect skill if you have one, or evaluate structure with code-quality-gate.
- **Copy and internal docs:** techwriter skill.
- **Correctness / security:** code-reviewer skill.
