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

- **Ship:** Foundation scaffold on **`origin/feature/foundation-scaffold`** (not merged to `main`).
- **Phase:** 1 — Rust core. **Next story:** Epic 1.1 Card Definitions.
- **Client path:** SwiftUI + UniFFI (not Flutter).
- **Handoff:** Receiving agents get **`You are the receiving agent.`** as first line; brief is second person.

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
- **Handoff protocol:** First chat line **`You are the receiving agent.`** Body uses **You** …; never tell the receiver to write a handoff. Template: [`.cursor/handoff/_template.md`](.cursor/handoff/_template.md).

## Git workflow

1. Integration branch: **`main`**
2. Feature branches: `feature/<story-id>-short-topic` — merge only after **CMPH**
3. Gate before push: `cargo test -p push_core`
4. **UCPH** pushes feature branch only; **CMPH** merges to `main`

## Handoff protocol

UCPH/CMPH → Receiver brief for the **next** agent (see template). SWAT → handoff-checklist + review swarm.

## Epic close (SWAT)

Follow [.cursor/rules/epic-close.mdc](.cursor/rules/epic-close.mdc) only on **SWAT** or the epic’s last CMPH.
