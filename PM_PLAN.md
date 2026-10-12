# PM_PLAN — PushSimulator

Operating model: `.cursor/rules/` ship commands (UCPH, CMPH, SWAT), human check, plan-coordinate versioning. Do not delete them.

## Phases

| Phase | Focus | Primary crate / surface |
|-------|--------|-------------------------|
| **1** | Rust core engine (TDD) | `push_core` |
| **1b** | Watch a four-person game | `push_wasm` + `viewer/` |
| **2** | Monte Carlo + bots | `push_sim` |
| **3** | iOS local play | UniFFI + SwiftUI |
| **4** | Multiplayer | Axum + WebSockets |

## Current

- **Phase 1 / Sprint 1:** Epic 1.1 through Epic 1.8 are complete — [doc/sprints/SPRINT_1.md](doc/sprints/SPRINT_1.md)
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
- **1.6.3:** on `main` at `2abbf4e`. A card whose `locked_until_turn` is ahead of the turn counter cannot be played, hit, or used to replace a wild. `GameState::advance_turn` moves the counter up by one and leaves the rest of the table as it was. The round does not end. `points` and `total_score` stay as they were. `feature/1.6.3-wild-lock` is kept.
- **Epic 1.6:** complete — 2026-10-07. A player on the board can add cards onto melds already there, and can swap a natural card for a wild. That wild locks until the turn counter plus one and cannot be played until the counter catches up. The close is on `main`. Content is `2abbf4e`. The close commit is `e5de020`. `feature/1.6.3-wild-lock` stays.
- **1.7.1:** on `main` at `c61f3aa`. `Action::DiscardCard` places one card from the hand onto the discard pile. An off-board player cannot discard a card that could join a meld. The card just taken, or drawn on a push, can. `advance_turn` clears that exemption. A locked card that cannot be played is safe to discard until the counter catches up. A player on the board can discard a card that fits. The round does not end. `points` and `total_score` stay as they were. `feature/1.7.1-safe-discard` is kept.
- **1.7.2:** on `main` at `8375560`. A failed discard of a card that fits sets `TurnPhase::PenaltyDrawing`. `Action::DrawFromDeck` keeps cards that fit, including a wild, and discards the first safe card. A locked card is discarded. An empty pile stays in that phase. The last two safe cards split. `advance_turn` returns to playing. The round does not end. `points` and `total_score` stay as they were. `feature/1.7.2-penalty-draw` is kept.
- **1.7.3:** on `main` at `6b97524`. A play, a hit, or a discard that leaves the hand empty sets `round_over`. A card left in the hand does not. `round_number`, `points`, and `total_score` stay as they were. `feature/1.7.3-round-victory` is kept.
- **Epic 1.7:** complete — 2026-10-07. An off-board player cannot discard a card that fits a meld. The card just taken can. A failed discard draws until a safe card. An empty hand ends the round. The close is on `main`. Content is `6b97524`. `feature/1.7.3-round-victory` stays.
- **1.8.1:** on `main` at `8f41d12`. Steal a joker, wait until the lock catches up, then go out with that joker. Hitting it while the lock is ahead leaves the round open. A card left in the hand does not end the round. The other seat can go out first. The joker's 20 stays in that hand. `points` and `total_score` stay as they were. Once `round_over` is set, every action is refused. Discarding the stolen joker on the turn it was taken stays allowed. `feature/1.8.1-steal-hold-win` is kept.
- **1.8.2:** on `main` at `a73a3bd`. A push of a playable card sets `TurnPhase::PenaltyDrawing` and `penalty_seat` on that off-board seat. The pusher can still discard. That discard does not clear the penalty, and the pusher cannot `DrawFromDeck`. The trapped seat draws until a safe card. The joker's 20 stays in the hand. `points` and `total_score` stay as they were. `advance_turn` clears the penalty. `feature/1.8.2-pushed-penalty-trap` is kept.
- **1.8.3:** on `main` at `f80c752`. It locks one turn that lays two sets, hits the set just laid, hits a run, steals a joker, and discards a card that fits. The 8♦ is refused before that lay-down and allowed after. A 3♠ fits 5♠–8♠ only after a 4♠. The stolen joker's 20 stays in the hand. `points` and `total_score` stay as they were. If the other seat is already in penalty drawing, this turn leaves that penalty, and only that seat can draw. `feature/1.8.3-omniturn` is kept.
- **1.8.4:** on `main` at `411f227`. It scores each remaining hand onto `total_score` and deals the next round. The empty hand adds 0. A hand of 4, Jack, Ace, and a locked joker adds 50. `points` stay as they were. The two on the board and the king on the discard are not scored. Locks are cleared. The board is empty. Nobody is on the board. The counter is 0. The phase is playing. Round 1 becomes round 2. Round 5 deals round 6. A call while the round is still open leaves the table. `feature/1.8.4-round-transition` is kept.
- **1.8.5:** on `main` at `b2303b9`. A penalty draw keeps a joker and discards the king. The joker stays in the hand, with `locked_until_turn` still 0. Hitting it is refused. Discarding it returns that seat to penalty drawing. The other seat cannot draw. A later king is discarded and the joker remains. Its 20 stays in the hand. The card just taken stays the only quick discard. `points` and `total_score` stay as they were. The round does not end. `feature/1.8.5-trapped-by-a-draw` is kept.
- **Epic 1.8:** complete — 2026-10-07. A stolen joker can wait and win. A push can trap a seat into the penalty draw. One turn can lay down, hit, steal, and discard. The round scores the hands that remain and deals again. A joker drawn in that penalty stays in the hand. The close is on `main`. Content is `b2303b9`. `feature/1.8.5-trapped-by-a-draw` stays.
- **2.1:** on `main` at `e0e6b42`. `generate_legal_moves` lists every action `apply` would accept for one seat. The table stays as it was. The 8♦ hits the eights. The 9♥ hits the heart run. The joker hits either. The 4♦ and the king are discards only. The 3♠ joins 5♠–8♠ only with the 4♠. Off the board, two sets of three can be laid down. The 8♦ that fits cannot be discarded. Six fours are two sets of three. A locked card can be discarded and cannot be hit. The 5♦ steals the joker. The card just taken can be discarded. `DrawFromDeck` is listed only for the penalty seat. A closed round lists nothing. `points` and `total_score` stay as they were. `feature/2.1-legal-moves` is kept.
- **2.2:** on `main` at `d815423`. Two random seats finish five rounds. Seed 1 leaves round 6 dealt and unplayed. `points` stay 0. Totals were 340 and 1025 before free melding. Every card id stays on the table. `feature/2.2-random-bot` is kept.
- **Free melding (post-board):** on `main` at `2a1dadb`. Once `is_on_board`, `PlayMeld` accepts any valid sets or runs without the round minimum. Move gen and the random bot lay those free melds. Seed 1 totals were 420 and 40 before the stall fix. Seeds 2, 3, 10, and 11 finish. A 1,000-game probe finished 966 and stalled 34 at 8,000 turns. `legacy_oracle` stays frozen on the old refuse. `fix/free-melding-post-board` is kept.
- **Stall fix:** on `main` at `a4708f2`. An empty penalty draw used to return to playing and clear `penalty_seat`. The hand keeps every card it drew. On the board, one discardable card goes out. If nothing can be laid or hit, that seat discards without taking. Seed 1 scores 340 and 80. Seeds 1..=10000 finished in 213.42s with 0 stalls.
- **Deck-out sudden death:** on `main` at `9de67aa`. `TurnDraw::Empty` during a penalty draw sets `round_over` and returns true. The phase stays penalty drawing. A safe card still returns to playing. Seed 1 still scores 340 and 80. `legacy_oracle` stays frozen.
- **2.2.1:** on `main` at `faa63d4`. The visit callback returns `bool` or `ControlFlow` and stops a play, hit, steal, or discard walk. The bot keeps 100 plays or hits. A hand above 11 stops after 10,000 search steps. The one-card `hand.len() > 11` bypass is gone. Release batch 0 finished 1000 of 1000 in 13.305s, inside 60s. Seed 1 scores 5 and 85. Stage 3's five batches with the bypass still in place finished in 20.350s, 24.099s, 21.034s, 20.484s, and 19.539s. The 14-card bench passed. `legacy_oracle` stays frozen. `feature/2.2.1-stage-3-throughput` is kept. Execution plan: [doc/REFACTOR_2_2_1.md](doc/REFACTOR_2_2_1.md).
- **2.3:** on `main` at `69534dd`. One thousand games run through `rayon::par_iter`. The lowest total wins. A tie is not a win. The CSV records seat win rates, average turns, and population score variance. Debug `test_rayon_parallelization` finished 1000 of 1000 in 12.77s. Seed 1 scores 5 and 85. `feature/2.3-parallel-csv` is kept.
- **2.4:** on `main`. Content is `f5868db`. A point-averse seat pushes and discards high cards and never holds a wild. A hoarder holds every wild through round 3. Keep(n) holds wilds while the hand has at most n, and sheds the surplus. A profile match deals 2 to 10 seats. `--players` copies seat 1 into every later seat. Two random seats still use `finish_random_game`. Seed 1 scores 5 and 85. `feature/2.4-strategic-bots` is kept.
- **Phase 2:** complete — 2026-10-09. Legal moves, a random bot, bounded search, Rayon batches, CSV metrics, and strategic profiles. The lowest total wins. A tie is not a win. On this Mac, 5 threads, release keep-N versus point-averse: at 3 seats keep-4 won 94,133 of 100,000 (mean 82.83); at 4 seats keep-5 won 79,222 (mean 144.32); at 5 seats keep-5 won 64,653 (mean 191.59). Pooled, keep-5 won 237,927 of 300,000 (mean 139.37). Keep-6 did not add wins.
- **Phase 3 forward:** stories **3.1–3.32** in [doc/BACKLOG.md](doc/BACKLOG.md). **3.5** is done. Next is **3.6**. The shell is `ios/CardGame/`. Apple's sample is read-only at `/Users/chaos/dev/AppleCardGameExample`, outside this workspace. Rules stay in `push_core`. Their drag, drop, reorder, faces, and backs stay theirs. Two seats. Seat 0 is the human. Seat 1 is one profile `push_sim` already has. One seed stays fixed. Opponent cards are their backs. Phase 3 closes at **3.32**: five rounds scored, the lowest total marked, round 6 dealt and unplayed, matching `push_core`.
- **3.1:** done. `ios/CardGame` launches their solitaire. No Push behavior. Public version **0.3.10**. Mac window, PASS. `feature/3.1-compile-sample` is kept. The Apple sample stays read-only outside the workspace.
- **3.2:** done. Their deck pile shows the undealt shoe from Rust. The exhibit shoe is 108. Public version **0.3.11**. Mac window, PASS. `feature/3.2-shoe` is kept.
- **3.3:** done. One natural rank and suit draws with their face view. The exhibit's first card is the four of hearts. The deck pile stays 108. Public version **0.3.12**. Mac window, PASS. `feature/3.3-one-face` is kept.
- **3.4:** done. A joker draws with their face view. The exhibit hand's joker is a white card, a black star, and the word Joker. The four of hearts stays. The deck pile stays 108. Public version **0.3.13**. Mac window, PASS. Game #9, 0 moves. `feature/3.4-joker-face` is kept.
- **3.5:** done. A two and a joker show a wild mark on that face. The mark is the word wild at the bottom. The joker stays a white card, a black star, and the word Joker. The two of clubs sits under it. The four of hearts stays. The deck pile stays 108. Public version **0.3.14**. Mac window, PASS. Game #9, 0 moves. `feature/3.5-wild-mark` is kept.
- **3.6:** A fixed Rust table shows the discard top with their face.
- **3.7:** Seat 0's hand sits in their hand layout. Nothing lifts into the engine.
- **3.8:** Seat 1's cards are their backs. The count is visible.
- **3.9:** One set from the fixture uses their pile layout.
- **3.10:** One run from the fixture uses their pile layout.
- **3.11:** The screen shows the round number and which seat is to move.
- **3.12:** A locked card is marked.
- **3.13:** Dropping the locked joker calls Rust and returns the card. The picture stays.
- **3.14:** A legal one-card hit calls `HitMeld`. The picture updates from Rust.
- **3.15:** A play that misses round 1's minimum is refused. The hand stays.
- **3.16:** Two sets of three call one `PlayMeld`. That seat is on the board.
- **3.17:** A later set, after that seat is on the board, calls `PlayMeld` with no round minimum.
- **3.18:** The wrong natural card is refused. The wild stays on the meld.
- **3.19:** The natural card replaces the wild. The wild enters the hand locked.
- **3.20:** Using that wild on the same turn is refused.
- **3.21:** Their discard control calls `TakeDiscard`. That card joins the hand.
- **3.22:** Their push control calls `PushDiscard`. The next seat receives the discard and one draw. The actor draws one.
- **3.23:** Off the board, discarding a card that fits is refused. The turn shows penalty drawing.
- **3.24:** One control calls `DrawFromDeck` until the safe card is the discard.
- **3.25:** A safe discard calls `DiscardCard` and the turn passes.
- **3.26:** An empty hand ends the round. Each seat's hand score is on the screen.
- **3.27:** Round 2 is dealt. The board is empty. Totals from round 1 remain.
- **3.28:** A penalty draw with nothing left ends the round. Those drawn cards are in the scores.
- **3.29:** One control plays seat 1's profile turn. The felt updates. A line names that action.
- **3.30:** The current round's minimum is on the screen.
- **3.31:** That seed plays all five rounds. The lowest total is marked.
- **3.32:** Round 6 is dealt and unplayed. The scores match a `push_core` test for that seed.
- **Retired iOS UI:** still on `main` at `d80ff16`. Content of the drop ship is `c61ef41`. Public version **0.3.9**. Deck size, the board, the hand row, and the Rust drop are in that tree. They are not the forward plan. Do not continue `ios/PushUI`. `feature/3.1-uniffi-deck-size`, `feature/3.2-board-ui`, `feature/3.3-drag-drop`, `feature/3.3.2-hand-row`, `feature/3.3.3-on-screen-drag`, and `feature/3.3.4-engine-drop` stay. A rewind of `main` to `6d52fbd` is not done.
- **Phase 1b:** open. Not started. Parallel to Phase 3. It does not block 3.1. Stories are in [doc/BACKLOG.md](doc/BACKLOG.md). One seed, four seats, profiles `push_sim` already has. `feature/1b-wasm-tick` is a two-seat sketch from `6d52fbd`. Do not branch from it.
- **1b.1:** Four dealt hands on `viewer/`. Draw, discard, Round 1. The board is empty.
- **1b.2:** One step. The log names the seat, the action, and the cards.
- **1b.3:** The first meld is cards on the felt.
- **1b.4:** Round 1 ends. Four scores show. Round 2 is dealt and the board is empty.
- **1b.5:** Five rounds play. Round 6 is dealt and unplayed. A rule-book mismatch becomes a `push_core` test.
- **Next (iOS):** **3.6**. A fixed Rust table shows the discard top with their face.
- **Next (viewer):** 1b.1 when asked.
- **Full backlog:** [doc/BACKLOG.md](doc/BACKLOG.md)
- **Commentary / deltas:** [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md) (retired 3.3 split; integration-chain Epic 1.9 suggested; Google export quirks)

## Version

Plan coordinate starts at **0.1.0**. Phase 2 stayed there (headless sim). The retired drop UI was **0.3.9**. Story **3.1** is **0.3.10**. Story **3.2** is **0.3.11**. Story **3.3** is **0.3.12**. Story **3.4** is **0.3.13**. Story **3.5** is **0.3.14**. A later story with no screen change does not bump. Each later screen change adds one. See [doc/VERSIONING.md](doc/VERSIONING.md).

Keep in sync with AGENT_HANDOFF *Current state* and [doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md).
