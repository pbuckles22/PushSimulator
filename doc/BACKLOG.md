# **Master Project Backlog: Push (Zero-Defect Execution)**

This master backlog translates the entire Project Management Plan into ultra-granular, sequential tasks. The AI Agent must execute these in exact order using a strict Red/Green/Refactor TDD cycle.

* **Red:** Write the test listed on the line. Run it. Watch it fail.  
* **Green:** Write the absolute minimum code to pass that specific test.  
* **Refactor:** Clean up the code. Run all previous tests to ensure no regressions. Move to the next line.

## **Phase 1: The Rust Core Engine**

### **Epic 1.1: Primitives & Deck Management**

**Status:** complete — 2026-10-06. Cards, wilds, a 108-card deck, shuffle, and draw. An empty draw pile leaves the top card when three or more discards remain. One leftover card goes to the current player. Two leftover cards are shuffled and split between the current player and the next.

**User Story 1.1.1: Card Definitions**

*As the game engine, I want to define a playing card so that I have a foundational object for all game mechanics.*

* \[x\] Write test test\_suit\_enum\_instantiation (Hearts, Diamonds, Clubs, Spades, None).  
* \[x\] Implement Suit enum.  
* \[x\] Write test test\_rank\_enum\_instantiation (Two through Ace, Joker).  
* \[x\] Implement Rank enum.  
* \[x\] Write test test\_card\_struct\_instantiation asserting a Card requires id, suit, rank, and locked\_until\_turn.  
* \[x\] Implement Card struct matching the Data Schema.

**User Story 1.1.2: Wild Card Identification**

*As the game engine, I want cards to know if they are wild so that I can apply special validation and scoring rules.*

* \[x\] Write test test\_card\_is\_wild\_true\_for\_joker.  
* \[x\] Implement is\_wild() returning true for Jokers.  
* \[x\] Write test test\_card\_is\_wild\_true\_for\_two.  
* \[x\] Update is\_wild() to also return true for Twos.  
* \[x\] Write test test\_card\_is\_wild\_false\_for\_standard\_card.  
* \[x\] Ensure is\_wild() returns false for standard ranks.

**User Story 1.1.3: Deck Generation**

*As the game engine, I want to generate a standard "Push" deck so that players have the correct 108 cards.*

* \[x\] Write test test\_deck\_instantiation\_count asserting a new deck has exactly 108 cards.  
* \[x\] Implement Deck struct and Deck::new() (2 decks x 52 cards \+ 4 Jokers).  
* \[x\] Write test test\_deck\_contains\_exact\_wild\_count asserting exactly 8 Twos and 4 Jokers.  
* \[x\] Refactor Deck::new() if necessary to pass test.

**User Story 1.1.4: Shuffling & Drawing**

*As the game engine, I want to shuffle and draw from the deck so that gameplay is randomized and state advances.*

* \[x\] Write test test\_deck\_shuffle asserting the order of cards changes after shuffling.  
* \[x\] Add rand crate and implement deck.shuffle().  
* \[x\] Write test test\_deck\_draw\_reduces\_count asserting drawing 1 card reduces deck size.  
* \[x\] Implement deck.draw() returning Option\<Card\>.  
* \[x\] Write test test\_deck\_draw\_empty\_reshuffles asserting an empty draw pile with three or more discard cards takes that pile except its top card, shuffles, and draws.  
* \[x\] Implement that reshuffle.  
* \[x\] Write test test\_deck\_draw\_last\_card\_goes\_to\_current and test\_deck\_draw\_last\_two\_split. One leftover card goes to the current player. Two leftover cards are shuffled and split between the current player and the next.  
* \[x\] Implement the one-card and two-card leftovers. Both piles empty still draws nothing.

### **Epic 1.2: Player State & Scoring**

**Status:** complete — 2026-10-06. Player, deal, card penalties, and the hand total. **1.2.4.3** is closed: a short draw pile deals the cards it has, then panics. **1.2.4.2** seeds shuffle and draw and closes Epic 1.2. Epic 1.4 is complete — 2026-10-06. **1.3.1**, **1.3.2**, **1.4.1**, and **1.4.2** are on `main`.

**User Story 1.2.1: Player Initialization**

*As the game engine, I want to create a Player so that I can track their hand, board status, and score.*

* \[x\] Write test test\_player\_instantiation asserting a new player starts with 0 points and is\_on\_board \= false.  
* \[x\] Implement Player struct and Player::new().

**User Story 1.2.2: Dealing Hands**

*As the game engine, I want to deal cards to a player so that they have a starting hand.*

* \[x\] Write test test\_deal\_initial\_hand asserting two or more players each receive exactly 10 cards, one card at a time.  
* \[x\] Implement deal\_initial\_hands. One player is refused.

**User Story 1.2.3: Point Calculations**

*As the game engine, I want to calculate the value of any card so that penalties can be tallied.*

* \[x\] Write test test\_score\_card\_pip asserting ranks 3-9 return 5 points.  
* \[x\] Implement pip scoring in get\_penalty\_value().  
* \[x\] Write test test\_score\_card\_face asserting 10-K returns 10 points.  
* \[x\] Implement face card scoring.  
* \[x\] Write test test\_score\_card\_ace asserting Aces return 15 points.  
* \[x\] Implement Ace scoring.  
* \[x\] Write test test\_score\_card\_wild asserting 2s and Jokers return 20 points.  
* \[x\] Implement wild card scoring.  
* \[x\] Write test test\_calculate\_hand\_total asserting hand sums correctly.  
* \[x\] Implement calculate\_hand\_penalty() iterating over hand.

Wild scoring is sprint story **1.2.5**. The two hand-total lines above are sprint story **1.2.6**. They stay under this heading because the Google export listed every penalty step on story 1.2.3. The sprint order is pips, then face and ace, then wilds, then the hand total.

Face and ace scoring is sprint story **1.2.4.1**. **1.2.4.3** below is closed. **1.2.4.2** seeds shuffle and draw and closes Epic 1.2. Epic 1.4 is complete — 2026-10-06. **1.3.1**, **1.3.2**, **1.4.1**, and **1.4.2** are on `main`.

**1.2.4.2: Tech debt**

* \[x\] Seed `Deck::shuffle` in tests, or inject the rng, so the order check does not depend on `thread_rng` (`TECH_DEBT.md`). Then add the property test that the same card ids survive shuffle, draw, and a three-or-more reshuffle. Fixed chain tests already lock those ids.  
* \[x\] Remove the uncalled arms, or make them real behavior: `reshuffle_discard` on an empty discard, `reshuffle_discard` putting a single card back, and `TurnDraw::Empty` after a reshuffle. `draw` does not reach those paths.  
* \[x\] Keep one chain test that `Player::new` does not take a drawn card. The other "player starts empty" chains repeat that.  
* \[x\] Add CI that runs `cargo test -p push_core`. The 2026-10-06 LLVM run (58 tests, 98.1% lines) stays a local measurement until a coverage gate is chosen.  
* \[x\] Name the evidence sink in `TEST_PLAN.md`, or leave one explicit TBD. `TEST_PLAN.md` leaves it TBD until a runtime log exists.  
* \[x\] Drop the unused `serde` dependency on `push_core`, or use it (`TECH_DEBT.md`).  
This closes Epic 1.2. Epic 1.4 is complete — 2026-10-06.

**1.2.4.3: Test gaps**

* \[x\] Write test test\_deal\_initial\_hand\_short\_draw\_pile. A draw pile of three cards deals those three, then panics. The chain is `test_suit_rank_card_deck_new_is_wild_shuffle_deal_short_draw_pile`.  
* \[x\] Do not add a `should_panic` test for `get_penalty_value` on a Two or a joker. **1.2.5** scores those 20. Hand total is **1.2.6**. Face cards score 10 and an ace scores 15.  
* \[x\] Card ids already survive shuffle, draw, and a three-or-more reshuffle in the fixed chain tests. A property test waits on the seeded rng in **1.2.4.2**.  
* \[x\] Doc-tests stay 0 until a public function's rustdoc example is the lock.  
* \[x\] `push_wasm` and `push_sim` stay untested here. Tier 2 starts with those phases.

### **Epic 1.3: The "Push" & Draw Phase**

**Status:** complete — 2026-10-06. Take the top discard, or push it to the next player with a draw-pile card. The pushing player then draws.

**User Story 1.3.1: Standard Draw**

**Status:** on `main` — 2026-10-06 at `6a00eff`. The discard pile's top card moves into the player's hand. `feature/1.3.1-take-discard` is kept.

*As the game engine, I want a player to take a discard so they can add it to their hand.*

* \[x\] Write test test\_take\_discard asserting the discard pile's top card moves to the player's hand.  
* \[x\] Implement Action::TakeDiscard handler.

**User Story 1.3.2: Pushing a Discard**

**Status:** on `main` — 2026-10-06 at `1500b02`. The next player receives the top discard and the top draw-pile card. The pushing player then draws. `feature/1.3.2-push-discard` is kept.

*As the game engine, I want a player to push a discard so they can force a penalty on the next player.*

* \[x\] Write test test\_push\_discard\_moves\_card asserting Player 1 pushing a discard moves it to Player 2's hand.  
* \[x\] Implement Action::PushDiscard handler transferring the card.  
* \[x\] Write test test\_push\_discard\_penalty\_draw asserting Player 2 also receives a card from the top of the Draw Pile.  
* \[x\] Update PushDiscard handler to enforce the penalty draw.  
* \[x\] Write test test\_push\_turn\_advancement asserting Player 1 now draws from the deck to start their real turn.  
* \[x\] Update state machine to handle post-push turn execution.

### **Epic 1.4: Validation Engine**

**Status:** complete — 2026-10-06. A set is three or more cards of one rank. A run is four or more cards of one suit in order. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run. `feature/1.4.1-validate-sets` and `feature/1.4.2-validate-runs` stay. This close is on `main`. Content is `2341904`. `feature/1.4-swat` stays.

**User Story 1.4.1: Validating Sets**

**Status:** on `main` — 2026-10-06 at `62220b3`. Three or more cards of one rank are a set. Twos and jokers fill that rank. `feature/1.4.1-validate-sets` is kept.

*As the game engine, I want to validate Sets so that players cannot play illegal groups of cards.*

* \[x\] Write test test\_validate\_set\_naturals asserting \[4H, 4S, 4C\] is true, \[4H, 4S, 5C\] is false.  
* \[x\] Implement validate\_set() logic checking ranks.  
* \[x\] Write test test\_validate\_set\_wilds asserting \[4H, Joker, 2S\] is true, \[Joker, Joker, Joker\] is true.  
* \[x\] Update validate\_set() to accommodate wild cards gracefully.

**User Story 1.4.2: Validating Runs & Aces**

**Status:** on `main` — 2026-10-06 at `da56999`. Four or more cards of one suit in order are a run. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run. `feature/1.4.2-validate-runs` is kept.

*As the game engine, I want to validate Runs so that sequential suit plays are strictly enforced.*

* \[x\] Write test test\_validate\_run\_naturals asserting \[4H, 5H, 6H, 7H\] is true, mixed suits false.  
* \[x\] Implement validate\_run() logic checking suits and sequential ranks.  
* \[x\] Write test test\_validate\_run\_wilds asserting \[4H, Joker, 6H, 2S\] is true.  
* \[x\] Update validate\_run() to infer missing sequential values using wild cards.  
* \[x\] Write test test\_ace\_placement\_low asserting \[AH, 2C, 3H, 4H\] is true (2 is wild).  
* \[x\] Update validate\_run() for Ace \= 1 logic.  
* \[x\] Write test test\_ace\_placement\_high asserting \[JH, QH, KH, AH\] is true.  
* \[x\] Update validate\_run() for Ace \= 14 logic.  
* \[x\] Write test test\_ace\_wrap\_rejection asserting \[KH, AH, 2C, 3H\] is false.  
* \[x\] Ensure wrap-around logic is strictly forbidden.

### **Epic 1.5: Melding & Getting on the Board**

**Status:** complete — 2026-10-07. A player gets on the board by meeting that round's minimum in one play. Round 1 is two sets of 3. Round 2 is a set of 3 and a run of 4. Round 3 is two runs of 4. Round 4 is three sets of 3. Round 5 is a set of 3 and a run of 7. A meld may be larger than the minimum. An extra meld fails. `Action::PlayMeld` moves those melds onto the board and sets `is_on_board`. The close is on `main`. Content is `e0e1204`. `feature/1.5-swat` stays.

**User Story 1.5.1: Round Requirements**

*As the game engine, I want to strictly enforce round minimums before letting a player on the board.*

* \[x\] Write test test\_round\_1\_minimum\_rejection asserting playing one Set of 3 fails in Round 1\.  
* \[x\] Implement check\_round\_requirements(round\_number, melds).  
* \[x\] Write test test\_round\_1\_minimum\_acceptance asserting playing two Sets of 3 passes.  
* \[x\] Write test test\_round\_1\_exceeding\_minimum asserting playing two Sets of 4 passes.  
* \[x\] Refactor checking logic to allow \> minimum sizes.

**User Story 1.5.2: Board Entry Execution**

*As the game engine, I want to move verified melds to the board and flag the player.*

* \[x\] Write test test\_play\_meld\_success asserting valid cards move from hand to GameState.board.  
* \[x\] Implement Action::PlayMeld handler.  
* \[x\] Write test test\_play\_meld\_flags\_player asserting successful play sets player.is\_on\_board \= true.  
* \[x\] Update Action::PlayMeld state mutation.

### **Epic 1.6: Hitting & Wild Card Stealing**

**Status:** complete — 2026-10-07. A player on the board can add cards onto melds already there, and can swap a natural card for a wild. That wild locks until the turn counter plus one and cannot be played until the counter catches up. The close is on `main`. Content is `2abbf4e`. The close commit is `e5de020`. `feature/1.6.3-wild-lock` stays.

**User Story 1.6.1: Hitting Existing Melds**

*As the game engine, I want players to add cards to existing melds ONLY if they are on the board.*

* \[x\] Write test test\_hit\_rejection\_if\_not\_on\_board asserting action fails if is\_on\_board \= false.  
* \[x\] Implement guard in Action::HitMeld handler.  
* \[x\] Write test test\_valid\_hit\_set asserting adding 8H to \[8S, 8C, Joker\] works.  
* \[x\] Implement hit logic for Sets.  
* \[x\] Write test test\_valid\_hit\_run asserting adding 8H to \[5H, 6H, 7H\] works.  
* \[x\] Implement hit logic for Runs.

**User Story 1.6.2: Stealing Wild Cards**

*As the game engine, I want players to swap natural cards for wild cards on the board.*

* \[x\] Write test test\_wild\_steal\_success asserting a 5D replaces a Joker, and the Joker moves to hand.  
* \[x\] Implement Action::StealWild handler.  
* \[x\] Write test test\_wild\_steal\_applies\_lock asserting the stolen Joker receives locked\_until\_turn \= global\_turn\_counter \+ 1\.  
* \[x\] Update StealWild to calculate and apply the lock duration.

**User Story 1.6.3: Wild Card Lock Enforcement**

*As the game engine, I want to prevent players from using stolen wild cards immediately.*

* \[x\] Write test test\_stolen\_wild\_play\_rejection asserting playing a locked wild card fails.  
* \[x\] Update validation engine to check locked\_until\_turn against global\_turn\_counter.  
* \[x\] Write test test\_stolen\_wild\_play\_acceptance asserting play succeeds after turn counter advances.  
* \[x\] Ensure state machine properly advances turn counters.

### **Epic 1.7: Discarding & Penalties**

**Status:** complete — 2026-10-07. Content is `6b97524`.

**User Story 1.7.1: Safe vs. Unsafe Discards**

*As the game engine, I want to enforce that off-board players cannot discard playable cards.*

* \[x\] Write test test\_quick\_discard asserting a drawn card can be immediately discarded.  
* \[x\] Implement tracking of the currently drawn card to bypass safety checks.  
* \[x\] Write test test\_pre\_board\_discard\_rejection asserting discarding 7H when a \[5H, 6H, 8H(wild)\] exists fails.  
* \[x\] Implement board-scanning logic inside Action::DiscardCard to detect playability.

**User Story 1.7.2: The Penalty Draw Loop**

*As the game engine, I want to trap players who cannot discard into drawing until they find a safe card.*

* \[x\] Write test test\_penalty\_draw\_initiation asserting failure to discard changes state to TurnPhase::PenaltyDrawing.  
* \[x\] Implement phase change on failed discard.  
* \[x\] Write test test\_penalty\_draw\_execution mocking 3 playable draws and 1 safe draw, asserting hand size increases and turn ends on safe discard.  
* \[x\] Implement loop logic in Action::DrawFromDeck when in Penalty phase.

**User Story 1.7.3: Round Victory**

*As the game engine, I want to end the round immediately when a hand is empty.*

* \[x\] Write test test\_round\_victory\_on\_hit asserting round ends if hitting leaves 0 cards in hand (no discard needed).  
* \[x\] Implement round end trigger in HitMeld and PlayMeld.  
* \[x\] Write test test\_round\_victory\_on\_discard asserting round ends if discard leaves 0 cards.  
* \[x\] Implement round end trigger in DiscardCard.

### **Epic 1.8: System Integration (The Crucible)**

**Status:** complete — 2026-10-07. A drawn joker stays in the hand when the king is discarded. Content is `b2303b9`. `feature/1.8.5-trapped-by-a-draw` stays.

**User Story 1.8.1: Steal, Hold, and Win**

*As the game engine, I want to ensure complex turn chains resolve perfectly.*

* \[x\] Write test test\_steal\_hold\_win matching PM plan scenario (Steal Joker \-\> wait \-\> win).  
* \[x\] Execute test and debug any state machine failures.

**User Story 1.8.2: Pushed Penalty Trap**

*As the game engine, I want to ensure pushed penalties cascade correctly into discard penalties.*

* \[x\] Write test test\_pushed\_penalty\_trap matching PM plan scenario.  
* \[x\] Execute test and debug.

**User Story 1.8.3: Omniturn Execution**

*As the game engine, I want to ensure a player can do everything in one turn.*

* \[x\] Write test test\_multi\_action\_omniturn (Get on board \-\> Hit 2x \-\> Steal Wild \-\> Discard).  
* \[x\] Execute test and debug.

**User Story 1.8.4: Round Transition & Wipe**

*As the game engine, I want to calculate scores and wipe the board for the next round.*

* \[x\] Write test test\_round\_transition\_and\_wipe checking point tallies and exact deck/board reset.  
* \[x\] Implement advance\_to\_next\_round() logic.

**User Story 1.8.5: Trapped by a Draw**

*As the game engine, I want to ensure drawing a wild card during a penalty loop doesn't break the game.*

* \[x\] Write test test\_trapped\_by\_a\_draw (Draw Joker \-\> Draw King \-\> Discard King \-\> Stuck with Joker).  
* \[x\] Execute test and debug.

## **Phase 2: Monte Carlo Simulator**

**User Story 2.1: Legal Move Generation**

*As the simulator, I need to know every valid action so my AI can choose one.*

* \[x\] Write test test\_generate\_legal\_moves checking a given hand against board state.  
* \[x\] Implement generate\_legal\_moves(game\_state) \-\> Vec\<Action\>.

**User Story 2.2: Random Bot & Simulation Loop**

*As the simulator, I want bots to play headless games so I can test completion.*

* \[x\] Write test test\_headless\_random\_game asserting two random bots can finish 5 rounds.  
* \[x\] Implement Random Bot AI logic.

**User Story 2.2.1: Move Generation Performance & Combinatorics**

*As the simulator, I need clone-free validation and bounded move-search costs so large Monte Carlo batches are practical without changing legal behavior.*

* \[x\] Execute the deterministic validation, differential-oracle, lazy-generation, and benchmark plan in [REFACTOR_2_2_1.md](REFACTOR_2_2_1.md). Phases 0–3 and Phase 4 are on `main`. Seed 1 scores 5 and 85. Release batch 0 finished 1000 of 1000 in 13.305s.
* \[x\] Remove the large-hand fallback. `visit_legal_kind` stops after a capped sample. A hand above 11 stops after 10,000 search steps. The one-card bypass is gone.

**User Story 2.3: Parallelization & Metric Logging**

*As the simulator, I want to run millions of games fast so I get valid data.*

* \[x\] Write test test\_rayon\_parallelization wrapping 1,000 games in rayon::par\_iter().  
* \[x\] Implement concurrent game loop.  
* \[x\] Implement CSV logging (Seat win rates, average turns, score variance).

**User Story 2.4: Strategic Bots**

**Status:** complete — 2026-10-09. Content is `f5868db`. A point-averse seat never holds a wild. A hoarder holds every wild through round 3. Keep(n) holds at most n wilds. A match deals 2 to 10 seats. `feature/2.4-strategic-bots` is kept. Phase 2 is complete.

*As the simulator, I want different AI profiles to discover optimal meta-strategies.*

* \[x\] Implement "Point-Averse Bot" (Aggressively pushes/discards high cards).  
* \[x\] Implement "Hoarder Bot" (Holds Wilds until late rounds).  
* \[x\] Run 100,000 iterations to compare profiles.

## **Phase 1b: Watch a four-person game**

**Status:** open. Not started. This track is parallel to Phase 3. It does not sit in front of 3.3.1. An iOS thread starts at 3.3.1. A viewer thread starts at 1b.1 and stays in this order. Rules stay in `push_core`. The page only shows them. A mismatch with the rule book becomes a `push_core` test. `feature/1b-wasm-tick` is a two-seat sketch from `6d52fbd`. Do not branch from it. Branch each story from current `main`.

The page is `viewer/`. The bridge is `push_wasm`. Four seats come from `finish_profile_game` / `play_profile_turn` and the profiles `push_sim` already has. One seed and one lineup stay fixed so the same game can be replayed. Each story is done when that page is what you see.

**User Story 1b.1: Four dealt hands**

*As a reviewer, I want a dealt four-person table on the page before anyone plays.*

* \[ \] Deal four seats from the fixed seed through `push_wasm`.  
* \[ \] Show four hands, the draw pile, the discard, and Round 1.  
* \[ \] Leave the board empty. No action has been applied.

**User Story 1b.2: One step, one log line**

*As a reviewer, I want one click to play one seat and say what it did.*

* \[ \] Step calls `play_profile_turn` for the seat whose turn it is.  
* \[ \] The felt updates to that table.  
* \[ \] The log names the seat, the action (push, take, meld, hit, steal, or discard), and the cards.

**User Story 1b.3: First meld on the felt**

*As a reviewer, I want to see the first lay-down as cards.*

* \[ \] Stepping continues until a seat is on the board.  
* \[ \] That meld is cards on the felt.  
* \[ \] The log shows the lay-down.

**User Story 1b.4: Round 1 ends**

*As a reviewer, I want to see a round finish and the next deal.*

* \[ \] Play continues until round 1 is over.  
* \[ \] The four scores are on the page.  
* \[ \] The next picture is round 2 dealt: the board is empty and the hands are dealt.

**User Story 1b.5: Full match**

*As a reviewer, I want to watch all five rounds and compare the flow to the rule book.*

* \[ \] Play runs the five rounds. Round 6 is dealt and unplayed.  
* \[ \] The log is still on the page.  
* \[ \] A rule-book mismatch is recorded as a `push_core` test, not a rule inside the page.

## **Phase 3: iOS SwiftUI App (Local Play)**

**User Story 3.1: Rust/Swift Bridge**

**Status:** on `main`. Content is `398e20a`. Public version **0.3.1**. `Game::get_deck_size` is 108. The SwiftUI screen shows Deck size 108. `feature/3.1-uniffi-deck-size` is kept.

*As an iOS developer, I need to call my Rust engine from Swift so I don't have to rewrite the rules.*

* \[x\] Generate .xcframework using UniFFI (or Swift-Bridge).  
* \[x\] Write basic SwiftUI view calling Game.get\_deck\_size() and rendering text.

**User Story 3.2: Reactive UI & Board Rendering**

**Status:** on `main`. Content is `adac445`. Public version **0.3.2**. The screen shows 0.3.2, Deck size 108, Round 2, three fours, and 4♥ 5♥ 6♥ 7♥. `feature/3.2-board-ui` is kept.

*As a player, I want to see the state of the game natively on my screen.*

* \[x\] Implement @Published state wrapping the Rust GameState.  
* \[x\] Build SwiftUI CardView changing dynamically based on Suit/Rank.  
* \[x\] Build SwiftUI BoardView laying out Sets and Runs.

**User Story 3.3.1: Drop API**

**Status:** on `main`. Content is `9afe342`. Public version stays **0.3.2**. The screen is still 3.2. `swift test --package-path ios/PushUI` is green (35 tests). `feature/3.3-drag-drop` is kept.

*As a player, I want a drop to become a play or a hit before the screen animates.*

* \[x\] `CardDrag` is `Transferable`.  
* \[x\] A new-meld drop is one `PlayMeld` group. A meld drop is one `HitMeld`.  
* \[x\] `settleDrop` returns the same hand and board on `GameError` and on a refusal. An accept returns the engine table.  
* \[x\] The published picture still has no hand.

**User Story 3.3.2: Hand row**

*As a player, I want to see my hand on the screen so I know what I can drag.*

* \[ \] The local seat’s hand is on the screen.  
* \[ \] Opponent hands stay off the picture.  
* \[ \] No drag yet.

**User Story 3.3.3: On-screen drag**

*As a player, I want to drag a hand card onto a meld or a new-meld zone.*

* \[ \] A drop calls the 3.3.1 API.  
* \[ \] A `GameError` or a refusal leaves the card in the hand.  
* \[ \] The picture updates only on accept.  
* \[ \] The engine is still a stand-in.

**User Story 3.3.4: Engine drop**

*As a player, I want that drop to go through Rust so the rules decide.*

* \[ \] UniFFI applies `PlayMeld` or `HitMeld`.  
* \[ \] `GameError` and a refusal come from Rust.  
* \[ \] The shoe on the exhibit stays 108.

Take, discard, steal, and scores stay out of 3.3.1–3.3.4.

## **Phase 4: Multiplayer Server**

**User Story 4.1.1: Axum listens**

*As a remote player, I want a server process that accepts connections.*

* \[ \] Axum binds and serves a health or ready route.

**User Story 4.1.2: Headless WebSocket handshake**

*As a remote player, I want a test client to complete the handshake.*

* \[ \] A headless client opens the WebSocket and finishes the handshake.

**User Story 4.1.3: iOS WebSocket handshake**

*As a remote player, I want the iOS client to complete that same handshake.*

* \[ \] The iOS client opens the WebSocket and finishes the handshake.

**User Story 4.2.1: Room code**

*As a remote player, I want a 4-digit code that names my room.*

* \[ \] A room has a 4-digit code.

**User Story 4.2.2: Room engine**

*As a remote player, I want that room to hold one game.*

* \[ \] That room holds one `GameState`.

**User Story 4.3.1: Drop intent JSON**

*As a remote player, I want my drag to travel as JSON.*

* \[ \] A 3.3.1 drop intent serializes to JSON.

**User Story 4.3.2: Server applies the drop**

*As a remote player, I want the server to run that drop through the engine.*

* \[ \] The server applies that JSON through `push_core` and returns the snapshot.

**User Story 4.3.3: Opponent sees the snapshot**

*As a remote player, I want the other client to show that snapshot.*

* \[ \] The other client shows that snapshot.

## **Final Milestone: CI/CD Pipeline**

**User Story 5.1: Automated Testing Enforcement**

*As the lead developer, I want to prevent bad code from reaching the main branch.*

* \[x\] Create GitHub Actions YAML for Rust toolchain. `.github/workflows/push-core.yml` runs `cargo test` for `push_core`, `push_sim`, and `push_ffi`.

**User Story 5.1.1: Swift UI tests in CI**

*As the lead developer, I want the iOS package tests to run on every push.*

* \[ \] Run `swift test --package-path ios/PushUI` in CI.

Branch protection that requires a green suite stays a later checkbox. It does not jump ahead of 3.3.2.