# Feature branch archaeology

When someone says “it used to work” and nobody can find the commit, the fix is traceability.

## Problem

- **`main`** moves. The working slice often lived on a **feature branch**.
- A runtime log from one session is not a regression suite.
- A public version maps to a **ship**, not always to one story id and not always to one git sha.
- Searching only **`main`** misses the branch that held the working slice.

## Practice

| Step | Do |
|------|----|
| **Branch** | One PM story → `feature/<story-id>-short-topic` ([one-story-one-ship](../../.cursor/rules/one-story-one-ship.mdc)). |
| **Ship commit** | Subject names the story and the public version. |
| **Truth docs** | Same commit when they said **UCPH** or **CMPH**: `PM_PLAN`, TEST_PLAN, `doc/PROJECT_STATUS.md`, AGENT_HANDOFF *Current state*. |
| **Push** | `git push -u origin <branch>` before a human-check wait or a handoff. |
| **Merge** | **`main`** only after human **PASS** (or waive) and **CMPH** ([no-auto-merge-main](../../.cursor/rules/no-auto-merge-main.mdc)). |
| **After merge** | **CMPH** (no **D**): **keep** local and remote. **CMPHD**: delete after `main` is pushed. Record the land sha in PROJECT_STATUS. |
| **Find it later** | `git log main --oneline --grep='<story>'`, `git branch -a`, handoff notes, PROJECT_STATUS land sha, TEST_PLAN. |

## Keep the branch

**CMPH** (no **D**):

1. Merge the feature branch into **`main`**.
2. **Keep** local and remote. Do not delete.
3. Record in PROJECT_STATUS: land sha.

**CMPHD** (the **D** means delete): same land, then delete local and remote.

## What does not help

- Merging without the truth docs (a version bump alone is not enough).
- Deleting the branch before PROJECT_STATUS records the land sha.
- Debugging in the runtime a lock that has no Tier 1 name ([testing.mdc](../../.cursor/rules/testing.mdc)).
