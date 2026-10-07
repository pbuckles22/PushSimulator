# Agent handoff — PushSimulator

## Purpose

**Push** card-game engine and simulator: pure Rust core, WASM debug table (Phase 1b), Monte Carlo (Phase 2), SwiftUI + UniFFI iOS (Phase 3), Axum multiplayer (Phase 4). Agentic layer from [AgenticTemplate](https://github.com/pbuckles22/AgenticTemplate).

**Repo:** https://github.com/pbuckles22/PushSimulator

---

## Source of truth

- **Status:** [doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md)
- **Architecture:** [doc/ARCHITECTURE.md](doc/ARCHITECTURE.md)
- **Plan commentary:** [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md)
- **Backlog / Sprint 1:** [doc/BACKLOG.md](doc/BACKLOG.md), [doc/sprints/SPRINT_1.md](doc/sprints/SPRINT_1.md)
- **Rules:** [doc/requirements/GAME_RULES.md](doc/requirements/GAME_RULES.md)
- **Scope / phases:** [PM_PLAN.md](PM_PLAN.md)
- **Skills:** [.cursor/skills/](.cursor/skills/)

## Current state

- **Ship:** 1.7.2 is parked on **`feature/1.7.2-penalty-draw`**. Not on `main`. A failed discard of a card that fits sets `TurnPhase::PenaltyDrawing`. `Action::DrawFromDeck` keeps cards that fit, including a wild, and discards the first safe card. A locked card is discarded. An empty pile stays in that phase. The last two safe cards split. `advance_turn` returns to playing. The round does not end. `points` and `total_score` stay as they were. 1.7.1 is on **`main` at `c61f3aa`**. `Action::DiscardCard` places one card from the hand onto the discard pile. An off-board player cannot discard a card that could join a meld. The card just taken, or drawn on a push, can. `advance_turn` clears that exemption. A locked card that cannot be played is safe to discard until the counter catches up. A player on the board can discard a card that fits. The round does not end. `feature/1.7.1-safe-discard` is kept. Epic 1.6 is complete — 2026-10-07 — on **`main`**. The close commit is **`e5de020`**. The lock content is **`2abbf4e`**. `feature/1.6.3-wild-lock` is kept. A card whose `locked_until_turn` is still ahead of the turn counter cannot be played, hit, or used to replace a wild. `GameState::advance_turn` moves the counter up by one. The round does not end. 1.6.2 is on **`main` at `b5b2ac5`**. A natural card replaces a wild on a meld. The wild moves into the hand and `locked_until_turn` becomes the turn counter plus one. The counter stays put. A player who is off the board is refused. The round does not end. `feature/1.6.2-steal-wild` is kept. Epic 1.5 is complete — 2026-10-07. A player gets on the board by meeting that round's minimum in one play. Round 1 is two sets of 3. Round 2 is a set of 3 and a run of 4. Round 3 is two runs of 4. Round 4 is three sets of 3. Round 5 is a set of 3 and a run of 7. A meld may be larger than the minimum. An extra meld fails. `Action::PlayMeld` moves those melds onto the board and sets `is_on_board`. The close is on `main`. Content is `e0e1204`. `feature/1.5-swat` stays. 1.6.1 is on **`main` at `6c4a13e`**. `Action::HitMeld` adds cards from that player's hand onto one or more melds already on `GameState.board`. The player has to be on the board. Each meld, after those cards are added, has to be a set or a run. A 3 can join a set of threes and a 4 of spades can join a spade run in the same action. A card that does not fit refuses the whole action. A player who is not on the board is refused. The hand can be empty. The round does not end. `points` and `total_score` stay as they were. `feature/1.6.1-hit-meld` is kept. Handoff wording is on **`main` at `acc1865`**. The opening line is `You are the agent to execute the below hand off.` “Check in and merge” is CMPH, and on the epic’s last story it closes the epic. Files inside the handoff folders stay local. `README.md` and `_template.md` stay tracked. `node_modules/` is ignored. `feature/cursor-rules-checkin` is kept. 1.5.2 test locks are on **`main` at `b06eb23`**. `PlayMeld` already moved the cards. Tests lock wild sets leaving the hand, a set of 5 beside a set of 4, seat 1 opening the board, seat 2 at a three-seat table, and four wilds filling the run in round 2. The shoe chains take a discard into the meld, let the pushed seat lay down the penalty card, and use the round on the table. Every handoff includes `write thorough tests. red, integration, find missing ones of both kinds.` `feature/1.5.2-board-entry-test-locks` is kept. 1.5.2 is on **`main` at `572863a`**. `Action::PlayMeld` moves verified melds from that player's hand onto `GameState.board` and sets `is_on_board`. The round on the table is the one that is checked. A refused play leaves the hand, the board, the piles, and both scores unchanged. `feature/1.5.2-board-entry` is kept. 1.5.1 test locks are on **`main` at `5628cf8`**. The checker already enforced them. Tests lock one meld of six fours, a pair, an empty meld, a repeated id, four wilds as one requirement, ace-low and ace-high runs, and those checks on cards from the dealt shoe. `feature/1.5.1-requirement-holes` is kept. 1.5.1 is on **`main` at `e86e8d7`**. `check_round_requirements` enforces the five round minimums. A meld counts when it is a set or a run. One meld fills one requirement. Sizes may exceed the minimum. An extra meld fails. A card used twice fails. Six fours may be two sets of three. `feature/1.5.1-round-requirements` is kept. 1.2.4.2 is on **`main` at `1209802`**. `Deck::shuffle_with` and `Deck::draw_with` take the rng. Card ids survive shuffle, draw, and a reshuffle of three or more. Epic 1.2 is complete. `feature/1.2.4.2-tech-debt` is kept. 1.4.2 is on **`main` at `da56999`**. `validate_run` accepts four or more cards of one suit in order. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run. `feature/1.4.2-validate-runs` is kept. 1.2.4.3 is on **`main` at `b70cfa2`**. A short draw pile deals the cards it has, then panics. `feature/1.2.4.3-deal-short-deck` is kept. 1.4.1 Validating Sets is on **`main` at `62220b3`**. `validate_set` accepts three or more cards of one rank. Twos and jokers fill that rank. A group of only wilds is a set. 1.3.2 is on **`main` at `1500b02`**. `Action::PushDiscard` gives the next player the top discard and the top draw-pile card. The pushing player then draws. Epic 1.3 is complete. `feature/1.4.1-validate-sets` and `feature/1.3.2-push-discard` are kept. `main` before these lands was 1.3.1 at `1dfdbc6` (TakeDiscard content `6a00eff`). `feature/1.3.1-take-discard` is kept. Earlier Epic 1.2 history stays at `b7e4b93` (tip `2272506`). `feature/1.2.6-hand-total` is kept. Earlier 1.2 branches stay: `feature/1.2.5-wild-scoring`, `feature/1.2.4-face-ace-scoring`, `feature/1.2.3-pip-scoring`, `feature/1.2.2-deal-hands`, `feature/1.2.1-player-init`. Epic 1.1 stays inside that history. `feature/1.1.4-empty-reshuffle` is kept. `feature/1.1.4-shuffle-draw` is kept. Prior park: `origin/feature/1.1.2-wild-cards @ 71c50c7`.
- **Phase:** 1 — Rust core. **Next:** 1.7.3. Epic 1.7 stays open. 1.7.2 is parked on `feature/1.7.2-penalty-draw`, not on `main`. A failed discard of a card that fits sets `TurnPhase::PenaltyDrawing`. `Action::DrawFromDeck` keeps cards that fit and discards the first safe card. `advance_turn` returns to playing. The round does not end. 1.7.1 is on `main` at `c61f3aa`. `Action::DiscardCard` places one card from the hand onto the discard pile. An off-board player cannot discard a card that could join a meld. The card just taken, or drawn on a push, can. `advance_turn` clears that exemption. A player on the board can discard a card that fits. The round does not end. `feature/1.7.1-safe-discard` is kept. Epic 1.6 is complete — 2026-10-07, on `main`. The close commit is `e5de020`. The lock content is `2abbf4e`. `feature/1.6.3-wild-lock` is kept. A player on the board can hit a meld and steal a wild. The stolen wild cannot be played until the counter catches up. 1.6.2 is on `main` at `b5b2ac5`. `Action::StealWild` swaps a natural card for a wild on a meld. The wild moves into the hand and `locked_until_turn` becomes `turn_counter` plus one. The counter stays put. A player who is off the board is refused. An 8♦ does not replace the joker in 4♦, joker, 6♦, 7♦. The round does not end. `points` and `total_score` stay as they were. `feature/1.6.2-steal-wild` is kept. Epic 1.5 is complete. 1.6.1 `Action::HitMeld` adds cards from the hand onto one or more melds already on the board. The player has to be on the board. A card that does not fit refuses the whole action. The hand can be empty. The round does not end. `feature/1.6.1-hit-meld` is kept. Handoff wording is on `main` at `acc1865`. 1.5.2 test locks are on `main` at `b06eb23`. 1.5.2 `Action::PlayMeld` moves verified melds onto `GameState.board` and sets `is_on_board`. 1.5.1 test locks cover one meld of six, wild groups, and ace runs. 1.5.1 `check_round_requirements` enforces the five round minimums. Six fours may be two sets of three. Epic 1.4 is complete — 2026-10-06. The close is on `main`. Content is `2341904`. `feature/1.4-swat` stays. 1.4.2 `validate_run` accepts four or more cards of one suit in order. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run. 1.2.4.3 is on `main` at `b70cfa2`: a short draw pile deals the cards it has, then panics. 1.2.4.2 seeds `Deck::shuffle_with` and `Deck::draw_with`. Card ids survive shuffle, draw, and a reshuffle of three or more. Epic 1.2 is complete. 1.4.1 `validate_set` accepts three or more cards of one rank. Twos and jokers fill that rank. A group of only wilds is a set. Two cards, or two different natural ranks, are not. 1.3.2 `Action::PushDiscard` moves the discard top onto the next player's hand, then that player draws the top of the draw pile. The pushing player draws the next card. Cards under the pushed top stay. `points` and `total_score` stay 0. 1.3.1 `Action::TakeDiscard` moves the discard pile's top card onto the end of the player's hand. Cards under that top stay. The draw pile stays. `points` and `total_score` stay 0. 1.2.6 `calculate_hand_penalty` sums the hand. A hand of 4, Jack, Ace, and Joker is 50. `add_hand_penalty_to_total` adds that onto `total_score`. `points` stays 0. 1.2.5 returns 20 for a two and a joker. 1.2.4.1 returns 10 for a 10 through King and 15 for an ace. 1.2.3 returns 5 for ranks 3 through 9. 1.2.2 deals one card at a time to 2 or more players, 10 cards each. A one-player deal is refused. The discard pile stays put. 1.2.1 `Player::new` starts at 0 points, `is_on_board = false`, empty hand, `total_score` 0.
- **Version:** no bump (workspace 0.1.0; not a user-visible build).
- **Client path:** SwiftUI + UniFFI (not Flutter).
- **Handoff:** The next session gets **`You are the agent to execute the below hand off.`** as first line; brief is second person.

## Run and test

Requires Rust (`rustup`). Add `%USERPROFILE%\.cargo\bin` to PATH on Windows if `cargo` is missing.

```bash
cargo test -p push_core
cargo test
wasm-pack build push_wasm --target web --out-dir ../viewer/pkg
cargo run -p push_sim
```

Viewer: `cd viewer && npx --yes serve .`

**Merge-ready gate:** `cargo test -p push_core`

## Conventions

- Game rules live only in `push_core`.
- Strict TDD for Epic 1.x; integration chains after Epic 1.8.
- **Handoff protocol:** First chat line **`You are the agent to execute the below hand off.`** Body uses **You** …; never tell the receiver to write a handoff. Every brief includes this line verbatim, between **Next steps** and **Measured**: `write thorough tests. red, integration, find missing ones of both kinds.` Template: [`.cursor/handoff/_template.md`](.cursor/handoff/_template.md).

## Git workflow

1. Integration branch: **`main`**
2. Feature branches: `feature/<story-id>-short-topic` — merge only after **CMPH**
3. Gate before push: `cargo test -p push_core`
4. **UCPH** pushes feature branch only; **CMPH** merges to `main`

## Handoff protocol

UCPH/CMPH → Receiver brief for the **next** agent (see template). SWAT → handoff-checklist + review swarm.

## Epic close (SWAT)

Follow [.cursor/rules/epic-close.mdc](.cursor/rules/epic-close.mdc) only on **SWAT** or the epic’s last CMPH.
