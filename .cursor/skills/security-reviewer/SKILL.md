---
name: security-reviewer
description: Lightweight security review for agent work: secrets hygiene, safe logging, least privilege, and threat-model-lite checks.
---

# Security Reviewer — Lightweight Safety Gate

Use this skill when:
- Handling credentials, keys, tokens, licenses, or personal data
- Adding logging, telemetry, handoff notes, or diagnostics
- Changing network exposure, firewall rules, remote access, or installs

Goal: prevent accidental secret leaks and reduce obvious security footguns.

---

## Quick checks (must do)

1. **Secrets hygiene**
   - Do not commit secrets (`.env`, tokens, passwords, API keys).
   - Redact secrets from logs, screenshots, and handoff notes.
   - If a secret might have been exposed, treat it as compromised: rotate it.

2. **Safe logging**
   - Log **events and outcomes**, not raw secrets or full environment dumps.
   - Prefer minimal excerpts with context over full log spam.

3. **Least privilege**
   - If elevation/admin is required, document **why** and minimize the elevated surface area.
   - Avoid enabling services/features globally when scoped changes suffice.

4. **Attack surface**
   - If opening ports / enabling remote access, document:
     - what changed, how to verify, and how to revert

---

## Output format (for handoff / PR summary)

- **Security review**: PASS/WARN/FAIL
- **Notes**: 1–5 bullets, include mitigations/rollbacks if WARN/FAIL

