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

- **Phase 1 / Sprint 1:** Epic 1.1 and Epic 1.2 are complete. Epic 1.3 is in progress — [doc/sprints/SPRINT_1.md](doc/sprints/SPRINT_1.md)
- **Epic 1.1:** complete — 2026-10-06. Empty-deck rule: [doc/requirements/GAME_RULES.md](doc/requirements/GAME_RULES.md).
- **Epic 1.2:** complete — 2026-10-06. A player starts empty. Two or more players are dealt 10 cards, one at a time. Ranks 3–9 score 5, a 10 through King scores 10, an ace scores 15, and a two or a joker scores 20. A hand of 4, Jack, Ace, and Joker totals 50 and that penalty adds onto `total_score`. `points` stays 0. **1.2.4.2** and **1.2.4.3** stay deferred.
- **1.3.1:** parked on `feature/1.3.1-take-discard` (not merged). `Action::TakeDiscard` moves the discard pile's top card into the player's hand. Cards under that top stay. The draw pile stays. `points` and `total_score` stay 0.
- **Next:** 1.3.2 Pushing a Discard, only when asked. A push moves the top discard to the next player's hand.
- **Full backlog:** [doc/BACKLOG.md](doc/BACKLOG.md)
- **Commentary / deltas:** [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md) (integration-chain Epic 1.9 suggested; Google export quirks)

## Version

Plan coordinate starts at **0.1.0** after first product ship on `main`. See [doc/VERSIONING.md](doc/VERSIONING.md).

Keep in sync with AGENT_HANDOFF *Current state* and [doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md).
