# PM_PLAN — PushSimulator

Operating model: `.cursor/rules/` ship commands (UCPH, CMPH, SWAT), human check, plan-coordinate versioning. Do not delete them.

## Phases

| Phase | Focus | Primary crate / surface |
|-------|--------|-------------------------|
| **1** | Rust core engine (TDD) | `push_core` |
| **1b** | WASM debug table | `push_wasm` + `viewer/` |
| **2** | Monte Carlo + bots | `push_sim` |
| **3** | iOS local play | UniFFI + SwiftUI |
| **4** | Multiplayer | Axum + WebSockets |

## Current

- **Phase 1 / Sprint 1:** Epic 1.1, Epic 1.2, Epic 1.3, Epic 1.4, and Epic 1.5 are complete — [doc/sprints/SPRINT_1.md](doc/sprints/SPRINT_1.md)
- **Epic 1.1:** complete — 2026-10-06. Empty-deck rule: [doc/requirements/GAME_RULES.md](doc/requirements/GAME_RULES.md).
- **Epic 1.2:** complete — 2026-10-06. A player starts empty. Two or more players are dealt 10 cards, one at a time. Ranks 3–9 score 5, a 10 through King scores 10, an ace scores 15, and a two or a joker scores 20. A hand of 4, Jack, Ace, and Joker totals 50 and that penalty adds onto `total_score`. `points` stays 0. **1.2.4.3** is closed. **1.2.4.2** seeds shuffle and draw, keeps card ids through a reshuffle, and closes the epic.
- **1.3.1:** on `main` at `6a00eff`. `Action::TakeDiscard` moves the discard pile's top card into the player's hand. Cards under that top stay. The draw pile stays. `points` and `total_score` stay 0. `feature/1.3.1-take-discard` is kept.
- **Epic 1.3:** complete — 2026-10-06. Take the top discard, or push it: the next player receives that card and the top of the draw pile, then the pushing player draws.
- **1.3.2:** on `main` at `1500b02`. `Action::PushDiscard` gives the next player two cards: the top discard and the top of the draw pile. The pushing player then draws. Cards under the pushed top stay. `points` and `total_score` stay 0. `feature/1.3.2-push-discard` is kept.
- **1.4.1:** on `main` at `62220b3`. `validate_set` accepts three or more cards of one rank. Twos and jokers fill that rank. A group of only wilds is a set. Two cards, or two different natural ranks, are not. `feature/1.4.1-validate-sets` is kept.
- **1.4.2:** on `main` at `da56999`. `validate_run` accepts four or more cards of one suit in order. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run. `feature/1.4.2-validate-runs` is kept.
- **Epic 1.4:** complete — 2026-10-06. A set is three or more cards of one rank. A run is four or more cards of one suit in order. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run. This close is on `main`. Content is `2341904`. `feature/1.4-swat` stays.
- **1.5.1:** on `main` at `e86e8d7`. `check_round_requirements` enforces the five round minimums. A meld counts when it is a set or a run. One meld fills one requirement. Sizes may exceed the minimum. An extra meld fails. A card used twice fails. Six fours may be two sets of three. `feature/1.5.1-round-requirements` is kept.
- **1.5.1 tests:** on `main` at `5628cf8`. The checker already enforced these. Tests lock one meld of six fours, a pair, an empty meld, a repeated id, four wilds as one requirement, ace-low and ace-high runs, and those checks on cards from the dealt shoe. `feature/1.5.1-requirement-holes` is kept.
- **1.5.2:** on `main` at `572863a`. `Action::PlayMeld` moves verified melds from that player's hand onto `GameState.board` and sets `is_on_board`. The round on the table is the one that is checked. A refused play leaves the hand, the board, the piles, and both scores unchanged. `feature/1.5.2-board-entry` is kept.
- **1.5.2 tests:** on `main` at `b06eb23`. `PlayMeld` already did this. Tests lock wild sets leaving the hand, a set of 5 beside a set of 4, seat 1 opening the board, seat 2 at a three-seat table, four wilds filling the run in round 2, a taken discard in the meld, the pushed seat laying down the penalty card, and the round on the table deciding the same shoe melds. `feature/1.5.2-board-entry-test-locks` is kept.
- **Epic 1.5:** complete — 2026-10-07. A player gets on the board by meeting that round's minimum in one play. Round 1 is two sets of 3. Round 2 is a set of 3 and a run of 4. Round 3 is two runs of 4. Round 4 is three sets of 3. Round 5 is a set of 3 and a run of 7. A meld may be larger than the minimum. An extra meld fails. `Action::PlayMeld` moves those melds onto the board and sets `is_on_board`. The close is on `main`. Content is `e0e1204`. `feature/1.5-swat` stays.
- **1.6.1:** on `main` at `6c4a13e`. `Action::HitMeld` adds cards from that player's hand onto one or more melds already on `GameState.board`. The player has to be on the board. Each meld, after those cards are added, has to be a set or a run. A 3 can join a set of threes and a 4 of spades can join a spade run in the same action. A card that does not fit refuses the whole action. A player who is not on the board is refused. The hand can be empty. The round does not end. `points` and `total_score` stay as they were. `feature/1.6.1-hit-meld` is kept.
- **1.6.2:** on `main` at `b5b2ac5`. `Action::StealWild` swaps a natural card for a wild on a meld. The wild moves onto the end of the hand and `locked_until_turn` becomes `turn_counter` plus one. The counter stays put. A player who is off the board is refused. A card that is not the one the wild stands for is refused. An 8♦ does not replace the joker in 4♦, joker, 6♦, 7♦. The round does not end. `points` and `total_score` stay as they were. `feature/1.6.2-steal-wild` is kept.
- **Next:** 1.6.3. Epic 1.6 stays open. That story is the last one in the epic, so its land closes Epic 1.6. Epic 1.5 is complete.
- **Full backlog:** [doc/BACKLOG.md](doc/BACKLOG.md)
- **Commentary / deltas:** [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md) (integration-chain Epic 1.9 suggested; Google export quirks)

## Version

Plan coordinate starts at **0.1.0** after first product ship on `main`. See [doc/VERSIONING.md](doc/VERSIONING.md).

Keep in sync with AGENT_HANDOFF *Current state* and [doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md).
