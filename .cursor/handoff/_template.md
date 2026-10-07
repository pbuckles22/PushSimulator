# Receiver brief — required handoff shape

**Audience:** The **receiving agent** — the *next* Cursor session that opens after this one. **Not** the human operator. **Not** the agent writing this brief.

**Every UCPH or CMPH close** must:

1. Open chat with: **`You are the agent to execute the below hand off.`** (first line, verbatim).
2. Paste this brief **in chat** (second person: “You …”, not “The next agent should …”).
3. Write the **same body** to the handoff note.
4. Include this line in the brief, verbatim, on its own line between **Next steps** and **Measured**: `write thorough tests. red, integration, find missing ones of both kinds.`

A one-line “landed / pushed” is not a handoff. Imperatives like “Write a handoff” or “Update docs” confuse the receiver into thinking they are the *leaving* agent — forbidden.

- **UCPH:** feature-branch park. **On** is `origin/feature/…` (not merged).
- **CMPH:** landed on `main`. **On** is `origin/main @ sha`.

Do **not** run code-reviewer, dead-code, or tech-debt on mid-epic **UCPH** / **CMPH**. **Gates** (PASS/WARN) only on **SWAT** / epic close, and only in the file.

## Filename (mandatory — last line of chat and note)

| Location | Pattern |
|----------|---------|
| Prefer | `doc/handoff/NNNN-HANDOFF-YYYY-MM-DD_HHmm.md` |
| Copy | `.cursor/handoff/NNNN-handoff-YYYY-MM-DD_HHmm.md` |

- `NNNN` = next unused monotonic serial (`0001`, `0002`, …). Never reuse. Never edit an old note in place.
- `YYYY-MM-DD_HHmm` = local 24h time.
- End the brief with: `**Filename:** \`doc/handoff/NNNN-HANDOFF-YYYY-MM-DD_HHmm.md\``

---

## Worked example (docs-only close) — copy this density

You are the agent to execute the below hand off.

Receiver brief (0.2 closed)

**Objective:** You start the next story only when the human asks. Do not re-prove this land.

**Git**

| | |
|--|--|
| **Story** | 0.2 `[x]` |
| **Version** | no bump (rules only) |
| **On** | `origin/main @ abc1234` |
| **Do not** | re-merge 0.2, or `git log` to confirm |
| **Next** | wait until asked |

**Decisions:** Ship commands are UCPH / CMPH / SWAT; merge to main needs CMPH permission; public version counter is not the story id.

**In scope next:** whatever they ask. **Out:** starting the next story during this pause.

**Acceptance (already met):** rules land on `main`; no runtime, so no human check.

**Next steps:** 1) Stop. 2) On the next ask, branch from this `main`.

write thorough tests. red, integration, find missing ones of both kinds.

**Measured:** none this turn

**Filename:** `doc/handoff/0001-HANDOFF-YYYY-MM-DD_HHmm.md`

---

## Blank (fill every heading)

You are the agent to execute the below hand off.

Receiver brief (N.M open | closed | WIP)

**Objective:** (one sentence: what **you** do next, and what you must not re-prove)

**Git**

| | |
|--|--|
| **Story** | N.M `[x]` / `[ ]` |
| **Version** | public version, or `no bump` |
| **On** | `origin/main @ <sha>` **or** `origin/feature/… @ <sha> (not merged)` |
| **Do not** | (the wasted step: re-merge, re-run the check, `git log`) |
| **Next** | (one story id, or pause until asked) |

**Decisions:** (decision and why)

**In scope next:** … **Out:** …

**Acceptance (already met | not yet):** Tier 1 …; human check …; version …

**Next steps:** 1) … 2) … 3) …

write thorough tests. red, integration, find missing ones of both kinds.

**Measured:** numbers vs the prior run, or `none this turn`

**Filename:** `doc/handoff/NNNN-HANDOFF-YYYY-MM-DD_HHmm.md`

## Gates (SWAT file only — omit on UCPH and CMPH)

| Gate | Result |
|------|--------|
| Code review | PASS / WARN / FAIL + one line |
| Dead code | none / removed … |
| Tech debt | none new / Do first: … |
| Tests | command and result |
| Readiness | N/A or one line |
| Security | N/A or PASS/WARN/FAIL |
