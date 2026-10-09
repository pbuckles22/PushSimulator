# PushSimulator

Rust-centric engine and simulator for the card game **Push**. Agentic Cursor foundation from AgenticTemplate. Phase 2 is complete: headless matches, Rayon batches, and strategic seats (point-averse, hoarder, keep-N) for 2 to 10 players. Epics 1.1 through 1.8 are complete: a deck, a deal, scoring, take and push, checks for a set and a run, and getting on the board. A player on the board can add cards onto melds already there, and can swap a natural card for a wild. That wild cannot be played until the turn counter catches up. An off-board player cannot discard a card that fits a meld. The card just taken or drawn can. An empty hand ends the round. The next round scores the hands that remain and deals again. A joker drawn during a penalty stays in the hand.

**Repository:** https://github.com/pbuckles22/PushSimulator

## New here?

**Start with [CONTRIBUTING.md](CONTRIBUTING.md)** and **[doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md)**.

Architecture and commentary: [doc/ARCHITECTURE.md](doc/ARCHITECTURE.md), [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md).

## Stack

| Layer | Tech |
|-------|------|
| Core | `push_core` (Rust) |
| Debug viewer | `push_wasm` + `viewer/` (WASM / HTML) |
| Monte Carlo | `push_sim` (Rayon) — Phase 2 complete |
| iOS | SwiftUI + UniFFI — Phase 3 |
| Multiplayer | Axum — Phase 4 |

## Quick start

```bash
# Needs rustup / cargo on PATH
cargo test -p push_core
cargo run -p push_sim
```

Viewer (after Phase 1b WASM build):

```bash
wasm-pack build push_wasm --target web --out-dir ../viewer/pkg
cd viewer && npx --yes serve .
```

## Docs layout

- **`doc/`** — organized project truth (requirements, backlog, sprints, commentary)
- **`docs/`** — original Google export (preserved; see `docs/README.md`)
