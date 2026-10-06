---
name: techwriter
description: Creates and edits project documentation for both public and internal audiences. Use when writing or updating README.md, AGENT_HANDOFF (or AGENTS.md), CONTRIBUTING, or internal docs (e.g. PM_PLAN, DEV_GUIDE, TEST_TDD, DESIGN_SYSTEM). Covers structure, tone, and what to include for each doc type.
---

# Techwriter — Project Documentation

Use this skill when creating or revising project docs so they stay consistent and useful for humans and agents.

---

## Doc types and audience

| Doc | Audience | Goal |
|-----|----------|------|
| **README.md** | Users, contributors, new devs | Orient quickly; run and contribute. |
| **AGENT_HANDOFF.md** / **AGENTS.md** | AI agents, devs resuming work | Hand off context: what's done, what's next, where truth lives. |
| **Internal** (PM_PLAN in root; .cursor/skills/: DEV_GUIDE, TEST_TDD, DESIGN_SYSTEM) | Team and agents | Single source of truth for scope, tech, tests, design. |

---

## README.md (public-facing)

**Include:** Project name and one-line description; how to run; main stack; link to more docs.

**Style:** Clear, scannable. Short paragraphs, bullets, code blocks for commands.

---

## AGENT_HANDOFF.md (agent and handoff)

**Include:** Repo purpose; source-of-truth links (PM_PLAN, DEV_GUIDE, TEST_TDD, DESIGN_SYSTEM); run and test commands; handoff protocol. Put **what shipped** in **PM_PLAN** / product plan on **`main`**; **session** narrative stays **local** (gitignored `*-HANDOFF-*.md` / `handoff-*.md`) — not in AGENT_HANDOFF.

**Session handoff files:** Naming and timestamps are defined in **AGENT_HANDOFF.md**. The mandatory steps live in **`.cursor/rules/handoff-checklist.mdc`**. When you change **how** handoffs are named or structured, update **AGENT_HANDOFF.md** (and the checklist if steps change); use **techwriter** for tone/structure.

**Style:** Dense but structured. Headings and bullets so agents can jump to the right section.

### Dual-audience blurbs (pre-commit chat, optional PR description)

**AGENT_HANDOFF.md** → _Git workflow_ may require **two** blurbs before commit (unless the user waived): one for **maintainers**, one for **people who only use the shipped product**.

| Blurb      | Voice                                                                                                                  | Avoid in the product-user blurb                                                                                              |
| ---------- | ---------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **First**  | Plan and engineering context; epic/story, risk, pointers to files or modules if helpful.                               | (No extra restriction.)                                                                                                      |
| **Second** | Plain, concrete, user-outcome only—like short **release notes** ("You can still …", "Nothing changes for you when …"). | File paths, symbols, DOM/CSS/HTML, APIs, test or CI tool names, git vocabulary—unless the user asked for a technical answer. |

**Habit:** After drafting, ask whether a **non-developer** who uses the product would understand the **second** blurb without opening the repo. Rewrite that blurb until the answer is yes.

---

## Internal docs

Keep in sync with the code. Update when behavior, stack, or process changes. Use the same tone: concise, actionable, one concern per section.
