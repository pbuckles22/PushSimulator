# Project status — PushSimulator

**Last updated:** 2026-10-06

---

## Summary

Push card-game simulator: **Rust workspace** + **agentic foundation** + **WASM debug table shell**. Public repo: **https://github.com/pbuckles22/PushSimulator**. Epic 1.1 is complete on `main` at `248075f`: cards, wilds, a 108-card deck, shuffle, draw, and the empty-deck leftovers.

---

## Active branch

| Branch | Role |
|--------|------|
| **`main`** | Epic 1.1 complete — cards, wilds, 108-card deck, shuffle, draw, empty-deck leftovers |
| **`feature/1.2.4-face-ace-scoring`** | Parked. 10 through King score 10. An ace scores 15. Not merged |
| **`feature/1.2.3-pip-scoring`** | Parked. Ranks 3–9 score 5 points. Not merged |
| **`feature/1.2.2-deal-hands`** | Parked. Deal 10 cards to each of 2 or more players, one card at a time. Not merged |
| **`feature/1.2.1-player-init`** | Parked. `Player::new` starts empty. Not merged |
| **`feature/1.1.4-empty-reshuffle`** | Kept. Empty-deck leftovers |
| **`feature/1.1.4-shuffle-draw`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.1.2-wild-cards`** | Ancestor of `main`. Do not merge separately |
| **`feature/1.1-card-definitions`** | Ancestor of `main`. Do not merge separately |
| **`feature/foundation-scaffold`** | Ancestor of `main`. Do not merge separately |

---

## Completed

- AgenticTemplate layer + handoff framing (`You are the agent to execute the below hand off.`)
- Google docs organized under `doc/` (raw dump in `docs/`)
- Cargo workspace: `push_core`, `push_wasm`, `push_sim` — Tier 1 green (`cargo test -p push_core`)
- `viewer/` HTML/CSS/JS shell
- GitHub repo created (public)
- **1.1.1** Suit, Rank, Card (`id`, `suit`, `rank`, `locked_until_turn`)
- **1.1.3** `Deck::new`: 2 decks × 52 cards + 2 jokers per deck (4 jokers, 8 twos, 108 cards)
- **1.1.2** `Card::is_wild`: jokers and twos are wild; other ranks are not. A new deck has 12 wilds.
- **1.1.4** `Deck::shuffle` and `Deck::draw`. Order changes, one draw leaves 107 cards. Three or more discard cards leave the top card and reshuffle. One leftover card goes to the current player. Two leftover cards are shuffled and split. Both piles empty draws nothing.

---

## Next

| Item | Detail |
|------|--------|
| **1.2.5** | Wild card scoring — a two and a joker score 20, only when asked |
| **1.2.4.2** | Tech debt from the 2026-10-06 test review — not the next red/green |
| **1.2.4.3** | Test gaps from that review — not the next red/green |

---

## Reading order

1. [CONTRIBUTING.md](../CONTRIBUTING.md)
2. This file · [PLAN_COMMENTARY.md](PLAN_COMMENTARY.md) · [ARCHITECTURE.md](ARCHITECTURE.md)
3. [AGENT_HANDOFF.md](../AGENT_HANDOFF.md)
