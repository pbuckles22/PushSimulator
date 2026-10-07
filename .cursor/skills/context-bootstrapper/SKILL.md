---
name: context-bootstrapper
description: Receiving-agent protocol. Boots a new agent into the minimum correct context (select + isolate) and produces a clear next-step plan without guessing.
---

# Context Bootstrapper — Receiving Agent Protocol

Use this skill when starting work on this repo, resuming after a break, switching to a new “feature-agent”, or when context feels bloated/confusing.

**You are the agent to execute the below hand off.** If the prior session pasted a Receiver brief, it was written **for you** — not for you to re-issue. Do not “confirm the handoff landed” or write a new handoff unless you are closing with UCPH/CMPH.

Goal: reach a **confident, bounded next step** using **minimal context**.

---

## Bootstrap order (read in this order)

1. **Project baseline (always-on context)**  
   - `.cursor/rules/always.mdc`
   - `AGENT_HANDOFF.md`

2. **Current phase / feature truth** (choose the one that matches the user’s goal)  
   - `PM_PLAN.md` (phase/scope)
   - `TEST_PLAN.md` (Tier 1 / Tier 2 validation gates)

3. **Most recent session handoff note** (if present)  
   - `doc/handoff/NNNN-HANDOFF-YYYY-MM-DD_HHmm.md` or `.cursor/handoff/NNNN-handoff-YYYY-MM-DD_HHmm.md` (**highest `NNNN`**)  
   - Expect a **Receiver brief** (Objective, Git, Decisions, In/Out, Acceptance, Next steps, Measured, Filename). Spec: [`.cursor/handoff/_template.md`](../../handoff/_template.md).  
   - Read **that file only** for the session delta. Older notes are history. Do not use them to second-guess a later **Git** block.

4. **If the task is code-touching:** read the smallest set of files necessary to act safely.

---

## Git truth — do not re-prove the last ship

`AGENT_HANDOFF.md` → *Current state* plus the latest brief **Git** table are the land record.

- If **On** is **`origin/main @ sha`** and story **N.M** is `[x]`: **start at Next steps.** Do not `git log`, `git fetch`, or re-merge to confirm. Do not narrate that you checked it landed.
- `git status` is only to see if *your* tree is dirty before you edit.
- Re-check git only when Git truth is missing, says not merged or WIP, the user asked “is it on main?”, or you are the agent who will merge.

Mismatch (the brief says landed, Current state still names a feature branch): **Current state on the branch you have checked out wins.** Ask. Do not silently re-do the merge.

---

## Produce the “Receiver Brief” (what you must write next)

After reading, produce a short brief with:

- **Objective**: one sentence; restate the user goal precisely.
- **Scope**:
  - **In scope**:
  - **Out of scope**:
- **Constraints**: branch policy, “no guessing”, validation tier.
- **Acceptance criteria**: 2–5 verifiable checks.
- **Git**: one line from Git truth (`origin/main @ sha` or not merged, plus the next story). If that ship already landed, do not put “confirm it is on main” in Next steps.
- **Next steps**: 3–7 bite-sized steps, each with a validation hook.
- **Open questions**: only if something blocks safe progress.

If any required inputs are missing, stop and ask before acting.

---

## Token budget guidance (cheap but effective)

Preferred context payload for a new session:

- **Tracked truth**: `AGENT_HANDOFF.md` + the relevant plan doc(s) (project state)
- **One handoff note**: latest only (session delta)
- **Only the files you’re editing** (and their direct dependencies)

Avoid:
- Full transcript dumps
- Large logs unless they directly change decisions
- Repeating old execution details already captured in tracked docs
