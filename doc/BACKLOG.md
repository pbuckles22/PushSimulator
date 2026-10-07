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

**Status:** complete — 2026-10-06. Player, deal, card penalties, and the hand total. **1.2.4.3** is closed: a short draw pile deals the cards it has, then panics. **1.2.4.2** is next, only when asked. Epic 1.4 stays open until that lands. **1.3.1**, **1.3.2**, **1.4.1**, and **1.4.2** are on `main`.

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

Face and ace scoring is sprint story **1.2.4.1**. **1.2.4.3** below is closed. **1.2.4.2** is next, only when asked. Epic 1.4 stays open until that lands. **1.3.1**, **1.3.2**, **1.4.1**, and **1.4.2** are on `main`.

**1.2.4.2: Tech debt**

* \[ \] Seed `Deck::shuffle` in tests, or inject the rng, so the order check does not depend on `thread_rng` (`TECH_DEBT.md`). Then add the property test that the same card ids survive shuffle, draw, and a three-or-more reshuffle. Fixed chain tests already lock those ids.  
* \[ \] Remove the uncalled arms, or make them real behavior: `reshuffle_discard` on an empty discard, `reshuffle_discard` putting a single card back, and `TurnDraw::Empty` after a reshuffle. `draw` does not reach those paths.  
* \[ \] Keep one chain test that `Player::new` does not take a drawn card. The other "player starts empty" chains repeat that.  
* \[ \] Add CI that runs `cargo test -p push_core`. The 2026-10-06 LLVM run (58 tests, 98.1% lines) stays a local measurement until a coverage gate is chosen.  
* \[x\] Name the evidence sink in `TEST_PLAN.md`, or leave one explicit TBD. `TEST_PLAN.md` leaves it TBD until a runtime log exists.  
* \[ \] Drop the unused `serde` dependency on `push_core`, or use it (`TECH_DEBT.md`).  
Hit this before Epic 1.4 closes. It is the next story, only when asked.

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

**User Story 1.5.1: Round Requirements**

*As the game engine, I want to strictly enforce round minimums before letting a player on the board.*

* \[ \] Write test test\_round\_1\_minimum\_rejection asserting playing one Set of 3 fails in Round 1\.  
* \[ \] Implement check\_round\_requirements(round\_number, melds).  
* \[ \] Write test test\_round\_1\_minimum\_acceptance asserting playing two Sets of 3 passes.  
* \[ \] Write test test\_round\_1\_exceeding\_minimum asserting playing two Sets of 4 passes.  
* \[ \] Refactor checking logic to allow \> minimum sizes.

**User Story 1.5.2: Board Entry Execution**

*As the game engine, I want to move verified melds to the board and flag the player.*

* \[ \] Write test test\_play\_meld\_success asserting valid cards move from hand to GameState.board.  
* \[ \] Implement Action::PlayMeld handler.  
* \[ \] Write test test\_play\_meld\_flags\_player asserting successful play sets player.is\_on\_board \= true.  
* \[ \] Update Action::PlayMeld state mutation.

### **Epic 1.6: Hitting & Wild Card Stealing**

**User Story 1.6.1: Hitting Existing Melds**

*As the game engine, I want players to add cards to existing melds ONLY if they are on the board.*

* \[ \] Write test test\_hit\_rejection\_if\_not\_on\_board asserting action fails if is\_on\_board \= false.  
* \[ \] Implement guard in Action::HitMeld handler.  
* \[ \] Write test test\_valid\_hit\_set asserting adding 8H to \[8S, 8C, Joker\] works.  
* \[ \] Implement hit logic for Sets.  
* \[ \] Write test test\_valid\_hit\_run asserting adding 8H to \[5H, 6H, 7H\] works.  
* \[ \] Implement hit logic for Runs.

**User Story 1.6.2: Stealing Wild Cards**

*As the game engine, I want players to swap natural cards for wild cards on the board.*

* \[ \] Write test test\_wild\_steal\_success asserting a 5D replaces a Joker, and the Joker moves to hand.  
* \[ \] Implement Action::StealWild handler.  
* \[ \] Write test test\_wild\_steal\_applies\_lock asserting the stolen Joker receives locked\_until\_turn \= global\_turn\_counter \+ 1\.  
* \[ \] Update StealWild to calculate and apply the lock duration.

**User Story 1.6.3: Wild Card Lock Enforcement**

*As the game engine, I want to prevent players from using stolen wild cards immediately.*

* \[ \] Write test test\_stolen\_wild\_play\_rejection asserting playing a locked wild card fails.  
* \[ \] Update validation engine to check locked\_until\_turn against global\_turn\_counter.  
* \[ \] Write test test\_stolen\_wild\_play\_acceptance asserting play succeeds after turn counter advances.  
* \[ \] Ensure state machine properly advances turn counters.

### **Epic 1.7: Discarding & Penalties**

**User Story 1.7.1: Safe vs. Unsafe Discards**

*As the game engine, I want to enforce that off-board players cannot discard playable cards.*

* \[ \] Write test test\_quick\_discard asserting a drawn card can be immediately discarded.  
* \[ \] Implement tracking of the currently drawn card to bypass safety checks.  
* \[ \] Write test test\_pre\_board\_discard\_rejection asserting discarding 7H when a \[5H, 6H, 8H(wild)\] exists fails.  
* \[ \] Implement board-scanning logic inside Action::DiscardCard to detect playability.

**User Story 1.7.2: The Penalty Draw Loop**

*As the game engine, I want to trap players who cannot discard into drawing until they find a safe card.*

* \[ \] Write test test\_penalty\_draw\_initiation asserting failure to discard changes state to TurnPhase::PenaltyDrawing.  
* \[ \] Implement phase change on failed discard.  
* \[ \] Write test test\_penalty\_draw\_execution mocking 3 playable draws and 1 safe draw, asserting hand size increases and turn ends on safe discard.  
* \[ \] Implement loop logic in Action::DrawFromDeck when in Penalty phase.

**User Story 1.7.3: Round Victory**

*As the game engine, I want to end the round immediately when a hand is empty.*

* \[ \] Write test test\_round\_victory\_on\_hit asserting round ends if hitting leaves 0 cards in hand (no discard needed).  
* \[ \] Implement round end trigger in HitMeld and PlayMeld.  
* \[ \] Write test test\_round\_victory\_on\_discard asserting round ends if discard leaves 0 cards.  
* \[ \] Implement round end trigger in DiscardCard.

### **Epic 1.8: System Integration (The Crucible)**

**User Story 1.8.1: Steal, Hold, and Win**

*As the game engine, I want to ensure complex turn chains resolve perfectly.*

* \[ \] Write test test\_steal\_hold\_win matching PM plan scenario (Steal Joker \-\> wait \-\> win).  
* \[ \] Execute test and debug any state machine failures.

**User Story 1.8.2: Pushed Penalty Trap**

*As the game engine, I want to ensure pushed penalties cascade correctly into discard penalties.*

* \[ \] Write test test\_pushed\_penalty\_trap matching PM plan scenario.  
* \[ \] Execute test and debug.

**User Story 1.8.3: Omniturn Execution**

*As the game engine, I want to ensure a player can do everything in one turn.*

* \[ \] Write test test\_multi\_action\_omniturn (Get on board \-\> Hit 2x \-\> Steal Wild \-\> Discard).  
* \[ \] Execute test and debug.

**User Story 1.8.4: Round Transition & Wipe**

*As the game engine, I want to calculate scores and wipe the board for the next round.*

* \[ \] Write test test\_round\_transition\_and\_wipe checking point tallies and exact deck/board reset.  
* \[ \] Implement advance\_to\_next\_round() logic.

**User Story 1.8.5: Trapped by a Draw**

*As the game engine, I want to ensure drawing a wild card during a penalty loop doesn't break the game.*

* \[ \] Write test test\_trapped\_by\_a\_draw (Draw Joker \-\> Draw King \-\> Discard King \-\> Stuck with Joker).  
* \[ \] Execute test and debug.

## **Phase 2: Monte Carlo Simulator**

**User Story 2.1: Legal Move Generation**

*As the simulator, I need to know every valid action so my AI can choose one.*

* \[ \] Write test test\_generate\_legal\_moves checking a given hand against board state.  
* \[ \] Implement generate\_legal\_moves(game\_state) \-\> Vec\<Action\>.

**User Story 2.2: Random Bot & Simulation Loop**

*As the simulator, I want bots to play headless games so I can test completion.*

* \[ \] Write test test\_headless\_random\_game asserting two random bots can finish 5 rounds.  
* \[ \] Implement Random Bot AI logic.

**User Story 2.3: Parallelization & Metric Logging**

*As the simulator, I want to run millions of games fast so I get valid data.*

* \[ \] Write test test\_rayon\_parallelization wrapping 1,000 games in rayon::par\_iter().  
* \[ \] Implement concurrent game loop.  
* \[ \] Implement CSV logging (Seat win rates, average turns, score variance).

**User Story 2.4: Strategic Bots**

*As the simulator, I want different AI profiles to discover optimal meta-strategies.*

* \[ \] Implement "Point-Averse Bot" (Aggressively pushes/discards high cards).  
* \[ \] Implement "Hoarder Bot" (Holds Wilds until late rounds).  
* \[ \] Run 100,000 iterations to compare profiles.

## **Phase 3: iOS SwiftUI App (Local Play)**

**User Story 3.1: Rust/Swift Bridge**

*As an iOS developer, I need to call my Rust engine from Swift so I don't have to rewrite the rules.*

* \[ \] Generate .xcframework using UniFFI (or Swift-Bridge).  
* \[ \] Write basic SwiftUI view calling Game.get\_deck\_size() and rendering text.

**User Story 3.2: Reactive UI & Board Rendering**

*As a player, I want to see the state of the game natively on my screen.*

* \[ \] Implement @Published state wrapping the Rust GameState.  
* \[ \] Build SwiftUI CardView changing dynamically based on Suit/Rank.  
* \[ \] Build SwiftUI BoardView laying out Sets and Runs.

**User Story 3.3: Drag-and-Drop Move Execution**

*As a player, I want to interact with cards naturally.*

* \[ \] Implement SwiftUI .transferable for dragging cards.  
* \[ \] Map drop intents back to Action::PlayMeld or Action::HitMeld.  
* \[ \] Ensure UI snaps back gracefully if Rust engine returns GameError.

## **Phase 4: Multiplayer Server**

**User Story 4.1: Server Setup & Handshake**

*As a remote player, I want to connect to a cloud server so I can play with friends.*

* \[ \] Setup Rust Axum (or Actix-Web) server on EC2.  
* \[ \] Implement WebSocket endpoint and establish handshake from iOS client.

**User Story 4.2: Matchmaking & Game Instantiation**

*As a remote player, I want to join specific rooms.*

* \[ \] Implement 4-digit room code generation logic.  
* \[ \] Implement lobby state allocating new instances of the GameState engine per room.

**User Story 4.3: State Broadcast**

*As a remote player, I want to see my opponent's moves instantly.*

* \[ \] Serialize iOS drag-and-drop actions into JSON over WS.  
* \[ \] Deserialize on Server, process via Rust Engine, and broadcast new serialized state to all clients in the room.

## **Final Milestone: CI/CD Pipeline**

**User Story 5.1: Automated Testing Enforcement**

*As the lead developer, I want to prevent bad code from reaching the main branch.*

* \[ \] Create GitHub Actions YAML for Rust toolchain.  
* \[ \] Configure branch protection rules requiring 100% test pass rate for merges.