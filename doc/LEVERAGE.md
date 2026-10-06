# Leverage

Before writing new machinery, pick an existing wheel. Invent only when this repo’s invariants make that wheel unusable.

The decision log is [LEVERAGE_REGISTER.md](LEVERAGE_REGISTER.md). Do not re-scout a story that already has a row. If the wheel changes, update the row in the same ship.

## Order

1. **Find** — look for a native API, a standard library, or a known pattern that already does this.
2. **Fit** — check it against this repo’s invariants (allocation, threading, dependencies, license).
3. **Implement** — call it, adapt the idea, or invent. Say which one in the register.

## Decision words

| Word | Meaning |
|------|---------|
| **reuse** | Call it as-is |
| **adapt** | Same idea, this repo’s constraints |
| **invent** | We own it, and the register says why the existing wheel failed |

Ship a new dependency only after the native path fails a check this repo already has, and the user accepts that dependency.

## Register row

| Column | Meaning |
|--------|---------|
| **Leverage** | The API, pattern, or project |
| **Decision** | reuse, adapt, or invent |
| **Invent only if** | The gate that would justify new machinery |

Status: `[x]` shipped, `[~]` in flight, `[ ]` backlog.
