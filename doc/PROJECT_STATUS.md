# Project status — PushSimulator

**Last updated:** 2026-10-07

---

## Summary

Push card-game simulator: **Rust workspace** + **agentic foundation** + **WASM debug table shell**. Public repo: **https://github.com/pbuckles22/PushSimulator**. 1.5.2 is on `main` at `572863a`: a valid meld leaves that player's hand, lands on `GameState.board`, and sets `is_on_board`. A refused play leaves the table unchanged. `feature/1.5.2-board-entry` is kept. 1.5.1 test locks are on `main` at `5628cf8`: one meld of six fours fails, four wilds fill one requirement, and an ace-low or ace-high run counts. `feature/1.5.1-requirement-holes` is kept. 1.5.1 is on `main` at `e86e8d7`: round minimums are checked before a player gets on the board. Six fours may be two sets of three. Epic 1.4 is complete — 2026-10-06: a set is three or more cards of one rank, and a run is four or more of one suit in order. The close is on `main`. Content is `2341904`. `feature/1.4-swat` stays. Epic 1.2 is complete. 1.2.4.2 is on `main` at `1209802`: shuffle and draw take an rng, and card ids survive a reshuffle of three or more. 1.4.2 is on `main` at `da56999`: four or more cards of one suit in order are a run, and an ace is low or high. 1.2.4.3 is on `main` at `b70cfa2`: a short draw pile deals the cards it has, then panics. 1.4.1 is on `main` at `62220b3`: a set is three or more cards of one rank, and twos and jokers can fill that rank. 1.3.2 is on `main` at `1500b02`: the next player receives the top discard and the top draw-pile card, then the pushing player draws. Epic 1.3 is complete. 1.3.1 stays in that history at `1dfdbc6` (`6a00eff`): the top discard moves into the player's hand. Epic 1.1 (cards, wilds, a 108-card deck, shuffle, draw, and the empty-deck leftovers) is inside that history.

---

## Active branch

| Branch | Role |
|--------|------|
| **`main`** | 1.5.2 landed — a valid meld moves onto the board |
| **`feature/1.5.2-board-entry`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.5.1-requirement-holes`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.5.1-round-requirements`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.4-swat`** | Kept. Do not merge again |
| **`feature/1.2.4.2-tech-debt`** | Kept. Seeded shuffle and draw. Epic 1.2 closes with this land |
| **`feature/1.4.2-validate-runs`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.2.4.3-deal-short-deck`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.4.1-validate-sets`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.3.2-push-discard`** | Kept. Same history as this land. Do not merge again |
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
- **1.5.2** `Action::PlayMeld` moves verified melds from that player's hand onto `GameState.board` and sets `is_on_board`. The round on the table is the one that is checked. A card that is not in that hand fails. A refused play leaves the hand, the board, the piles, and both scores unchanged
- **1.5.1 tests** lock one meld of six fours, a pair, an empty meld, a repeated id, four wilds as one requirement, ace-low and ace-high runs, and those checks on cards from the dealt shoe
- **1.5.1** `check_round_requirements` enforces the five round minimums. A meld counts when it is a set or a run. Sizes may exceed the minimum. An extra meld fails. A card used twice fails. Six fours may be two sets of three
- **Epic 1.4** complete — 2026-10-06. A set is three or more cards of one rank. A run is four or more cards of one suit in order. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run
- **1.4.2** `validate_run` accepts four or more cards of one suit in order. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run
- **1.4.1** `validate_set` accepts three or more cards of one rank. Twos and jokers fill that rank. A group of only wilds is a set. Two cards, or two different natural ranks, are not

---

## Next

| Item | Detail |
|------|--------|
| **1.6.1** | Waits until asked. 1.5.2 is on `main` at `572863a`. Epic 1.5 stays open. |

---

## Reading order

1. [CONTRIBUTING.md](../CONTRIBUTING.md)
2. This file · [PLAN_COMMENTARY.md](PLAN_COMMENTARY.md) · [ARCHITECTURE.md](ARCHITECTURE.md)
3. [AGENT_HANDOFF.md](../AGENT_HANDOFF.md)
