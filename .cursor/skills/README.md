# Project skills

All agent skills and source-of-truth docs live here.

| Item | Path | When to use |
|------|------|-------------|
| **DEV_GUIDE** | [DEV_GUIDE.md](DEV_GUIDE.md) | Tech stack, architecture, repo layout, conventions. |
| **TEST_TDD** | [TEST_TDD.md](TEST_TDD.md) | What to test; TDD. |
| **DESIGN_SYSTEM** | [DESIGN_SYSTEM.md](DESIGN_SYSTEM.md) | Visuals, motion, haptics (placeholders). |
| **techwriter** | [techwriter/SKILL.md](techwriter/SKILL.md) | Editing README, AGENT_HANDOFF, or internal docs. |
| **tester** | [tester/SKILL.md](tester/SKILL.md) | Adding or changing tests; evidence loop (logs → Tier 1); black-box only. |
| **green-and-clean** | [green-and-clean/SKILL.md](green-and-clean/SKILL.md) | Operating model: no guessing, bounded scope, verifiable steps, clean context. |
| **context-bootstrapper** | [context-bootstrapper/SKILL.md](context-bootstrapper/SKILL.md) | Receiving-agent bootstrap: minimal read order + receiver brief. |
| **session-summarizer** | [session-summarizer/SKILL.md](session-summarizer/SKILL.md) | Leaving-agent: UCPH/CMPH Receiver brief in chat and `doc/handoff/`. |
| **tech-debt-evaluator** | [tech-debt-evaluator/SKILL.md](tech-debt-evaluator/SKILL.md) | Assessing tech debt; refactor/sprint planning. |
| **code-reviewer** | [code-reviewer/SKILL.md](code-reviewer/SKILL.md) | Reviewing diffs/PRs; correctness, conventions, tests. |
| **code-quality-gate** | [code-quality-gate/SKILL.md](code-quality-gate/SKILL.md) | Diff-scoped maintainability: readability, structure, anti-spaghetti (companion to code-reviewer). |
| **tech-lead** | [tech-lead/SKILL.md](tech-lead/SKILL.md) | Sequencing work, definition of done, risks, cross-cutting coordination. |
| **eval-engineer** | [eval-engineer/SKILL.md](eval-engineer/SKILL.md) | Defining lightweight evaluations and acceptance criteria (make “green” objective); harvest evidence into Tier 1. |
| **risk-manager** | [risk-manager/SKILL.md](risk-manager/SKILL.md) | Lightweight risk register (impact/likelihood/triggers/mitigations/rollback). |
| **release-manager** | [release-manager/SKILL.md](release-manager/SKILL.md) | Merge-ready/release discipline, rollback, and short release notes. |
| **security-reviewer** | [security-reviewer/SKILL.md](security-reviewer/SKILL.md) | Secrets hygiene, safe logging, least privilege, and lightweight security review. |
| **incident-triager** | [incident-triager/SKILL.md](incident-triager/SKILL.md) | Evidence-driven incident/debug workflow: repro, isolate, minimal fix, verify. |
| **pm-governance** | [pm-governance/SKILL.md](pm-governance/SKILL.md) | Sprint planning, scope, quality gates. |
| **ui-ux** | [ui-ux/SKILL.md](ui-ux/SKILL.md) | Screens, animations, layout, design tokens. |
| **game-readiness** | [game-readiness/SKILL.md](game-readiness/SKILL.md) | Playability, clarity, feedback, scale-readiness (e.g. games). |
| **visual-match** | [visual-match/SKILL.md](visual-match/SKILL.md) | Matching UI to reference examples (screenshots, specs). |
| **github-feature-workflow** | [github-feature-workflow/SKILL.md](github-feature-workflow/SKILL.md) | Feature branch → gate → commit → push branch. Merge to `main` only on CMPH. **CMPHD** deletes the branch. |

Scope/sprints: [PM_PLAN.md](../../PM_PLAN.md). Product epics or roadmap: your choice (e.g. `doc/plan/`). Session handoff files: often gitignored — see [.cursor/handoff/README.md](../handoff/README.md). Rules in `.cursor/rules/` reference this directory.
