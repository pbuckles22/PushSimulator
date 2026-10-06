# Agent handoff — PushSimulator

## Purpose

**Push** card-game engine and simulator: pure Rust core, WASM debug table (Phase 1b), Monte Carlo (Phase 2), SwiftUI + UniFFI iOS (Phase 3), Axum multiplayer (Phase 4). Agentic layer from [AgenticTemplate](https://github.com/pbuckles22/AgenticTemplate).

**Repo:** https://github.com/pbuckles22/PushSimulator

---

## Source of truth

- **Status:** [doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md)
- **Architecture:** [doc/ARCHITECTURE.md](doc/ARCHITECTURE.md)
- **Plan commentary:** [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md)
- **Backlog / Sprint 1:** [doc/BACKLOG.md](doc/BACKLOG.md), [doc/sprints/SPRINT_1.md](doc/sprints/SPRINT_1.md)
- **Rules:** [doc/requirements/GAME_RULES.md](doc/requirements/GAME_RULES.md)
- **Scope / phases:** [PM_PLAN.md](PM_PLAN.md)
- **Skills:** [.cursor/skills/](.cursor/skills/)

## Current state

- **Ship:** 1.2.3 is parked on **`feature/1.2.3-pip-scoring`** (not merged). 1.2.2 stays on **`feature/1.2.2-deal-hands`**. 1.2.1 stays on **`feature/1.2.1-player-init`**. Epic 1.1 stays on **`main` @ 94d0183**. `feature/1.1.4-empty-reshuffle` is kept. `feature/1.1.4-shuffle-draw` is kept. Prior park: `origin/feature/1.1.2-wild-cards @ 71c50c7`.
- **Phase:** 1 — Rust core. **Next story:** 1.2.4 Face card and ace scoring, only when asked. 1.2.3 `Card::get_penalty_value` returns 5 for ranks 3 through 9. A 2, a face card, an ace, or a joker still panics. 1.2.2 deals one card at a time to 2 or more players, 10 cards each. A one-player deal is refused. The discard pile stays put. 1.2.1 `Player::new` is on `feature/1.2.1-player-init` (0 points, `is_on_board = false`, empty hand).
- **Version:** no bump (workspace 0.1.0; not a user-visible build).
- **Client path:** SwiftUI + UniFFI (not Flutter).
- **Handoff:** The next session gets **`You are the agent to execute the below hand off.`** as first line; brief is second person.

## Run and test

Requires Rust (`rustup`). Add `%USERPROFILE%\.cargo\bin` to PATH on Windows if `cargo` is missing.

```bash
cargo test -p push_core
cargo test
wasm-pack build push_wasm --target web --out-dir ../viewer/pkg
cargo run -p push_sim
```

Viewer: `cd viewer && npx --yes serve .`

**Merge-ready gate:** `cargo test -p push_core`

## Conventions

- Game rules live only in `push_core`.
- Strict TDD for Epic 1.x; integration chains after Epic 1.8.
- **Handoff protocol:** First chat line **`You are the agent to execute the below hand off.`** Body uses **You** …; never tell the receiver to write a handoff. Template: [`.cursor/handoff/_template.md`](.cursor/handoff/_template.md).

## Git workflow

1. Integration branch: **`main`**
2. Feature branches: `feature/<story-id>-short-topic` — merge only after **CMPH**
3. Gate before push: `cargo test -p push_core`
4. **UCPH** pushes feature branch only; **CMPH** merges to `main`

## Handoff protocol

UCPH/CMPH → Receiver brief for the **next** agent (see template). SWAT → handoff-checklist + review swarm.

## Epic close (SWAT)

Follow [.cursor/rules/epic-close.mdc](.cursor/rules/epic-close.mdc) only on **SWAT** or the epic’s last CMPH.
