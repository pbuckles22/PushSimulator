# PushSimulator

Rust-centric engine and simulator for the card game **Push**. Agentic Cursor foundation from AgenticTemplate. Epics 1.1 through 1.5 are complete: a deck, a deal, scoring, take and push, checks for a set and a run, and getting on the board. A player can then add cards from their hand onto melds already there. Next is 1.6.2, stealing a wild.

**Repository:** https://github.com/pbuckles22/PushSimulator

## New here?

**Start with [CONTRIBUTING.md](CONTRIBUTING.md)** and **[doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md)**.

Architecture and commentary: [doc/ARCHITECTURE.md](doc/ARCHITECTURE.md), [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md).

## Stack

| Layer | Tech |
|-------|------|
| Core | `push_core` (Rust) |
| Debug viewer | `push_wasm` + `viewer/` (WASM / HTML) |
| Monte Carlo | `push_sim` (Rayon) — Phase 2 |
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
