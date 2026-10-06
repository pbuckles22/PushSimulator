# Plan commentary — PushSimulator

Commentary on Google's docs and how this repo layers agentic discipline on top. **Does not replace** [ARCHITECTURE.md](ARCHITECTURE.md), [BACKLOG.md](BACKLOG.md), or [sprints/SPRINT_1.md](sprints/SPRINT_1.md).

## How Google's docs landed

The `docs/` export had **mislabeled duplicates**:

| Filename claimed | Actual content |
|------------------|----------------|
| Architecture Blueprint | Architecture (good) |
| Push Zero-Defect Project Plan | Same Architecture file (duplicate) |
| Master Backlog | Full backlog (good) |
| Instructions for Next Steps | Same Master Backlog (duplicate) |
| Official Rulebook | Rulebook (good) |
| Data Structures & State Machine | Rulebook again (missing schema) |
| Monte Carlo Strategy Analysis | Rulebook again (missing strategy) |
| Sprint 1 Backlog | Sprint 1 (good) |

Organized copies live under `doc/`. Placeholders: [requirements/DATA_SCHEMA.md](requirements/DATA_SCHEMA.md), [requirements/MONTE_CARLO_STRATEGY.md](requirements/MONTE_CARLO_STRATEGY.md). Mislabeled originals archived under `doc/archive/`. Raw Google dump kept in `docs/` (do not overwrite).

## Architecture verdict

Google's plan is sound for this product:

1. **Pure Rust core** — single rules engine for sim, WASM, iOS, and server.
2. **Phase 1b WASM debug table** — watch bots early without Flutter/desktop cost.
3. **Rayon Monte Carlo** — Phase 2 on the same crate. Code stays on the Windows PC. The Mac is the first CPU host (`cargo build -p push_sim --release` on the Mac). Linux EC2 is the later scale host. Comparison runs are 100,000 games per matchup ([requirements/MONTE_CARLO_STRATEGY.md](requirements/MONTE_CARLO_STRATEGY.md)).
4. **SwiftUI + UniFFI for iOS** — native UI; no Flutter rewrite of the rules.
5. **Axum + WebSockets** — Phase 4 multiplayer on the same core.

We are **not** changing that stack. AgenticTemplate (not the Flutter FFI template) is the foundation because the long-term client is SwiftUI, not Flutter.

## What we add (agentic layer)

| Addition | Why |
|----------|-----|
| UCPH / CMPH / CMPHD / SWAT | Explicit ship commands; no silent merges to `main` |
| Integration chain tests | Unit red/green does not prove 1..X still work together ([.cursor/rules/integration-chain.mdc](../.cursor/rules/integration-chain.mdc)) |
| Plan-coordinate versioning | `MAJOR.EPIC.COUNTER` — never backwards |
| Gemini packs + dropzone | Bounded context for Gemini; inbound separate from conceptual pack |
| Feature branch archaeology | CMPH keeps feature branches as working-slice history |
| Human check | Plain-language smoke ask after install/run |
| Debug table + TDD panel | WASM viewer watches play; optional `test_status.json` for red/green |

## Suggested backlog delta (do not edit BACKLOG in place yet)

After Epic 1.8 (System Integration / The Crucible), add:

**Epic 1.9: Integration chain validation**

- Name each chain (e.g. deal → push → meld → hit → steal → discard → round end).
- One integration test per chain through real `push_core` APIs (not mocks of the engine).
- Fail if any prior step in 1..X breaks.

Phase 1b (WASM viewer) can start once Epic 1.2+ is playable headless; full bot-vs-bot on felt needs legal moves (Phase 2.1) or a scripted action feed.

## Viewer intent

- **Primary:** green felt table, CSS cards, scrolling action log (Google Phase 1b).
- **Secondary:** side panel for green/red counts and integration-chain status when a watch script writes `viewer/test_status.json`.

## Next concrete step

Start **Sprint 1 / Epic 1.1** in `push_core`: failing tests for Suit, Rank, Card, then green. See [sprints/SPRINT_1.md](sprints/SPRINT_1.md).
