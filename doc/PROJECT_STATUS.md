# Project status — PushSimulator

**Last updated:** 2026-10-06

---

## Summary

Push card-game simulator: **Rust workspace** + **agentic foundation** + **WASM debug table shell**. Public repo: **https://github.com/pbuckles22/PushSimulator**. Card definitions and the 108-card deck are parked on `feature/1.1-card-definitions` (not merged to `main`).

---

## Active branch

| Branch | Role |
|--------|------|
| **`feature/1.1-card-definitions`** | 1.1.1 Card Definitions and 1.1.3 Deck Generation — **not merged** to `main` |
| **`feature/foundation-scaffold`** | Foundation scaffold — pushed, **not merged** to `main` |
| **`main`** | Empty / not yet populated on remote |

---

## Completed

- AgenticTemplate layer + handoff receiver framing (`You are the receiving agent.`)
- Google docs organized under `doc/` (raw dump in `docs/`)
- Cargo workspace: `push_core`, `push_wasm`, `push_sim` — Tier 1 green (`cargo test -p push_core`)
- `viewer/` HTML/CSS/JS shell
- GitHub repo created (public)
- **1.1.1** Suit, Rank, Card (`id`, `suit`, `rank`, `locked_until_turn`)
- **1.1.3** `Deck::new`: 2 decks × 52 cards + 2 jokers per deck (4 jokers, 8 twos, 108 cards)

---

## Next

| Item | Detail |
|------|--------|
| **1.1.2** | Wild card identification — `test_card_is_wild_true_for_joker` first |
| **Merge** | **CMPH** only when human wants `main` populated |

---

## Reading order

1. [CONTRIBUTING.md](../CONTRIBUTING.md)
2. This file · [PLAN_COMMENTARY.md](PLAN_COMMENTARY.md) · [ARCHITECTURE.md](ARCHITECTURE.md)
3. [AGENT_HANDOFF.md](../AGENT_HANDOFF.md)
