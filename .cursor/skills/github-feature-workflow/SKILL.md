---
name: github-feature-workflow
description: >-
  Short-lived feature branches; TDD + lint + merge-ready command as exit criteria before
  commit; push the feature branch. Merge to main only after CMPH permission.
  Do not default to asking the user to open a PR. Use when implementing a feature or non-trivial
  fix, when the user asks for branch/git workflow, or after substantial edits
  that should not stay uncommitted.
---

# Git / GitHub feature branch workflow

**Policy in this template:** [AGENT_HANDOFF.md](../../AGENT_HANDOFF.md) — `main` is the integration branch; the **merge-ready command** is the quality bar. This skill defines **how work is finished**: tests (TDD where the tier applies), lint, format, merge-ready green, then **commit** (and **push** per AGENT_HANDOFF). **Pull requests are not part of the default exit** for this template unless the user explicitly asks for a PR or GitHub-based review.

## Do not prompt for PRs (agents)

- **Do not** tell the user to "open a PR," paste `github.com/.../pull/new/...` links, or treat opening a PR as the normal end-of-task step.
- **Do** treat **green merge-ready command** plus project test discipline ([tester](../tester/SKILL.md), [TEST_TDD.md](../TEST_TDD.md), [code-quality-gate](../code-quality-gate/SKILL.md) when relevant) as **merge-ready / commit-ready**.
- **If** the user says they want a PR, GitHub review, or external reviewers: then describe or open the PR as they asked.

**Completion mental model:** one branch ≈ one purpose → merge-ready green → **commit** → **push the feature branch**. If they said **CMPH** (or WRAP / land / merge to main / ship to main), merge to `main` and push in that same close-out. Do not ask again. **CMPH** keeps the branch. **CMPHD** deletes it. Otherwise stop. No roundabout "please open a PR" unless they chose that path. See [wrap-on-command.mdc](../../rules/wrap-on-command.mdc).

## When to apply

- User asks for a **feature branch**, **commit**, **push**, or **merge** (or explicitly **PR**).
- A **coherent slice** of work is done (e.g. one epic story, one bugfix) and should be **recorded in git** before the session ends.
- **Do not** create branches for one-line typo fixes unless the user wants it.

## Branch naming

- Prefer: `feature/<short-kebab-topic>` (e.g. `feature/epic-10-3-pause-keys`) or `fix/<issue-or-topic>`.
- Avoid ultra-long names; include epic/story id if it helps **PM_PLAN** / roadmap traceability.

## Branch-first rule (agents)

- **Do not** stack substantial implementation on **`main`** and only then create a feature branch to "check in." That bypasses a proper branch history and the CI-before-commit discipline.
- **Do** start each non-trivial slice on a **new branch**: `git fetch origin`, `git checkout main`, `git pull`, `git checkout -b feature/<topic>`, then implement, run merge-ready, commit, push the feature branch. Merge to `main` only after CMPH permission ([no-auto-merge-main.mdc](../../rules/no-auto-merge-main.mdc)).
- If work already landed on **`main`** without a branch, recover discipline going forward; optionally **`git checkout -b feature/<topic>`** from **`main`** before the _next_ slice so new commits are branch-first.

## Exit criteria before commit (ship bar)

Treat these as satisfied **before** `git commit` on anything beyond trivial doc typos (adjust if the user narrows scope):

1. **TDD / tests** — [TEST_TDD.md](../TEST_TDD.md) + [tester](../tester/SKILL.md): failing test first when the changed surface is covered by Tier 1 or Tier 2; suite green for what you touched.
2. **Lint + format** — covered by your merge-ready command (linter, Prettier/formatter check).
3. **Full merge-ready** — your project's merge-ready command green (tests, build, E2E if applicable). If the stack **compiles**, run **and count** every documented compile (typically test/Debug **and** Release); a green test compile does not replace Release. Non-compiling stacks (scripts, twitchurl*, DJ tools) skip the counter.
4. **Quality** — For non-trivial edits, use [code-quality-gate](../code-quality-gate/SKILL.md) as appropriate (readability, complexity, obvious foot-guns).

When 1–4 are green: **commit** and **push the feature branch**. That is the **done** state until they grant CMPH. It is not "waiting for the user to open a PR," and it is not an automatic merge.

## Pre-checkin and pre-next-feature checks

- **Before commits** that change behavior or tests (not one-line doc typos): run your **merge-ready command** — same gate as [AGENT_HANDOFF.md](../../AGENT_HANDOFF.md) for **`main`**.
- **Before merging** to **`main`**: merge-ready green on the feature branch.
- **Before starting the next feature** after a merged story: run merge-ready on updated **`main`** (`git checkout main && git pull`) so Tier 1 + Tier 2 still pass against **origin/main** before new work begins (catches drift if the final gate was skipped).
- **After push to `main`:** run **Verify CI after push** (above) when the repo has GitHub Actions — users may learn about failures via email before agents do; agents must check explicitly.

## Standard sequence

1. **Start from current `main` (or agreed base):** `git fetch origin` when remote exists; `git status` (clean or intentional WIP).
2. **Create branch:** `git checkout -b <name>` — **before** writing production code for the slice.
3. **Implement** with tests as required by [tester skill](../tester/SKILL.md) and [TEST_TDD.md](../TEST_TDD.md) (**red → green** when that tier applies).
4. **Gate before commit:** meet **[Exit criteria before commit](#exit-criteria-before-commit-ship-bar)**; your merge-ready command is the all-in-one gate here.
5. **Commit:** clear, imperative subject line; body only if context helps (what/why, not noise). One logical commit per slice is fine; multiple small commits are fine if they tell a story.
6. **Push:** `git push -u origin <branch>` (first time); later `git push` on that branch.
7. **Stop before `main` unless CMPH:** The push is the feature branch only until they say **CMPH**, WRAP, land, or ship to main. Then merge in that same close-out ([no-auto-merge-main.mdc](../../rules/no-auto-merge-main.mdc)): `git checkout main && git pull && git merge <branch> && git push origin main`, then paste the Receiver brief ([`.cursor/handoff/_template.md`](../../handoff/_template.md)). **Do not** nudge them toward opening a PR by default. **Keep** the feature branch unless they typed **CMPHD**.
8. **Verify CI after push** (when GitHub Actions or equivalent exist): agents do **not** receive GitHub email notifications. After pushing to `main` (or any branch with CI), confirm the remote run — do not treat local merge-ready alone as ship-complete.

   ```bash
   # Get latest run ID, then watch (required in non-interactive/agent sessions)
   RUN=$(gh run list --repo OWNER/REPO --limit 1 --json databaseId -q '.[0].databaseId')
   gh run watch "$RUN" --exit-status

   gh run list --repo OWNER/REPO --limit 1              # quick status check
   gh run view <run-id> --log-failed                     # diagnose failures
   ```

   Local tests can pass while CI fails (different OS, env vars, path semantics). Fix and push again until CI is green before declaring the slice done.

9. **After merge to `main`:** checkout `main`, `git pull`. **Keep** local and remote unless they typed **CMPHD** (the **D** means delete). Record the land sha in `doc/PROJECT_STATUS.md`. See [doc/git/Feature_Branch_Archaeology.md](../../../doc/git/Feature_Branch_Archaeology.md).
10. **Update product state** if scope shipped: [PM_PLAN.md](../../PM_PLAN.md) and your product plan — not only git history.

**PR (explicit opt-in only):** If and only if the user asked for a PR or GitHub review, add a PR with a short title and note merge-ready green. Otherwise skip PR language entirely.

## What this skill does _not_ do

- Replace **code review** or **handoff** — mid-epic land is **CMPH** (short Receiver brief). The review swarm is **SWAT** ([handoff-checklist.mdc](../../rules/handoff-checklist.mdc)).
- **Invent a PR step** — PRs are not the default completion signal. **Merge-ready + commit + push the feature branch** is. Merge to `main` only after CMPH permission.
