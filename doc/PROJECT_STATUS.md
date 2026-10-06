# Project status — PushSimulator

**Last updated:** 2026-10-06

---

## Summary

Push card-game simulator: **Rust workspace** + **agentic foundation** + **WASM debug table shell**. Public repo: **https://github.com/pbuckles22/PushSimulator**. Epic 1.1’s sprint slice (cards, wilds, deck, shuffle, draw) is on `main`. The empty-deck reshuffle is still open and blocks 1.2.

---

## Active branch

| Branch | Role |
|--------|------|
| **`main`** | Epic 1.1 first land — cards, wilds, 108-card deck, shuffle, draw |
| **`feature/1.1.4-shuffle-draw`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.1.2-wild-cards`** | Ancestor of `main`. Do not merge separately |
| **`feature/1.1-card-definitions`** | Ancestor of `main`. Do not merge separately |
| **`feature/foundation-scaffold`** | Ancestor of `main`. Do not merge separately |

---

## Completed

- AgenticTemplate layer + handoff receiver framing (`You are the receiving agent.`)
- Google docs organized under `doc/` (raw dump in `docs/`)
- Cargo workspace: `push_core`, `push_wasm`, `push_sim` — Tier 1 green (`cargo test -p push_core`)
- `viewer/` HTML/CSS/JS shell
- GitHub repo created (public)
- **1.1.1** Suit, Rank, Card (`id`, `suit`, `rank`, `locked_until_turn`)
- **1.1.3** `Deck::new`: 2 decks × 52 cards + 2 jokers per deck (4 jokers, 8 twos, 108 cards)
- **1.1.2** `Card::is_wild`: jokers and twos are wild; other ranks are not. A new deck has 12 wilds.
- **1.1.4** `Deck::shuffle` and `Deck::draw`: order changes, one draw leaves 107 cards, an empty deck returns `None`.

---

## Next

| Item | Detail |
|------|--------|
| **1.1.4 reshuffle** | Empty draw recycles the discard pile (minus the top card), shuffles, and draws. Blocks 1.2 |
| **1.2.1** | Player initialization — do not start until the reshuffle lines are done |

---

## Reading order

1. [CONTRIBUTING.md](../CONTRIBUTING.md)
2. This file · [PLAN_COMMENTARY.md](PLAN_COMMENTARY.md) · [ARCHITECTURE.md](ARCHITECTURE.md)
3. [AGENT_HANDOFF.md](../AGENT_HANDOFF.md)
