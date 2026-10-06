---
name: incident-triager
description: Evidence-driven debugging/incident workflow: reproduce, isolate, minimize, fix, verify, and prevent regressions.
---

# Incident Triager — Debug Without Drift

Use this skill when:
- Something is broken, flaky, or non-deterministic
- You’re doing “investigation work” that can easily drift
- You need to hand off a bug efficiently

Goal: get to a minimal reproducible case, a minimal fix, and a verifiable outcome.

---

## Triage protocol (must follow in order)

1. **Define the incident**
   - Symptom (what is observed)
   - Expected behavior
   - Severity (user-blocking / degraded / cosmetic)

2. **Reproduce**
   - Exact steps
   - Environment (OS/version/build, branch, config)
   - Is it deterministic? (Y/N)

3. **Isolate**
   - Smallest scope that still reproduces (file, module, step, config)
   - Identify the first bad point (when it starts failing)

4. **Hypothesize (bounded)**
   - List 1–3 plausible causes max
   - For each, define a test that could disprove it

5. **Fix (minimal)**
   - Prefer the smallest change that addresses the root cause
   - Avoid “refactor while here” unless explicitly requested

6. **Verify**
   - Rerun the reproduction steps
   - Add/extend a cheap regression check if possible (Tier 1)

7. **Document**
   - Record the outcome and the prevention check
   - Summarize in a handoff note using session-summarizer if handing off

---

## Output format (for handoff)

- **Repro**: steps + determinism
- **Root cause**: (if known)
- **Fix**: what changed (summary)
- **Verification**: what was run and result
- **Next**: if still open, what to try next (bounded)

