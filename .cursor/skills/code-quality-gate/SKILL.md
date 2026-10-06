---
name: code-quality-gate
description: >-
  Change-aware quality pass: diff scope, readability, function size and control
  flow, comments where non-obvious, and alignment with project linting. Use
  before merge or with handoff when the user wants industry-style
  maintainability beyond what tests assert.
---

# Code quality gate — Project

**Machine enforcement** lives in your linter/formatter config and merge-ready command. This skill is the **human + agent** companion: review the **latest change set** (not the whole repo at random) for layout, clarity, and spaghetti risk that linters only approximate.

---

## When to use

- After a coherent slice of work, **before** `git push` / merge to `main`, especially without a PR diff view.
- With **code-reviewer** and **tech-debt-evaluator** for handoffs (see `.cursor/rules/handoff-checklist.mdc`).
- When the user asks for **best practices**, **readability**, or **anti-spaghetti** review.

---

## Step 1 — Establish the change set

Prefer **git** scope (adjust base as agreed; default is last merge base or `main`):

```bash
git fetch origin
git diff origin/main...HEAD
```

If work is uncommitted:

```bash
git diff
git diff --cached
```

Read the **full diff** for touched files. If the diff is huge, split the review by directory and note ordering.

---

## Step 2 — Confirm the machine gate

Run your project's linter and formatter check (or your merge-ready command). If either fails, **stop** and fix; this skill does not override your project's lint config.

Understand any **tiered rules** in your linter config (e.g. stricter complexity limits for `src/lib/**` vs relaxed for legacy modules). New code should still meet the stricter bar even in relaxed zones.

---

## Step 3 — Change-aware checklist (human)

Apply to **lines changed and their immediate call sites** (expand context in the editor as needed).

| Topic                     | What to verify                                                                                                                                   |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Single responsibility** | Each new or edited function does one thing; deep nesting (`if` inside `if` inside `if`) is a smell — prefer early returns or extracted helpers.  |
| **Length**                | Even where linting allows longer functions, **new** blocks should not balloon; extract helpers with stable names.                                |
| **Naming**                | Names state intent (`computeNextFire` not `data`); avoid abbreviations unless domain-standard.                                                   |
| **Comments**              | Comment **why**, not what the syntax already says. Non-obvious invariants, timing, or quirks deserve a short note.                               |
| **Duplication**           | Copy-paste across branches in the same diff → consider one helper.                                                                               |
| **Types**                 | Avoid `any` (TypeScript) or equivalent weak types; narrow unions at boundaries (messages, storage).                                              |
| **Error handling**        | User-visible failures should be actionable; silent catches need a one-line rationale.                                                            |

---

## Step 4 — Output

Produce a short result for the user or handoff note:

- **PASS** — Diff is proportionate, readable, and consistent with project conventions; no new debt worth blocking.
- **WARN** — Minor issues (naming, missing comment on tricky spot, small extraction); suggest follow-up.
- **FAIL** — Spaghetti, unsafe patterns, or violations of project rules; **must** fix before merge even if linting is green.

Cite **file paths** and, when useful, **function names** or line ranges from the diff (not line numbers that drift after edits).

---

## Relationship to other skills

| Skill                        | Role                                                               |
| ---------------------------- | ------------------------------------------------------------------ |
| **code-reviewer**            | Correctness, security, tests, docs.                                |
| **code-quality-gate** (this) | Diff-scoped maintainability and industry-style structure.          |
| **tech-debt-evaluator**      | Prioritized backlog of larger refactors.                           |
| **tester** / **TEST_TDD**    | Automated red/green; not a substitute for reading the diff.        |

---

## Tightening the gate over time

When large hotspots are refactored into smaller modules:

1. Lower **complexity** / **max-lines** in your linter config for those paths.
2. Remove or narrow **overrides** so CI enforces the same bar everywhere.

Do not weaken lint config to greenwash failing quality; fix or track in **tech-debt** with a ticket/story reference.
