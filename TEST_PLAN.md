# Test plan — PushSimulator

## Tier 1: Fast feedback

```bash
cargo test -p push_core
```

Optional full workspace:

```bash
cargo test
```

---

## Integration chain (1..X together)

Headless tests in `push_core` that run shipped behaviors **together** through real engine APIs. Rule: [.cursor/rules/integration-chain.mdc](.cursor/rules/integration-chain.mdc).

Current chain through shuffle and draw: Suit → Rank → Card → `Deck::new` → `is_wild` → `shuffle` → `draw`, named `test_suit_rank_card_deck_new_is_wild_shuffle_draw` (`push_core/tests/suit_rank_card_deck.rs`). It locks 12 wilds through a shuffle (same cards, new order) and a draw down to empty. Earlier chains stay: `test_suit_rank_card_deck_new_is_wild_twelve_wilds` and `test_suit_rank_card_deck_new_two_decks_two_jokers_each`.

Player initialization continues that chain in the same file. Each shipped draw outcome then calls `Player::new`, and the new hand stays empty: ordinary draw (`test_suit_rank_card_deck_new_is_wild_shuffle_draw_player_new`), deck and 12 wilds with no draw (`test_suit_rank_card_deck_new_is_wild_player_new`), both piles empty, a discard that stays put, a three-or-more reshuffle, one draw-pile card then the last discard card, two ordinary draw-pile cards, the last leftover card, and the last two leftover cards.

Dealing continues that chain in the same file. `test_deal_initial_hand` deals one card at a time to two players so each hand has 10 cards and the deck is 20 cards smaller. `test_deal_initial_hand_three_players_one_card_at_a_time` continues that rotation to a third player. `test_deal_initial_hand_refuses_one_player` rejects a one-player deal and leaves the deck and that hand unchanged. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_initial_hand` and `test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands` run that same rotation after shuffle. Later draws do not put cards into those hands: an ordinary draw, a discard that stays put, a three-or-more reshuffle, one draw-pile card then the last discard, two ordinary pile cards, the last leftover card, and the last two leftover cards.

Pip scoring continues that chain. `test_score_card_pip` returns 5 for ranks 3 through 9. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_pip_penalty` and `test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_pip_penalty` score those ranks after the deal. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_pip_penalty` scores them after a later draw. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_pip_penalty` scores a pip that stayed on the discard pile. Player points stay 0.

Face and ace scoring continues that chain. `test_score_card_face` returns 10 for a 10, Jack, Queen, and King. `test_score_card_ace` returns 15 for an ace. Suit, id, and lock stay put. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_face_ace_penalty` and `test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_face_ace_penalty` score those ranks after the deal. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_face_ace_penalty` scores them after a later draw. `test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_face_ace_penalty` scores a king and an ace that stayed on the discard pile. Every 10 through King in play scores 10 (32 cards). Every ace scores 15 (8 cards). Pips still score 5. Player points stay 0. A two or a joker still panics.

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
