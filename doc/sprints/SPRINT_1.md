# **Sprint 1 Backlog: Core Engine Foundation**

This backlog translates the Project Management Plan into ultra-granular, sequential tasks. The AI Agent must execute these in exact order using a strict Red/Green/Refactor TDD cycle.

* **Red:** Write the test listed on the line. Run it. Watch it fail.  
* **Green:** Write the absolute minimum code to pass that specific test.  
* **Refactor:** Clean up the code. Run all previous tests to ensure no regressions. Move to the next line.

## **Epic 1.1: Primitives & Deck Management**

**Status:** complete — 2026-10-06

### **User Story 1.1.1: Card Definitions**

*As the game engine, I want to define a playing card so that I have a foundational object for all game mechanics.*

* \[x\] Write test test\_suit\_enum\_instantiation (Hearts, Diamonds, Clubs, Spades, None).  
* \[x\] Implement Suit enum to pass test.  
* \[x\] Write test test\_rank\_enum\_instantiation (Two through Ace, Joker).  
* \[x\] Implement Rank enum to pass test.  
* \[x\] Write test test\_card\_struct\_instantiation asserting a Card requires an id, suit, rank, and locked\_until\_turn.  
* \[x\] Implement Card struct matching the Data Schema to pass test.

### **User Story 1.1.2: Wild Card Identification**

*As the game engine, I want cards to know if they are wild so that I can apply special validation and scoring rules to them later.*

* \[x\] Write test test\_card\_is\_wild\_true\_for\_joker.  
* \[x\] Implement is\_wild() method returning true for Jokers to pass test.  
* \[x\] Write test test\_card\_is\_wild\_true\_for\_two.  
* \[x\] Update is\_wild() method to also return true for Twos to pass test.  
* \[x\] Write test test\_card\_is\_wild\_false\_for\_standard\_card (e.g., 3 of Hearts).  
* \[x\] Ensure is\_wild() method correctly returns false to pass test.

### **User Story 1.1.3: Deck Generation**

*As the game engine, I want to generate a standard "Push" deck so that players have the correct 108 cards to play with.*

* \[x\] Write test test\_deck\_instantiation\_count asserting a new deck has exactly 108 cards.  
* \[x\] Implement Deck struct and Deck::new() method with nested loops (2 decks x 52 cards \+ 4 Jokers) to pass test.  
* \[x\] Write test test\_deck\_contains\_exact\_wild\_count asserting the new deck contains exactly 8 Twos and 4 Jokers.  
* \[x\] Refactor Deck::new() if necessary to pass test.

### **User Story 1.1.4: Shuffling & Drawing**

*As the game engine, I want to shuffle and draw from the deck so that gameplay is randomized and state advances.*

* \[x\] Write test test\_deck\_shuffle asserting the order of cards in deck.cards changes after shuffling.  
* \[x\] Add rand crate to Cargo.toml.  
* \[x\] Implement deck.shuffle() using a random number generator to pass test.  
* \[x\] Write test test\_deck\_draw\_reduces\_count asserting drawing 1 card reduces deck size from 108 to 107\.  
* \[x\] Implement deck.draw() returning Option\<Card\> to pass test.  
* \[x\] Write test test\_deck\_draw\_empty asserting drawing from an empty deck returns nothing when both piles are empty.  
* \[x\] Ensure deck.draw() handles empty vectors gracefully to pass test.  
* \[x\] One leftover discard card goes to the current player. Two leftover cards are shuffled and split between the current player and the next. Three or more still leave the top card (`doc/requirements/GAME_RULES.md`).

## **Epic 1.2: Player State & Scoring**

**Status:** complete — 2026-10-06. A player starts empty. Two or more players are dealt 10 cards, one at a time. Ranks 3–9 score 5, a 10 through King scores 10, an ace scores 15, and a two or a joker scores 20. A hand of 4, Jack, Ace, and Joker totals 50, and that penalty adds onto `total_score`. **1.2.4.2** and **1.2.4.3** stay deferred. **1.3.1** is parked. Next is **1.3.2**, only when asked.

### **User Story 1.2.1: Player Initialization**

*As the game engine, I want to create a Player so that I can track their hand, board status, and score.*

* \[x\] Write test test\_player\_instantiation asserting a new player starts with 0 points and is\_on\_board \= false.  
* \[x\] Implement Player struct matching the Data Schema to pass test.  
* \[x\] Write test test\_player\_hand\_starts\_empty asserting a new player has 0 cards.  
* \[x\] Implement Player::new(id, seat\_index) to pass test.

### **User Story 1.2.2: Dealing Hands**

*As the game engine, I want to deal cards to a player so that they have a starting hand.*

* \[x\] Write test test\_deal\_initial\_hand asserting two or more players each receive exactly 10 cards, one card at a time, and the deck shrinks by 10 per player.  
* \[x\] Implement deal\_initial\_hands. One player is refused. The discard pile stays put.

### **User Story 1.2.3: Pip Card Scoring (3-9)**

*As the game engine, I want to calculate the value of pip cards so that penalties can be tallied at the end of a round.*

* \[x\] Write test test\_score\_card\_pip passing a 3, 5, and 9, asserting each returns 5 points.  
* \[x\] Implement card.get\_penalty\_value() returning 5 for ranks 3 through 9 to pass test.

### **User Story 1.2.4: Face Card & Ace Scoring**

*As the game engine, I want to calculate the value of high cards so that appropriate penalties are applied.*

**1.2.4.1** is the scoring below, parked on `feature/1.2.4-face-ace-scoring`. **1.2.4.2** and **1.2.4.3** are the 2026-10-06 test review. They are not the next red/green.

* \[x\] Write test test\_score\_card\_face passing a 10, Jack, Queen, and King, asserting each returns 10 points.  
* \[x\] Update card.get\_penalty\_value() to handle face cards to pass test.  
* \[x\] Write test test\_score\_card\_ace passing an Ace, asserting it returns 15 points.  
* \[x\] Update card.get\_penalty\_value() to handle Aces to pass test.

**1.2.4.2: Tech debt**

* \[ \] Seed `Deck::shuffle` in tests, or inject the rng, so the order check does not depend on `thread_rng` (`TECH_DEBT.md`).  
* \[ \] Remove the uncalled arms, or make them real behavior: `reshuffle_discard` on an empty discard, `reshuffle_discard` putting a single card back, and `TurnDraw::Empty` after a reshuffle. `draw` does not reach those paths.  
* \[ \] Keep one chain test that `Player::new` does not take a drawn card. The other "player starts empty" chains repeat that.  
* \[ \] Add CI that runs `cargo test -p push_core`. The 2026-10-06 LLVM run (58 tests, 98.1% lines) stays a local measurement until a coverage gate is chosen.  
* \[ \] Name the evidence sink in `TEST_PLAN.md`, or leave one explicit TBD.  
* \[ \] Drop the unused `serde` dependency on `push_core`, or use it (`TECH_DEBT.md`).

**1.2.4.3: Test gaps**

* \[ \] Write a test that `deal_initial_hands` runs out of draw-pile cards. The panic is the same line as a successful pop, so `player.rs` at 100% line coverage does not cover a short deck.  
* \[ \] Do not add a `should_panic` test for `get_penalty_value` on a Two or a joker. **1.2.5** scores those 20. Hand total is **1.2.6**. Face cards score 10 and an ace scores 15.  
* \[ \] Add a property test that the same card ids survive shuffle, draw, and a three-or-more reshuffle. Fixed scenarios already check this.  
* \[ \] Doc-tests are 0. Add them only when a public function's rustdoc example is the lock.  
* \[ \] `push_wasm` and `push_sim` have no tests. Tier 2 (viewer, iOS, multiplayer) stays in `TEST_PLAN.md` for later phases. This item does not start those.

### **User Story 1.2.5: Wild Card Scoring**

*As the game engine, I want to calculate the massive penalty for wild cards so that players are properly punished for hoarding them.*

* \[x\] Write test test\_score\_card\_wild passing a Two and a Joker, asserting each returns 20 points.  
* \[x\] Update card.get\_penalty\_value() to handle Twos and Jokers to pass test.

### **User Story 1.2.6: Total Hand Calculation**

*As the game engine, I want to sum a player's hand so that I can add it to their historical score.*

* \[x\] Write test test\_calculate\_hand\_total passing a hand of \[4, Jack, Ace, Joker\]. Assert total is 50 (5 \+ 10 \+ 15 \+ 20).  
* \[x\] Implement player.calculate\_hand\_penalty() to iterate over the hand and sum the values to pass test.  
* \[x\] Write test test\_add\_to\_total\_score asserting a penalty of 50 correctly updates player.total\_score from 0 to 50\.  
* \[x\] Implement score update logic to pass test.

## **Epic 1.3: The "Push" & Draw Phase**

### **User Story 1.3.1: Standard Draw**

**Status:** parked — 2026-10-06 on `feature/1.3.1-take-discard` (not merged).

*As the game engine, I want a player to take a discard so they can add it to their hand.*

* \[x\] Write test test\_take\_discard asserting the discard pile's top card moves to the player's hand.  
* \[x\] Implement Action::TakeDiscard handler. Next is **1.3.2**, only when asked.