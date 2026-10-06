# Project status — PushSimulator

**Last updated:** 2026-10-06

---

## Summary

Push card-game simulator: **Rust workspace** + **agentic foundation** + **WASM debug table shell**. Public repo: **https://github.com/pbuckles22/PushSimulator**. Foundation scaffold parked on feature branch (not merged to `main`).

---

## Active branch

| Branch | Role |
|--------|------|
| **`feature/foundation-scaffold`** | Foundation scaffold — pushed, **not merged** to `main` |
| **`main`** | Empty / not yet populated on remote |

---

## Completed (scaffold ship)

- AgenticTemplate layer + handoff receiver framing (`You are the receiving agent.`)
- Google docs organized under `doc/` (raw dump in `docs/`)
- Cargo workspace: `push_core`, `push_wasm`, `push_sim` — Tier 1 green (`cargo test -p push_core`, 1 passed)
- `viewer/` HTML/CSS/JS shell
- GitHub repo created (public)

---

## Next

| Item | Detail |
|------|--------|
| **Epic 1.1** | Card Definitions TDD — [sprints/SPRINT_1.md](sprints/SPRINT_1.md) |
| **Merge** | **CMPH** only when human wants `main` populated |

---

## Reading order

1. [CONTRIBUTING.md](../CONTRIBUTING.md)
2. This file · [PLAN_COMMENTARY.md](PLAN_COMMENTARY.md) · [ARCHITECTURE.md](ARCHITECTURE.md)
3. [AGENT_HANDOFF.md](../AGENT_HANDOFF.md)
