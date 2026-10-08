# Project status — PushSimulator

**Last updated:** 2026-10-08

---

## Summary

Story 2.2.1 is in progress. Phases 0–3 plus Phase 4 stages 1–2 are on `main` at `90ab20b`. `has_draw_capacity` counts one-card draws and treats two leftover discards as none. `validate_action` accepts, rejects, or marks a move invalid without cloning the table. A 14-card walk allocates at most 16 times per emitted action, plus 64. Release ceilings cover one take, a 750-play walk, and a 15-hit walk. Seed 1 still finishes at 340 and 1025. The large-hand fallback stays. Next is Phase 4 Stage 3 in [REFACTOR_2_2_1.md](REFACTOR_2_2_1.md): five batches of 1,000 games, then remove that fallback.

2.2 is on `main` at `d815423`. Two random seats finish five rounds. Seed 1 leaves round 6 dealt and unplayed. `points` stay 0. Totals are 340 and 1025. Every card id stays on the table. `feature/2.2-random-bot` is kept. 2.1 is on `main` at `e0e6b42`. `generate_legal_moves` lists every action `apply` would accept for one seat. The table stays as it was. The 8♦ hits the eights. The 9♥ hits the heart run. The joker hits either. The 4♦ and the king are discards only. The 3♠ joins 5♠–8♠ only with the 4♠. Off the board, two sets of three can be laid down. The 8♦ that fits cannot be discarded. Six fours are two sets of three. A locked card can be discarded and cannot be hit. The 5♦ steals the joker. The card just taken can be discarded. `DrawFromDeck` is listed only for the penalty seat. A closed round lists nothing. `points` and `total_score` stay as they were. `feature/2.1-legal-moves` is kept. Epic 1.8 is complete — 2026-10-07 — on `main`. Content is `b2303b9`. A penalty draw keeps a joker and discards the king. The joker stays in the hand, with `locked_until_turn` still 0. Hitting it is refused. Discarding it returns that seat to penalty drawing. The other seat cannot draw. A later king is discarded and the joker remains. Its 20 stays in the hand. The card just taken stays the only quick discard. `points` and `total_score` stay as they were. The round does not end. `feature/1.8.5-trapped-by-a-draw` is kept. 1.8.4 is on `main` at `411f227`. It scores each remaining hand onto `total_score` and deals the next round. The empty hand adds 0. A hand of 4, Jack, Ace, and a locked joker adds 50. `points` stay as they were. The two on the board and the king on the discard are not scored. Locks are cleared. The board is empty. Nobody is on the board. The counter is 0. The phase is playing. Round 1 becomes round 2. Round 5 deals round 6. A call while the round is still open leaves the table. `feature/1.8.4-round-transition` is kept. 1.8.3 is on `main` at `f80c752`. It locks one turn that lays two sets, hits the set just laid, hits a run, steals a joker, and discards a card that fits. The 8♦ is refused before that lay-down and allowed after. A 3♠ fits 5♠–8♠ only after a 4♠. The stolen joker's 20 stays in the hand. `points` and `total_score` stay as they were. If the other seat is already in penalty drawing, this turn leaves that penalty, and only that seat can draw. `feature/1.8.3-omniturn` is kept. 1.8.2 is on `main` at `a73a3bd`. It keeps the discard penalty on the seat a push trapped. A push of a playable card sets `TurnPhase::PenaltyDrawing` and `penalty_seat` on that off-board seat. The pusher can still discard. That discard does not clear the penalty, and the pusher cannot `DrawFromDeck`. The trapped seat draws until a safe card. The joker's 20 stays in the hand. `points` and `total_score` stay as they were. `advance_turn` clears the penalty. `feature/1.8.2-pushed-penalty-trap` is kept. 1.8.1 is on `main` at `8f41d12`. It steals a joker, waits until the lock catches up, and goes out with that joker. A card left in the hand does not end the round. The other seat can go out first. The joker's 20 stays in that hand. `points` and `total_score` stay as they were. Once `round_over` is set, every action is refused. Discarding the stolen joker on the turn it was taken stays allowed. `feature/1.8.1-steal-hold-win` is kept. Epic 1.7 is complete — 2026-10-07 — on `main`. Content is `6b97524`. A play, a hit, or a discard that leaves the hand empty sets `round_over`. A card left in the hand does not. `round_number`, `points`, and `total_score` stay as they were. `feature/1.7.3-round-victory` is kept. 1.7.2 is on `main` at `8375560`. A failed discard of a card that fits sets `TurnPhase::PenaltyDrawing`. `Action::DrawFromDeck` keeps cards that fit, including a wild, and discards the first safe card. A locked card is discarded. An empty pile stays in that phase. `advance_turn` returns to playing. `feature/1.7.2-penalty-draw` is kept. 1.7.1 is on `main` at `c61f3aa`. `Action::DiscardCard` places one card from the hand onto the discard pile. An off-board player cannot discard a card that could join a meld. The card just taken, or drawn on a push, can. `advance_turn` clears that exemption. A player on the board can discard a card that fits. The round does not end. `feature/1.7.1-safe-discard` is kept. Epic 1.6 is complete — 2026-10-07 — on `main`. The close commit is `e5de020`. The lock content is `2abbf4e`. `feature/1.6.3-wild-lock` is kept. A card whose lock is still ahead of the turn counter cannot be played, hit, or used to replace a wild. `advance_turn` moves the counter up by one. The round does not end. 1.6.2 is on `main` at `b5b2ac5`. A natural card replaces a wild on a meld. The wild moves into the hand and locks until the turn counter plus one. The round does not end. `feature/1.6.2-steal-wild` is kept. Epic 1.5 is complete — 2026-10-07. A player gets on the board by meeting that round's minimum in one play. The close is on `main`. Content is `e0e1204`. `feature/1.5-swat` stays. 1.6.1 is on `main` at `6c4a13e`: `Action::HitMeld` adds cards from that player's hand onto one or more melds already on the board. The player has to be on the board. One action can hit several melds. A card that does not fit refuses the whole action. The hand can be empty. The round does not end. `feature/1.6.1-hit-meld` is kept. Push card-game simulator: **Rust workspace** + **agentic foundation** + **WASM debug table shell**. Public repo: **https://github.com/pbuckles22/PushSimulator**. Handoff wording is on `main` at `acc1865`: the opening line is `You are the agent to execute the below hand off.`, and “check in and merge” closes the epic on its last story. Handoff session files stay local. `feature/cursor-rules-checkin` is kept. 1.5.2 test locks are on `main` at `b06eb23`: wild sets, oversized sets, a later seat, and the shoe chains were already what `PlayMeld` did. `feature/1.5.2-board-entry-test-locks` is kept. 1.5.2 is on `main` at `572863a`: a valid meld leaves that player's hand, lands on `GameState.board`, and sets `is_on_board`. A refused play leaves the table unchanged. `feature/1.5.2-board-entry` is kept. 1.5.1 test locks are on `main` at `5628cf8`: one meld of six fours fails, four wilds fill one requirement, and an ace-low or ace-high run counts. `feature/1.5.1-requirement-holes` is kept. 1.5.1 is on `main` at `e86e8d7`: round minimums are checked before a player gets on the board. Six fours may be two sets of three. Epic 1.4 is complete — 2026-10-06: a set is three or more cards of one rank, and a run is four or more of one suit in order. The close is on `main`. Content is `2341904`. `feature/1.4-swat` stays. Epic 1.2 is complete. 1.2.4.2 is on `main` at `1209802`: shuffle and draw take an rng, and card ids survive a reshuffle of three or more. 1.4.2 is on `main` at `da56999`: four or more cards of one suit in order are a run, and an ace is low or high. 1.2.4.3 is on `main` at `b70cfa2`: a short draw pile deals the cards it has, then panics. 1.4.1 is on `main` at `62220b3`: a set is three or more cards of one rank, and twos and jokers can fill that rank. 1.3.2 is on `main` at `1500b02`: the next player receives the top discard and the top draw-pile card, then the pushing player draws. Epic 1.3 is complete. 1.3.1 stays in that history at `1dfdbc6` (`6a00eff`): the top discard moves into the player's hand. Epic 1.1 (cards, wilds, a 108-card deck, shuffle, draw, and the empty-deck leftovers) is inside that history.

---

## Active branch

| Branch | Role |
|--------|------|
| **`main`** | 2.2.1 stages 1–2 at `90ab20b`. Story stays open |
| **`feature/2.2.1-performance-plan`** | Kept. Stage 3 is next |
| **`feature/2.2-random-bot`** | Kept. Do not merge again |
| **`feature/2.1-legal-moves`** | Kept. Do not merge again |
| **`feature/1.8.5-trapped-by-a-draw`** | Kept. Epic 1.8 close. Do not merge again |
| **`feature/1.8.4-round-transition`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.8.3-omniturn`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.8.2-pushed-penalty-trap`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.8.1-steal-hold-win`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.7.3-round-victory`** | Kept. Epic 1.7 close. Do not merge again |
| **`feature/1.7.2-penalty-draw`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.7.1-safe-discard`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.6.3-wild-lock`** | Kept. Epic 1.6 close. Do not merge again |
| **`feature/1.5-swat`** | Kept. Epic 1.5 close. Do not merge again |
| **`feature/1.6.2-steal-wild`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.6.1-hit-meld`** | Kept. Same history as this land. Do not merge again |
| **`feature/cursor-rules-checkin`** | Kept. Same history as this land. Do not merge again |
| **`feature/1.5.2-board-entry-test-locks`** | Kept. Same history as this land. Do not merge again |
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

- **2.2** on `main` at `d815423`. Two random seats finish five rounds. Seed 1 leaves round 6 dealt and unplayed. `points` stay 0. Totals are 340 and 1025. Every card id stays on the table. `feature/2.2-random-bot` is kept
- **2.1** on `main` at `e0e6b42`. `generate_legal_moves` lists every action `apply` would accept for one seat. The table stays as it was. The 8♦ hits the eights. The 9♥ hits the heart run. The joker hits either. The 4♦ and the king are discards only. The 3♠ joins 5♠–8♠ only with the 4♠. Off the board, two sets of three can be laid down. The 8♦ that fits cannot be discarded. Six fours are two sets of three. A locked card can be discarded and cannot be hit. The 5♦ steals the joker. The card just taken can be discarded. `DrawFromDeck` is listed only for the penalty seat. A closed round lists nothing. `points` and `total_score` stay as they were. `feature/2.1-legal-moves` is kept
- **Epic 1.8** complete — 2026-10-07. A stolen joker can wait and win. A push can trap a seat into the penalty draw. One turn can lay down, hit, steal, and discard. The round scores the hands that remain and deals again. A joker drawn in that penalty stays in the hand. The close is on `main`. Content is `b2303b9`. `feature/1.8.5-trapped-by-a-draw` stays
- **1.8.5** on `main` at `b2303b9`. A penalty draw keeps a joker and discards the king. The joker stays in the hand, with `locked_until_turn` still 0. Hitting it is refused. Discarding it returns that seat to penalty drawing. The other seat cannot draw. A later king is discarded and the joker remains. Its 20 stays in the hand. The card just taken stays the only quick discard. `points` and `total_score` stay as they were. The round does not end
- **1.8.4** on `main` at `411f227`. It scores each remaining hand onto `total_score` and deals the next round. The empty hand adds 0. A hand of 4, Jack, Ace, and a locked joker adds 50. `points` stay as they were. The two on the board and the king on the discard are not scored. Locks are cleared. The board is empty. Nobody is on the board. The counter is 0. The phase is playing. Round 1 becomes round 2. Round 5 deals round 6. A call while the round is still open leaves the table
- **1.8.3** on `main` at `f80c752`. It locks one turn that lays two sets, hits the set just laid, hits a run, steals a joker, and discards a card that fits. The 8♦ is refused before that lay-down and allowed after. A 3♠ fits 5♠–8♠ only after a 4♠. The stolen joker's 20 stays in the hand. `points` and `total_score` stay as they were. If the other seat is already in penalty drawing, this turn leaves that penalty, and only that seat can draw
- **1.8.2** on `main` at `a73a3bd`. A push of a playable card sets `TurnPhase::PenaltyDrawing` and `penalty_seat` on that off-board seat. The pusher can still discard. That discard does not clear the penalty, and the pusher cannot `DrawFromDeck`. The trapped seat draws until a safe card. The joker's 20 stays in the hand. `points` and `total_score` stay as they were. `advance_turn` clears the penalty
- **1.8.1** on `main` at `8f41d12`. Steal a joker, wait until the lock catches up, then go out with that joker. A card left in the hand does not end the round. The other seat can go out first. The joker's 20 stays in that hand. `points` and `total_score` stay as they were. Once `round_over` is set, every action is refused. Discarding the stolen joker on the turn it was taken stays allowed
- **Epic 1.7** complete — 2026-10-07. An off-board player cannot discard a card that fits a meld. A failed discard draws until a safe card. An empty hand ends the round. The close is on `main`. Content is `6b97524`. `feature/1.7.3-round-victory` stays
- **1.7.3** A play, a hit, or a discard that leaves the hand empty sets `round_over`. A card left in the hand does not. `round_number`, `points`, and `total_score` stay as they were
- **1.7.2** on `main` at `8375560`. A failed discard of a card that fits sets `TurnPhase::PenaltyDrawing`. `Action::DrawFromDeck` keeps cards that fit and discards the first safe card. An empty pile stays in that phase. `advance_turn` returns to playing. The draw does not end the round. `points` and `total_score` stay as they were. `feature/1.7.2-penalty-draw` is kept
- **1.7.1** `Action::DiscardCard` places one card from the hand onto the discard pile. An off-board player cannot discard a card that could join a meld. The card just taken, or drawn on a push, can. `advance_turn` clears that exemption. A locked card that cannot be played is safe to discard until the counter catches up. A player on the board can discard a card that fits. The hand can be empty. The round does not end. `points` and `total_score` stay as they were
- **Epic 1.6** complete — 2026-10-07. A player on the board can hit a meld and steal a wild. The stolen wild cannot be played until the turn counter catches up. The close is on `main`. Content is `2abbf4e`. The close commit is `e5de020`. `feature/1.6.3-wild-lock` stays
- **1.6.3** A card whose `locked_until_turn` is ahead of the turn counter cannot be played, hit, or used to replace a wild. `GameState::advance_turn` moves the counter up by one. The round does not end. `points` and `total_score` stay as they were
- **Epic 1.5** complete — 2026-10-07. Round 1 is two sets of 3. Round 2 is a set of 3 and a run of 4. Round 3 is two runs of 4. Round 4 is three sets of 3. Round 5 is a set of 3 and a run of 7. A meld may be larger than the minimum. An extra meld fails. `Action::PlayMeld` moves those melds onto the board and sets `is_on_board`. The close is on `main`. Content is `e0e1204`. `feature/1.5-swat` stays
- **1.6.1** `Action::HitMeld` adds cards from that player's hand onto one or more melds already on `GameState.board`. The player has to be on the board. A 3 can join a set of threes and a 4 of spades can join a spade run in the same action. A card that does not fit refuses the whole action. The hand can be empty. The round does not end. `points` and `total_score` stay as they were
- Handoff wording on `main` at `acc1865`. “Check in and merge” is CMPH and closes the epic on its last story. Handoff session files stay local
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
- **1.5.2 tests** lock wild sets leaving the hand, a set of 5 beside a set of 4, seat 1 opening the board, seat 2 at a three-seat table, four wilds filling the run in round 2, and those plays on cards from the dealt shoe
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
| **2.2.1** | Execute [REFACTOR_2_2_1.md](REFACTOR_2_2_1.md) before parallel games and CSV metrics. |

---

## Reading order

1. [CONTRIBUTING.md](../CONTRIBUTING.md)
2. This file · [PLAN_COMMENTARY.md](PLAN_COMMENTARY.md) · [ARCHITECTURE.md](ARCHITECTURE.md)
3. [AGENT_HANDOFF.md](../AGENT_HANDOFF.md)
