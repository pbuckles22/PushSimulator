//! Chain: undealt shoe → exhibit snapshot → `PlayMeld` refusal → `HitMeld`
//! accept → a later refusal. `GameError` is an actor the table does not have.
//! The shoe stays 108. The opponent hand stays off the snapshot.

use push_core::actions::{Action, MeldHit};
use push_core::card::{Card, Rank, Suit};
use push_core::deck::Deck;
use push_core::game_state::GameState;
use push_core::player::Player;
use push_ffi::{apply_drop, snapshot_from_state, DropVerdict, Game, HitRequest};

#[test]
fn test_exhibit_shoe_play_meld_refuses_the_king_and_hit_meld_takes_the_two() {
    let fresh = Game::new();
    let exhibit = Game::exhibit_set_and_run();
    let before = exhibit.table_snapshot();
    assert_eq!(exhibit.get_deck_size(), 108);
    assert_eq!(before.deck_size, 108);
    assert_eq!(before.round_number, 2);
    assert_eq!(before.hand.len(), 4);
    assert!(before.hand.iter().all(|card| card.id != 90));
    assert!(before
        .board
        .iter()
        .flat_map(|meld| &meld.cards)
        .all(|card| card.id != 90));
    assert_eq!(exhibit.public_version(), fresh.public_version());

    let king = before.hand[1].clone();
    assert_eq!(
        exhibit.apply_play_meld(vec![vec![king.clone()]]),
        DropVerdict::Refused
    );
    let joker = before.hand[2].clone();
    assert_eq!(
        exhibit.apply_play_meld(vec![vec![joker.clone()]]),
        DropVerdict::Refused
    );
    assert_eq!(
        exhibit.apply_hit_meld(vec![HitRequest {
            meld_index: 0,
            cards: vec![joker],
        }]),
        DropVerdict::Refused
    );
    let eight = before.hand[0].clone();
    assert_eq!(
        exhibit.apply_hit_meld(vec![HitRequest {
            meld_index: 0,
            cards: vec![eight],
        }]),
        DropVerdict::Refused
    );
    let mut forged = king.clone();
    forged.rank = push_ffi::CardRank::Ace;
    assert_eq!(
        exhibit.apply_play_meld(vec![vec![forged]]),
        DropVerdict::Refused
    );
    assert_eq!(exhibit.apply_play_meld(vec![]), DropVerdict::Refused);
    assert_eq!(
        exhibit.apply_hit_meld(vec![HitRequest {
            meld_index: 9,
            cards: vec![king],
        }]),
        DropVerdict::Refused
    );
    assert_eq!(exhibit.table_snapshot(), before);
    assert_eq!(exhibit.get_deck_size(), 108);
    assert!(fresh.table_snapshot().board.is_empty());
    assert_eq!(fresh.get_deck_size(), 108);

    let two = before.hand[3].clone();
    assert_eq!(
        exhibit.apply_hit_meld(vec![HitRequest {
            meld_index: 0,
            cards: vec![two.clone()],
        }]),
        DropVerdict::Accepted
    );
    let after = exhibit.table_snapshot();
    assert_eq!(after.deck_size, 108);
    assert_eq!(exhibit.get_deck_size(), 108);
    assert_eq!(after.round_number, 2);
    assert_eq!(exhibit.public_version(), fresh.public_version());
    assert_eq!(after.hand.len(), 3);
    assert!(after.hand.iter().all(|card| card.id != two.id));
    assert_eq!(after.hand[0].id, 21);
    assert_eq!(after.hand[1].id, 22);
    assert_eq!(after.hand[2].id, 23);
    assert_eq!(after.hand[2].locked_until_turn, 2);
    assert_eq!(after.board[0].cards.len(), 4);
    assert_eq!(after.board[0].cards[3].id, two.id);
    assert_eq!(after.board[0].cards[3].rank, push_ffi::CardRank::Two);
    assert_eq!(after.board[1], before.board[1]);
    assert!(after.hand.iter().all(|card| card.id != 90));
    assert!(after
        .board
        .iter()
        .flat_map(|meld| &meld.cards)
        .all(|card| card.id != 90));
    assert_eq!(
        exhibit.apply_hit_meld(vec![HitRequest {
            meld_index: 0,
            cards: vec![two],
        }]),
        DropVerdict::Refused
    );
    assert_eq!(exhibit.table_snapshot(), after);
    assert_eq!(exhibit.get_deck_size(), 108);
    assert_eq!(fresh.table_snapshot().board.len(), 0);
    assert_eq!(fresh.get_deck_size(), 108);
}

#[test]
fn test_empty_game_play_and_hit_refuse_and_the_shoe_stays_108() {
    let game = Game::new();
    let before = game.table_snapshot();
    assert_eq!(before.deck_size, 108);
    assert!(before.board.is_empty());
    assert!(before.hand.is_empty());
    let king = push_ffi::CardSnapshot {
        id: 22,
        suit: push_ffi::CardSuit::Spades,
        rank: push_ffi::CardRank::King,
        locked_until_turn: 0,
    };
    assert_eq!(
        game.apply_play_meld(vec![vec![king.clone()]]),
        DropVerdict::Refused
    );
    assert_eq!(
        game.apply_hit_meld(vec![HitRequest {
            meld_index: 0,
            cards: vec![king],
        }]),
        DropVerdict::Refused
    );
    assert_eq!(game.table_snapshot(), before);
    assert_eq!(game.get_deck_size(), 108);
}

/// Chain: a legal `PlayMeld` through the same drop the exhibit uses, then an
/// actor the table does not have. That actor is `GameError`. Scores stay 0.
#[test]
fn test_play_meld_accept_then_game_error_leaves_the_table() {
    let mut deck = Deck::new();
    let set = vec![
        take(&mut deck, Suit::Hearts, Rank::Four),
        take(&mut deck, Suit::Spades, Rank::Four),
        take(&mut deck, Suit::Clubs, Rank::Four),
    ];
    let keeper = take(&mut deck, Suit::Diamonds, Rank::King);
    let mut state = GameState::new(vec![Player::new(0, 0), Player::new(1, 1)], deck);
    state.round_number = 2;
    state.players[0].is_on_board = true;
    state.players[0].hand = set.clone();
    state.players[0].hand.push(keeper);
    state.players[0].points = 4;
    state.players[0].total_score = 9;
    state.players[1].hand = vec![Card {
        id: 90,
        suit: Suit::Hearts,
        rank: Rank::Ace,
        locked_until_turn: 0,
    }];
    let before = snapshot_from_state(&state);
    assert!(before.hand.iter().all(|card| card.id != 90));

    let mut missed = state.clone();
    assert_eq!(
        apply_drop(&mut missed, 2, Action::PlayMeld(vec![set.clone()])),
        DropVerdict::GameError
    );
    assert_eq!(missed, state);

    assert_eq!(
        apply_drop(&mut state, 0, Action::PlayMeld(vec![set.clone()])),
        DropVerdict::Accepted
    );
    let snap = snapshot_from_state(&state);
    assert_eq!(snap.board.len(), 1);
    assert_eq!(snap.board[0].cards.len(), 3);
    assert_eq!(snap.hand.len(), 1);
    assert_eq!(snap.deck_size, before.deck_size);
    assert!(snap
        .hand
        .iter()
        .chain(snap.board.iter().flat_map(|meld| &meld.cards))
        .all(|card| card.id != 90));
    assert_eq!(state.players[0].points, 4);
    assert_eq!(state.players[0].total_score, 9);
    assert!(!state.round_over);

    let laid = state.clone();
    assert_eq!(
        apply_drop(
            &mut state,
            3,
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![set[0]],
            }])
        ),
        DropVerdict::GameError
    );
    assert_eq!(state, laid);
}

fn take(deck: &mut Deck, suit: Suit, rank: Rank) -> Card {
    let index = deck
        .cards
        .iter()
        .position(|card| card.suit == suit && card.rank == rank)
        .expect("card");
    deck.cards.remove(index)
}
