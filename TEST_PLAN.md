# Test plan — PushSimulator

## Tier 1: Fast feedback

```bash
cargo test -p push_core
```

GitHub Actions runs that same command (`.github/workflows/push-core.yml`) on push and pull request.

Optional full workspace:

```bash
cargo test
```

---

## Integration chain (1..X together)

Headless tests in `push_core` that run shipped behaviors **together** through real engine APIs. Rule: [.cursor/rules/integration-chain.mdc](.cursor/rules/integration-chain.mdc).

Current chain through shuffle and draw: Suit → Rank → Card → `Deck::new` → `is_wild` → `shuffle` → `draw`, named `test_suit_rank_card_deck_new_is_wild_shuffle_draw` (`push_core/tests/suit_rank_card_deck.rs`). It locks 12 wilds through a shuffle (same cards, new order) and a draw down to empty. Earlier chains stay: `test_suit_rank_card_deck_new_is_wild_twelve_wilds` and `test_suit_rank_card_deck_new_two_decks_two_jokers_each`.

Player initialization continues that chain in the same file. One chain, `test_suit_rank_card_deck_new_is_wild_shuffle_draw_player_new`, shuffles, draws one card, then calls `Player::new`. That hand stays empty and does not take the drawn card.

Dealing continues that chain in the same file. `test_deal_initial_hand` deals one card at a time to two players so each hand has 10 cards and the deck is 20 cards smaller. `test_deal_initial_hand_short_draw_pile` leaves three draw-pile cards, deals those three, and then panics. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_short_draw_pile` does that after a shuffle. The dealt ids are the three that remained. `test_deal_initial_hand_three_players_one_card_at_a_time` continues that rotation to a third player. `test_deal_initial_hand_refuses_one_player` rejects a one-player deal and leaves the deck and that hand unchanged. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_initial_hand` and `test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands` run that same rotation after shuffle. Later draws do not put cards into those hands: an ordinary draw, a discard that stays put, a three-or-more reshuffle, one draw-pile card then the last discard, two ordinary pile cards, the last leftover card, and the last two leftover cards.

Pip scoring continues that chain. `test_score_card_pip` returns 5 for ranks 3 through 9. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_pip_penalty` and `test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_pip_penalty` score those ranks after the deal. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_pip_penalty` scores them after a later draw. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_pip_penalty` scores a pip that stayed on the discard pile. Player points stay 0.

Face and ace scoring continues that chain. `test_score_card_face` returns 10 for a 10, Jack, Queen, and King. `test_score_card_ace` returns 15 for an ace. Suit, id, and lock stay put. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_face_ace_penalty` and `test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_face_ace_penalty` score those ranks after the deal. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_face_ace_penalty` scores them after a later draw. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_face_ace_penalty` scores a king and an ace that stayed on the discard pile. Every 10 through King in play scores 10 (32 cards). Every ace scores 15 (8 cards). Pips still score 5. Player points stay 0.

Wild scoring continues that chain. `test_score_card_wild` returns 20 for a two and a joker. Suit, id, and lock stay put. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_wild_penalty` and `test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_wild_penalty` score those ranks after the deal. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_two_penalty` and `test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_joker_penalty` score a drawn two and a drawn joker. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_wild_penalty` scores a two and a joker left on the discard pile. Every two scores 20 (8 cards). Every joker scores 20 (4 cards). Pips still score 5. A 10 through King still scores 10. An ace still scores 15. Player points stay 0.

Hand total continues that chain. `test_calculate_hand_total` sums a hand of 4, Jack, Ace, and Joker to 50. `test_add_to_total_score` adds that 50 onto `total_score` and leaves `points` at 0. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_hand_total` and `test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_hand_total` sum each dealt hand. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_hand_total` keeps the drawn card out of those totals. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_hand_total` keeps a two and a joker on the discard pile out of those totals. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_add_to_total_score` adds one player's hand penalty onto that player's `total_score` only.

Taking the discard continues that chain. `test_take_discard` moves the discard pile's top card onto the end of the player's hand and leaves the card under it. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_take_top` deals two players, leaves a discard, and takes the top ace into player 0's hand. That hand grows from 10 cards to 11 and its penalty rises by 15. The other hand stays the dealt 10. The draw pile stays. `points` and `total_score` stay 0.

Pushing a discard continues that chain. `test_push_discard_moves_card` moves the top discard onto Player 2's hand. `test_push_discard_penalty_draw` also gives Player 2 the next draw-pile card. `test_push_turn_advancement` then gives Player 1 the card after that. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_push_top` deals two players, leaves a discard, and pushes the top ace: Player 2's hand grows from 10 to 12 and that penalty rises by 20 (15 for the ace, 5 for the four). Player 1's hand grows from 10 to 11 and that penalty rises by 10. Cards under the ace stay. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_take_top_then_push` takes the ace, then pushes the card that was under it. `points` and `total_score` stay 0.

Validating a set continues that chain. `test_validate_set_naturals` accepts 4♥ 4♠ 4♣ and rejects 4♥ 4♠ 5♣. Two cards are not a set. Four of a kind is. `test_validate_set_wilds` accepts 4♥, a joker, and 2♠, and three jokers. A four, a five, and a joker are not a set. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_push_top_validate_set` deals two players, leaves a discard, and pushes the top ace to Player 2 with a four. Player 1 draws a jack. That ace, four, and jack are not a set. Three fours from that shoe are. A four, a joker, and a two are. Hands and piles stay as the push left them. `points` and `total_score` stay 0.

Validating a run continues that chain. `test_validate_run_naturals` accepts 4♥ 5♥ 6♥ 7♥ and rejects a mixed suit, a run of three, a gap, and a repeated rank. Five and seven of one suit are runs. Order does not matter. `test_validate_run_wilds` accepts 4♥, a joker, 6♥, and 2♠. One wild does not fill two holes. Two wilds can. A wild does not repair a natural of the wrong suit. Four jokers are a run. Three jokers are not. `test_ace_placement_low` accepts A♥, 2♣, 3♥, 4♥. Ace, 3, 4, 5 with no wild in the deuce place is not a run. `test_ace_placement_high` accepts J♥ Q♥ K♥ A♥. `test_ace_wrap_rejection` rejects K♥ A♥, a two, and 3♥. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_push_top_validate_set_validate_run` pushes, then checks a set and those run results on cards from that shoe. Hands and piles stay as the push left them.

Seeded shuffle continues that chain. `Deck::shuffle_with` and `Deck::draw_with` take the rng, so an order check does not use `thread_rng`. `test_shuffle_with_seed_repeats_order` repeats one seed and changes order for another. `test_card_ids_survive_seeded_shuffle_draw_and_reshuffle` keeps every card id through a seeded shuffle, a full draw, and a reshuffle of three or more, for seeds 0 through 23. An empty discard stays empty. One leftover card goes to the current player. `test_suit_rank_card_deck_new_is_wild_seeded_shuffle_deal_draw_reshuffle_push_validate_set_validate_run` deals, drains, reshuffles three or more, pushes, then checks a set and a run on that shoe. The same ids are still in play.

Round requirements continue that chain. `test_round_1_minimum_rejection` rejects one set of 3, and one set of 4, in round 1. `test_round_1_minimum_acceptance` accepts two sets of 3. `test_round_1_exceeding_minimum` accepts two sets of 4, and a set of 5 beside a set of 3. A third set fails. Two sets plus a run fail round 1. Six fours split into two sets of three pass. `test_round_1_six_fours_need_two_melds` rejects those six in one meld, in either order of the two melds. A group that is not a set fails. A pair, an empty meld, and the same id twice inside one meld fail. The same card in a set and a run fails. Wilds count when `validate_set` accepts them. Four wilds fill one requirement: two such groups pass rounds 1, 2, and 3, three pass round 4, and a third group fails rounds 1 and 3. Four wilds are not a round 5 run. Round 2 is one set of at least 3 and one run of at least 4. The run may come first. Two runs do not pass round 2. A gap fails. An extra four-wild group fails. Round 3 is two runs of at least 4. Eight cards in one run do not count as two. A run of 3 does not. Ace-low beside ace-high passes. A wrap fails. A shared card fails. Round 4 is three sets of at least 3. A run does not fill one of those sets. One set of 9 does not count as three. Order does not matter. Round 5 is one set of at least 3 and one run of at least 7. The run may be ace-low, ace-high, or unsorted. A set of 7 beside another set does not count as the run. Two runs of 7 do not pass. A wrap or a mixed suit fails. Two groups of seven wilds pass. A third fails. A layout that passes round 5 fails rounds 0 and 6. `test_validate_run_two_starts_low_and_length_bounds` accepts 2♥ 3♥ 4♥ 5♥, rejects one wild trying to fill two holes, accepts a king-ace extended by two wilds, accepts a 13-card suit, and rejects 14 cards. `test_suit_rank_card_deck_new_is_wild_seeded_shuffle_deal_draw_reshuffle_push_validate_set_validate_run_round_requirements` deals, drains, reshuffles, pushes, checks a set and a run on that shoe, then applies those round results. Six fours from that shoe are two sets, and one meld of those six fails. Two copies of the five of hearts may sit in one set. A card shared by two melds fails. Two runs from that shoe are round 3. Three sets are round 4. A fourth set fails. Jack through ace of hearts meets round 2. Hands and piles stay as the push left them.

Getting on the board continues that chain. `test_play_meld_success` moves two sets of 3 from the hand onto `GameState.board` and leaves the other cards in that hand, in order. A second copy of the same four stays in the hand. A lookalike four in the draw pile stays there. `test_play_meld_flags_player` sets that player's `is_on_board` and leaves the other player off. A second play is refused. `test_play_meld_refuses_and_leaves_the_table` refuses one set, a third set, a pair, an empty meld, a repeated id, a card used twice, a draw-pile card, a discard card, the other player's card, a forged lock, a run in round 1, one group of four wilds, one meld of six fours, rounds 0 and 6, and a player who is already on the board. The hand, the board, the piles, and both scores stay as they were. `test_play_meld_rounds_2_through_5` lays down rounds 2 through 5, including an unsorted run, an ace-low run, four wilds, and two groups of seven wilds. One run of 8 does not meet round 3. A run does not fill a set in round 4. A run of 4 does not meet round 5. `test_play_meld_six_fours_need_two_melds` rejects those six as one meld and accepts them as two. `test_play_meld_second_player_appends` adds the second player's melds after the first. One player cannot lay down the other's cards. `test_play_meld_can_empty_the_hand` leaves an empty hand, `points` at 4, and `total_score` at 9. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_push_round_requirements_play_meld` shuffles, deals, pushes, then plays two sets from that shoe. One set is refused. A set plus a run is refused in round 1. The run stays in the hand. The other seat and both piles stay put. Every card id is still on the table.

Empty-draw reshuffle continues that chain in the same file. Three or more discard cards leave the top card, then the recycled pile is drawn. `test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_last_card_goes_to_current` gives the last discard card to the current player. `test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_last_two_to_current_and_next` shuffles the last two and splits them. `test_suit_rank_card_deck_new_is_wild_shuffle_draw_one_then_last_discard_card` draws a remaining draw-pile card before that last discard card. `test_suit_rank_card_deck_new_is_wild_shuffle_draw_two_pile_cards_stay_ordinary` keeps two draw-pile cards as two ordinary draws.

Suggested after Epic 1.8: Epic 1.9 chains (see [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md)). Until then, Crucible scenarios in BACKLOG Epic 1.8 are the later multi-step suites.

```bash
cargo test -p push_core
```

---

## Tier 2: Integration / E2E

| Surface | When | Command / check |
|---------|------|-----------------|
| WASM viewer | Phase 1b | Build with `wasm-pack`, serve `viewer/`, smoke Load WASM + bot play |
| iOS | Phase 3 | Xcode + UniFFI smoke |
| Multiplayer | Phase 4 | WS handshake + room round-trip |

```bash
wasm-pack build push_wasm --target web --out-dir ../viewer/pkg
```

---

**Handoff:** Commands mirrored in [AGENT_HANDOFF.md](AGENT_HANDOFF.md).

**Compile counter:** Count every merge-ready compile (test/Debug and Release). Local `build_number.txt` (gitignored).

**Evidence sink:** TBD when a durable runtime log exists. Until then, CI/`cargo test` is the gate. Optional viewer hook: `viewer/test_status.json` for live red/green panel.
