# Project status — PushSimulator

**Last updated:** 2026-10-06

---

## Summary

Push card-game simulator: **Rust workspace** + **agentic foundation** + **WASM debug table shell**. Public repo: **https://github.com/pbuckles22/PushSimulator**. 1.3.2 is parked on `feature/1.3.2-push-discard` (not merged): the next player receives the top discard and the top draw-pile card, then the pushing player draws. 1.3.1 is on `main` at `1dfdbc6` (`6a00eff`): the top discard moves into the player's hand. Epic 1.2 stays inside that history at `b7e4b93`: a player, a 10-card deal, card penalties, and a hand total. Epic 1.1 (cards, wilds, a 108-card deck, shuffle, draw, and the empty-deck leftovers) is inside that history.

---

## Active branch

| Branch | Role |
|--------|------|
| **`feature/1.3.2-push-discard`** | Parked. Next player gets the top discard and the top draw card. Not merged |
| **`main`** | 1.3.1 landed — top discard moves into the player's hand |
| **`feature/1.3.1-take-discard`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.2.6-hand-total`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.2.5-wild-scoring`** | Kept. A two and a joker score 20 |
| **`feature/1.2.4-face-ace-scoring`** | Kept. 10 through King score 10. An ace scores 15 |
| **`feature/1.2.3-pip-scoring`** | Kept. Ranks 3–9 score 5 points |
| **`feature/1.2.2-deal-hands`** | Kept. Deal 10 cards to each of 2 or more players, one card at a time |
| **`feature/1.2.1-player-init`** | Kept. `Player::new` starts empty |
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
- **1.2.1** `Player::new`: 0 points, `total_score` 0, not on the board, empty hand
- **1.2.2** `deal_initial_hands`: 10 cards, one at a time, to 2 or more players. One player is refused. The discard pile stays put
- **1.2.3** Ranks 3–9 score 5
- **1.2.4.1** A 10 through King scores 10. An ace scores 15
- **1.2.5** A two and a joker score 20
- **1.2.6** `calculate_hand_penalty` sums the hand. A hand of 4, Jack, Ace, and Joker is 50. `add_hand_penalty_to_total` adds that onto `total_score`. `points` stays 0
- **1.3.1** `Action::TakeDiscard` moves the discard pile's top card onto the end of the player's hand. Cards under that top stay. The draw pile stays. `points` and `total_score` stay 0
- **1.3.2** `Action::PushDiscard` gives the next player the top discard and the top draw-pile card. The pushing player then draws. Cards under the pushed top stay. `points` and `total_score` stay 0

---

## Next

| Item | Detail |
|------|--------|
| **1.4.1** | Validating Sets — only when asked |
| **1.2.4.2** | Tech debt from the 2026-10-06 test review — not the next red/green |
| **1.2.4.3** | Test gaps from that review — not the next red/green |

---

## Reading order

1. [CONTRIBUTING.md](../CONTRIBUTING.md)
2. This file · [PLAN_COMMENTARY.md](PLAN_COMMENTARY.md) · [ARCHITECTURE.md](ARCHITECTURE.md)
3. [AGENT_HANDOFF.md](../AGENT_HANDOFF.md)
