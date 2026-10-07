---
name: session-summarizer
description: Leaving-agent protocol. Produces a compressed, decision-first handoff that preserves intent and next steps while stripping execution noise.
---

# Session Summarizer — Leaving Agent Protocol

Use this skill when ending a session, reducing context, handing work to a new agent, or closing **UCPH** / **CMPH**.

Goal: transfer **working state** with **minimal tokens**, in a shape the human reads in chat.

A gitignored note with no matching chat brief is an incomplete handoff. Do not run the review swarm on UCPH or CMPH ([wrap-on-command.mdc](../../rules/wrap-on-command.mdc)).

---

## Progressive summarization (what to keep vs strip)

Always keep (highest value per token):
1. **Decisions** (what we chose)
2. **Rationale** (why we chose it)
3. **Next steps** (what to do next, in order)
4. **Acceptance criteria / validation** (how to know it’s done)

Keep only summary-level:
- Actions taken (headline only)
- File paths changed (key files only)

Strip unless directly decision-relevant:
- Long logs
- Step-by-step execution transcripts
- Repeated context already captured in tracked docs

---

## Handoff note budget

Default target: **≤ 500 words** (≈ 700 tokens).

If you exceed the budget, remove execution details first.

---

## Chat and file (both required on UCPH and CMPH)

1. **First line of chat (verbatim):** `You are the receiving agent.`
2. Paste the **Receiver brief** as the closing message — same first line repeated at top of brief. Second person throughout (**You** …). Headings: [`.cursor/handoff/_template.md`](../../handoff/_template.md).
3. Write that **same body** to `doc/handoff/NNNN-HANDOFF-YYYY-MM-DD_HHmm.md` and copy it to `.cursor/handoff/NNNN-handoff-YYYY-MM-DD_HHmm.md`.
4. Last line of chat and note: `**Filename:** \`doc/handoff/NNNN-HANDOFF-YYYY-MM-DD_HHmm.md\``
5. Sync the **Git** rows into `AGENT_HANDOFF.md` → *Current state* (tracked).

`NNNN` is new and monotonic. Never reuse it. Never edit a prior note to “update” it. Timestamp is local `YYYY-MM-DD_HHmm`.

Required headings: Objective, Git (Story, Version, On, Do not, Next), Decisions, In scope / Out, Acceptance, Next steps, Measured, Filename.

Every brief includes this line verbatim, on its own line between **Next steps** and **Measured**. Do not drop it and do not paraphrase it: `write thorough tests. red, integration, find missing ones of both kinds.`

**Measured** is numbers versus the prior run, or `none this turn`. **Gates** live in the SWAT file only, not in the CMPH chat.

Keep the chat brief near the worked example length. Strip logs first.

---

## “Green and Clean” exit check

Before ending:
- Chat has the full Receiver brief, not only “see the handoff file.”
- Filename line is present and `NNNN` is unused.
- Tracked Git truth matches **On**.
- Acceptance and any measured numbers are real, not placeholders.
