//! Chain: undealt shoe → `Game::get_deck_size` → `Game::table_snapshot`
//! → `PlayMeld` of a set and a run → `snapshot_from_state` → exhibit board.
//! The snapshot hand is the local seat. The opponent hand stays off.

use push_core::actions::Action;
use push_core::card::{Card, Rank, Suit};
use push_core::deck::Deck;
use push_core::game_state::GameState;
use push_core::player::Player;
use push_ffi::{snapshot_from_state, CardRank, CardSuit, Game};

#[test]
fn test_deck_new_uniffi_game_get_deck_size_table_snapshot_empty_board() {
    let game = Game::new();
    let snap = game.table_snapshot();
    assert_eq!(game.get_deck_size(), 108);
    assert_eq!(game.public_version(), "0.3.9");
    assert_eq!(snap.deck_size, 108);
    assert_eq!(snap.round_number, 1);
    assert!(snap.board.is_empty());
    assert!(snap.hand.is_empty());
    assert_eq!(game.table_snapshot(), snap);
    assert_eq!(game.get_deck_size(), 108);

    let state = GameState::new(vec![Player::new(0, 0), Player::new(1, 1)], Deck::new());
    assert_eq!(snapshot_from_state(&state), snap);
}

#[test]
fn test_exhibit_set_and_run_keeps_the_shoe_and_lays_two_melds() {
    let fresh = Game::new();
    let exhibit = Game::exhibit_set_and_run();
    assert_eq!(exhibit.get_deck_size(), 108);
    assert_eq!(exhibit.public_version(), fresh.public_version());
    let snap = exhibit.table_snapshot();
    assert_eq!(snap.deck_size, 108);
    assert_eq!(snap.round_number, 2);
    assert_eq!(snap.board.len(), 2);
    assert_eq!(snap.board[0].cards.len(), 3);
    assert_eq!(snap.board[1].cards.len(), 4);
    assert_eq!(snap.hand.len(), 4);
    assert_face(
        &snap.board[0].cards[0],
        1,
        CardSuit::Hearts,
        CardRank::Four,
        0,
    );
    assert_face(
        &snap.board[0].cards[1],
        2,
        CardSuit::Spades,
        CardRank::Four,
        0,
    );
    assert_face(
        &snap.board[0].cards[2],
        3,
        CardSuit::Clubs,
        CardRank::Four,
        0,
    );
    assert_face(
        &snap.board[1].cards[0],
        10,
        CardSuit::Hearts,
        CardRank::Four,
        0,
    );
    assert_face(
        &snap.board[1].cards[1],
        11,
        CardSuit::Hearts,
        CardRank::Five,
        0,
    );
    assert_face(
        &snap.board[1].cards[2],
        12,
        CardSuit::Hearts,
        CardRank::Six,
        0,
    );
    assert_face(
        &snap.board[1].cards[3],
        13,
        CardSuit::Hearts,
        CardRank::Seven,
        0,
    );
    assert_face(&snap.hand[0], 21, CardSuit::Diamonds, CardRank::Eight, 0);
    assert_face(&snap.hand[1], 22, CardSuit::Spades, CardRank::King, 0);
    assert_face(&snap.hand[2], 23, CardSuit::None, CardRank::Joker, 2);
    assert_face(&snap.hand[3], 24, CardSuit::Clubs, CardRank::Two, 0);
    let board_ids = ids(&snap);
    assert!(snap.hand.iter().all(|card| !board_ids.contains(&card.id)));
    assert_eq!(exhibit.table_snapshot(), snap);
    assert!(fresh.table_snapshot().board.is_empty());
    assert_ne!(fresh.table_snapshot(), snap);
    assert_eq!(fresh.get_deck_size(), 108);
}

#[test]
fn test_deck_new_play_meld_set_and_run_snapshot_matches_the_exhibit_faces() {
    let mut deck = Deck::new();
    let set = vec![
        take(&mut deck, Suit::Hearts, Rank::Four),
        take(&mut deck, Suit::Spades, Rank::Four),
        take(&mut deck, Suit::Clubs, Rank::Four),
    ];
    let run = vec![
        take(&mut deck, Suit::Hearts, Rank::Four),
        take(&mut deck, Suit::Hearts, Rank::Five),
        take(&mut deck, Suit::Hearts, Rank::Six),
        take(&mut deck, Suit::Hearts, Rank::Seven),
    ];
    let king = take(&mut deck, Suit::Spades, Rank::King);
    let opponent_ace = take(&mut deck, Suit::Hearts, Rank::Ace);
    let mut state = table(deck, [&set[..], &run[..], &[king]].concat());
    state.players[1].hand = vec![opponent_ace];
    state.round_number = 2;
    let before = snapshot_from_state(&state);
    assert!(before.board.is_empty());
    assert_eq!(before.deck_size, 99);
    assert_meld(&before.hand, &[&set[..], &run[..], &[king]].concat());
    assert!(before.hand.iter().all(|card| card.id != opponent_ace.id));
    assert!(state.apply(Action::PlayMeld(vec![set.clone(), run.clone()]), 0));
    let snap = snapshot_from_state(&state);
    assert_eq!(snap.deck_size, 99);
    assert_eq!(snap.round_number, 2);
    assert_eq!(snap.board.len(), 2);
    assert_meld(&snap.board[0].cards, &set);
    assert_meld(&snap.board[1].cards, &run);
    assert!(snap
        .board
        .iter()
        .flat_map(|meld| &meld.cards)
        .all(|card| card.id != king.id));
    assert_meld(&snap.hand, &[king]);
    assert!(snap.hand.iter().all(|card| card.id != opponent_ace.id));
    assert!(snap
        .board
        .iter()
        .flat_map(|meld| &meld.cards)
        .all(|card| card.id != opponent_ace.id));
    let exhibit = Game::exhibit_set_and_run().table_snapshot();
    assert_eq!(faces(&snap), faces(&exhibit));
    assert_ne!(ids(&snap), ids(&exhibit));
    assert_eq!(Game::new().get_deck_size(), 108);
    assert_eq!(exhibit.deck_size, 108);
}

#[test]
fn test_play_meld_run_then_set_snapshot_keeps_that_order() {
    let mut deck = Deck::new();
    let run = vec![
        take(&mut deck, Suit::Hearts, Rank::Seven),
        take(&mut deck, Suit::Hearts, Rank::Six),
        take(&mut deck, Suit::Hearts, Rank::Five),
        take(&mut deck, Suit::Hearts, Rank::Four),
    ];
    let set = vec![
        take(&mut deck, Suit::Hearts, Rank::Four),
        take(&mut deck, Suit::Spades, Rank::Four),
        take(&mut deck, Suit::Clubs, Rank::Four),
    ];
    let mut state = table(deck, [&run[..], &set[..]].concat());
    state.round_number = 2;
    assert!(state.apply(Action::PlayMeld(vec![run.clone(), set.clone()]), 0));
    let snap = snapshot_from_state(&state);
    assert_meld(&snap.board[0].cards, &run);
    assert_meld(&snap.board[1].cards, &set);
    assert_eq!(snap.board[0].cards[0].rank, CardRank::Seven);
    assert_eq!(snap.board[0].cards[3].rank, CardRank::Four);
}

#[test]
fn test_refused_play_leaves_the_snapshot_board_empty() {
    let mut deck = Deck::new();
    let set = vec![
        take(&mut deck, Suit::Hearts, Rank::Four),
        take(&mut deck, Suit::Spades, Rank::Four),
        take(&mut deck, Suit::Clubs, Rank::Four),
    ];
    let run = vec![
        take(&mut deck, Suit::Hearts, Rank::Four),
        take(&mut deck, Suit::Hearts, Rank::Five),
        take(&mut deck, Suit::Hearts, Rank::Six),
        take(&mut deck, Suit::Hearts, Rank::Seven),
    ];
    let mut state = table(deck, [&set[..], &run[..]].concat());
    let offered = [&set[..], &run[..]].concat();
    let before = snapshot_from_state(&state);
    assert_meld(&before.hand, &offered);
    assert!(!state.apply(Action::PlayMeld(vec![set, run]), 0));
    assert_eq!(snapshot_from_state(&state), before);
    assert!(snapshot_from_state(&state).board.is_empty());
    assert_meld(&snapshot_from_state(&state).hand, &offered);
    assert_eq!(before.round_number, 1);
}

#[test]
fn test_play_meld_locked_four_and_jokers_keep_suit_rank_and_lock() {
    let mut deck = Deck::new();
    let mut locked = take(&mut deck, Suit::Hearts, Rank::Four);
    locked.locked_until_turn = 4;
    let set = vec![
        locked,
        take(&mut deck, Suit::Spades, Rank::Four),
        take(&mut deck, Suit::Clubs, Rank::Four),
    ];
    let jokers = vec![
        take_joker(&mut deck),
        take_joker(&mut deck),
        take_joker(&mut deck),
    ];
    let twos = vec![
        take(&mut deck, Suit::Hearts, Rank::Two),
        take(&mut deck, Suit::Diamonds, Rank::Two),
        take(&mut deck, Suit::Clubs, Rank::Two),
    ];
    let mut state = table(deck, [&set[..], &jokers[..], &twos[..]].concat());
    state.turn_counter = 4;
    assert!(state.apply(Action::PlayMeld(vec![set.clone(), jokers.clone()]), 0));
    let opened = snapshot_from_state(&state);
    assert_meld(&opened.board[0].cards, &set);
    assert_eq!(opened.board[0].cards[0].locked_until_turn, 4);
    assert_eq!(opened.board[0].cards[1].locked_until_turn, 0);
    assert_meld(&opened.board[1].cards, &jokers);
    assert!(opened.board[1]
        .cards
        .iter()
        .all(|card| card.suit == CardSuit::None && card.rank == CardRank::Joker));

    assert!(state.apply(Action::PlayMeld(vec![twos.clone()]), 0));
    let snap = snapshot_from_state(&state);
    assert_eq!(snap.board.len(), 3);
    assert_meld(&snap.board[2].cards, &twos);
    assert_eq!(snap.board[2].cards[0].suit, CardSuit::Hearts);
    assert_eq!(snap.board[2].cards[1].suit, CardSuit::Diamonds);
    assert_eq!(snap.board[2].cards[2].suit, CardSuit::Clubs);
    assert!(snap.board[2]
        .cards
        .iter()
        .all(|card| card.rank == CardRank::Two));
}

#[test]
fn test_snapshot_keeps_an_empty_meld_and_maps_every_rank() {
    let mut deck = Deck::new();
    let hearts: Vec<Card> = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ]
    .into_iter()
    .map(|rank| take(&mut deck, Suit::Hearts, rank))
    .collect();
    let diamond = take(&mut deck, Suit::Diamonds, Rank::Ace);
    let club = take(&mut deck, Suit::Clubs, Rank::Ace);
    let spade = take(&mut deck, Suit::Spades, Rank::Ace);
    let joker = take_joker(&mut deck);
    let mut state = GameState::new(vec![Player::new(0, 0), Player::new(1, 1)], deck);
    state.round_number = 0;
    state.board = vec![hearts.clone(), vec![], vec![diamond, club, spade, joker]];
    state.deck.cards.clear();
    let snap = snapshot_from_state(&state);
    assert_eq!(snap.deck_size, 0);
    assert_eq!(snap.round_number, 0);
    assert_eq!(snap.board.len(), 3);
    assert_meld(&snap.board[0].cards, &hearts);
    assert!(snap.board[1].cards.is_empty());
    assert_eq!(snap.board[2].cards[0].suit, CardSuit::Diamonds);
    assert_eq!(snap.board[2].cards[1].suit, CardSuit::Clubs);
    assert_eq!(snap.board[2].cards[2].suit, CardSuit::Spades);
    assert_eq!(snap.board[2].cards[3].suit, CardSuit::None);
    assert_eq!(snap.board[2].cards[3].rank, CardRank::Joker);
    let ranks = [
        CardRank::Two,
        CardRank::Three,
        CardRank::Four,
        CardRank::Five,
        CardRank::Six,
        CardRank::Seven,
        CardRank::Eight,
        CardRank::Nine,
        CardRank::Ten,
        CardRank::Jack,
        CardRank::Queen,
        CardRank::King,
        CardRank::Ace,
    ];
    for (card, rank) in snap.board[0].cards.iter().zip(ranks) {
        assert_eq!(card.suit, CardSuit::Hearts);
        assert_eq!(card.rank, rank);
        assert_eq!(card.locked_until_turn, 0);
    }
}

/// Chain: take → local hand and an opponent hand → snapshot.
/// The local cards stay in order. The opponent cards are not on the record.
#[test]
fn test_snapshot_copies_the_local_hand_and_leaves_the_opponent_hand_off() {
    let mut deck = Deck::new();
    let mut locked = take(&mut deck, Suit::Diamonds, Rank::Eight);
    locked.locked_until_turn = 2;
    let local = vec![
        locked,
        take(&mut deck, Suit::Spades, Rank::King),
        take_joker(&mut deck),
        take(&mut deck, Suit::Clubs, Rank::Two),
    ];
    let opponent = vec![
        take(&mut deck, Suit::Hearts, Rank::Ace),
        take(&mut deck, Suit::Clubs, Rank::Ace),
    ];
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = local.clone();
    players[1].hand = opponent.clone();
    let state = GameState::new(players, deck);
    let snap = snapshot_from_state(&state);
    assert!(snap.board.is_empty());
    assert_eq!(snap.deck_size, 102);
    assert_meld(&snap.hand, &local);
    assert_eq!(snap.hand[0].locked_until_turn, 2);
    assert_eq!(snap.hand[2].suit, CardSuit::None);
    assert_eq!(snap.hand[2].rank, CardRank::Joker);
    assert!(opponent
        .iter()
        .all(|card| !snap.hand.iter().any(|shown| shown.id == card.id)));
    assert!(opponent.iter().all(|card| !ids(&snap).contains(&card.id)));

    let mut only_opponent = Player::new(1, 1);
    only_opponent.hand = opponent;
    let hidden = snapshot_from_state(&GameState::new(vec![only_opponent], Deck::new()));
    assert!(hidden.hand.is_empty());
    assert_eq!(hidden.deck_size, 108);

    let none = snapshot_from_state(&GameState::new(vec![], Deck::new()));
    assert!(none.hand.is_empty());
    assert_eq!(none.deck_size, 108);
}

fn table(deck: Deck, hand: Vec<Card>) -> GameState {
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = hand;
    GameState::new(players, deck)
}

fn take(deck: &mut Deck, suit: Suit, rank: Rank) -> Card {
    let index = deck
        .cards
        .iter()
        .position(|card| card.suit == suit && card.rank == rank)
        .expect("the shoe contains this suit and rank");
    deck.cards.remove(index)
}

fn take_joker(deck: &mut Deck) -> Card {
    let index = deck
        .cards
        .iter()
        .position(|card| card.rank == Rank::Joker)
        .expect("the shoe contains a joker");
    deck.cards.remove(index)
}

fn assert_face(
    card: &push_ffi::CardSnapshot,
    id: u32,
    suit: CardSuit,
    rank: CardRank,
    locked_until_turn: u32,
) {
    assert_eq!(card.id, id);
    assert_eq!(card.suit, suit);
    assert_eq!(card.rank, rank);
    assert_eq!(card.locked_until_turn, locked_until_turn);
}

fn assert_meld(snap: &[push_ffi::CardSnapshot], cards: &[Card]) {
    assert_eq!(snap.len(), cards.len());
    for (shot, card) in snap.iter().zip(cards) {
        assert_eq!(shot.id, card.id);
        assert_eq!(shot.locked_until_turn, card.locked_until_turn);
        assert_eq!(shot.suit, suit_name(card.suit));
        assert_eq!(shot.rank, rank_name(card.rank));
    }
}

fn suit_name(suit: Suit) -> CardSuit {
    match suit {
        Suit::Hearts => CardSuit::Hearts,
        Suit::Diamonds => CardSuit::Diamonds,
        Suit::Clubs => CardSuit::Clubs,
        Suit::Spades => CardSuit::Spades,
        Suit::None => CardSuit::None,
    }
}

fn rank_name(rank: Rank) -> CardRank {
    match rank {
        Rank::Two => CardRank::Two,
        Rank::Three => CardRank::Three,
        Rank::Four => CardRank::Four,
        Rank::Five => CardRank::Five,
        Rank::Six => CardRank::Six,
        Rank::Seven => CardRank::Seven,
        Rank::Eight => CardRank::Eight,
        Rank::Nine => CardRank::Nine,
        Rank::Ten => CardRank::Ten,
        Rank::Jack => CardRank::Jack,
        Rank::Queen => CardRank::Queen,
        Rank::King => CardRank::King,
        Rank::Ace => CardRank::Ace,
        Rank::Joker => CardRank::Joker,
    }
}

fn faces(snap: &push_ffi::TableSnapshot) -> Vec<(CardSuit, CardRank)> {
    snap.board
        .iter()
        .flat_map(|meld| meld.cards.iter().map(|card| (card.suit, card.rank)))
        .collect()
}

fn ids(snap: &push_ffi::TableSnapshot) -> Vec<u32> {
    snap.board
        .iter()
        .flat_map(|meld| meld.cards.iter().map(|card| card.id))
        .collect()
}
