---
name: risk-manager
description: Maintain a lightweight risk register (top risks, triggers, mitigations, rollback) to prevent scope drift and improve decision quality.
---

# Risk Manager — Keep Risk Explicit

Use this skill when:
- Work affects stability, security, cost, or reliability
- You are about to make a risky change without a rollback
- You need to keep agents “on script” (risk clarity constrains drift)

Goal: keep a short, actionable risk register and make mitigation part of the plan.

---

## Risk register format (top 5 only)

Each risk must fit on 3–6 lines:

- **Risk**: ___
- **Impact**: Low/Med/High
- **Likelihood**: Low/Med/High
- **Trigger**: what signal indicates it’s happening
- **Mitigation**: what we do to prevent/limit it
- **Rollback**: how to safely revert

Prefer updating an existing tracked `RISKS.md` (or equivalent) rather than dumping risk analysis into chat.

---

## Rules

- If Impact is **High** and Rollback is unclear, **stop** and define rollback before proceeding.
- Keep mitigations **cheap** and directly tied to triggers.
- Risks that become reality should be captured as an **incident** (see incident triage protocol).

