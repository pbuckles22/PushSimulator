---
name: tech-debt-evaluator
description: Assess and prioritize technical debt (code, architecture, tests, docs, performance). Use when planning refactors, sprint planning, or evaluating codebase health.
---

# Tech debt evaluator — Project

Use this skill when evaluating technical debt, planning refactors, or assessing codebase health. Produces a structured, prioritized list.

---

## Role

- **Classify debt** by category: **code** (duplication, complexity, coupling), **architecture** (layering, boundaries), **tests** (gaps, flakiness), **documentation** (out-of-date specs), **performance** (UI thread, blocking work).
- **Severity:** **Low**, **Medium**, **High**, **Critical** (e.g. data loss, crash).
- **Prioritize:** Impact and effort; surface high-impact, lower-effort items first.
- **Reference:** Use DEV_GUIDE.md, TEST_TDD.md, DESIGN_SYSTEM.md, AGENT_HANDOFF.md as the bar. Debt = deviation or gap vs. those.

## When to use

- User asks for a tech-debt pass, health check, or refactor plan.
- Before a large feature or architectural change.
- Sprint or handoff planning.

## Output format

For each item: **Category** | **Severity** | **What** (one line + file/area) | **Why it matters** | **Suggested fix** | **Effort** (optional). Optionally group by "Do first" vs "Backlog".
