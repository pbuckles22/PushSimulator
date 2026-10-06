# PM_PLAN — PushSimulator

Operating model: `.cursor/rules/` ship commands (UCPH, CMPH, SWAT), human check, plan-coordinate versioning. Do not delete them.

## Phases

| Phase | Focus | Primary crate / surface |
|-------|--------|-------------------------|
| **1** | Rust core engine (TDD) | `push_core` |
| **1b** | WASM debug table | `push_wasm` + `viewer/` |
| **2** | Monte Carlo + bots | `push_sim` |
| **3** | iOS local play | UniFFI + SwiftUI |
| **4** | Multiplayer | Axum + WebSockets |

## Current

- **Phase 1 / Sprint 1:** Epic 1.1 is complete. Epic 1.2 is next when asked — [doc/sprints/SPRINT_1.md](doc/sprints/SPRINT_1.md)
- **Epic 1.1:** complete — 2026-10-06. Empty-deck rule: [doc/requirements/GAME_RULES.md](doc/requirements/GAME_RULES.md).
- **Next:** 1.2.3 Point calculations, only when asked. 1.2.2 `deal_initial_hands` is on `feature/1.2.2-deal-hands` (one card at a time to 2 or more players, 10 each). 1.2.1 `Player::new` is on `feature/1.2.1-player-init`.
- **Full backlog:** [doc/BACKLOG.md](doc/BACKLOG.md)
- **Commentary / deltas:** [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md) (integration-chain Epic 1.9 suggested; Google export quirks)

## Version

Plan coordinate starts at **0.1.0** after first product ship on `main`. See [doc/VERSIONING.md](doc/VERSIONING.md).

Keep in sync with AGENT_HANDOFF *Current state* and [doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md).
