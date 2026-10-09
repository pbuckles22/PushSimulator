//! A visitor can stop a legal-move walk before every action is listed.

use std::ops::ControlFlow;
use std::time::Instant;

use rand::rngs::StdRng;
use rand::SeedableRng;

use push_core::card::{Card, Rank, Suit};
use push_core::deck::Deck;
use push_core::game_state::GameState;
use push_core::legal_moves::{
    generate_legal_moves, visit_legal_kind, visit_legal_moves, LegalKind,
};
use push_core::player::Player;
use push_core::random_bot::play_random_turn;

fn take(deck: &mut Deck, suit: Suit, rank: Rank) -> Card {
    let index = deck
        .cards
        .iter()
        .position(|card| card.suit == suit && card.rank == rank)
        .expect("shoe has the card");
    deck.cards.remove(index)
}

fn table_from(hand: Vec<(Suit, Rank)>, round: u8) -> GameState {
    let mut deck = Deck::new();
    let mut cards = Vec::new();
    for (suit, rank) in hand {
        cards.push(take(&mut deck, suit, rank));
    }
    let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
    players[0].hand = cards;
    players[1].hand = vec![take(&mut deck, Suit::Clubs, Rank::Three)];
    let queen = take(&mut deck, Suit::Diamonds, Rank::Queen);
    deck.discard.push(queen);
    let mut state = GameState::new(players, deck);
    state.round_number = round;
    state
}

fn set_hand() -> Vec<(Suit, Rank)> {
    vec![
        (Suit::Hearts, Rank::Four),
        (Suit::Spades, Rank::Four),
        (Suit::Diamonds, Rank::Four),
        (Suit::Clubs, Rank::Four),
        (Suit::Hearts, Rank::Five),
        (Suit::Spades, Rank::Five),
        (Suit::Diamonds, Rank::Five),
        (Suit::Clubs, Rank::Five),
        (Suit::Hearts, Rank::Six),
        (Suit::Spades, Rank::Six),
        (Suit::Diamonds, Rank::Six),
        (Suit::Clubs, Rank::Six),
        (Suit::Hearts, Rank::Seven),
        (Suit::Spades, Rank::Seven),
    ]
}

fn heart_run_hand() -> Vec<(Suit, Rank)> {
    vec![
        (Suit::Hearts, Rank::Three),
        (Suit::Hearts, Rank::Four),
        (Suit::Hearts, Rank::Five),
        (Suit::Hearts, Rank::Six),
        (Suit::Hearts, Rank::Seven),
        (Suit::Hearts, Rank::Eight),
        (Suit::Hearts, Rank::Nine),
        (Suit::Hearts, Rank::Ten),
        (Suit::Hearts, Rank::Jack),
        (Suit::Hearts, Rank::Queen),
        (Suit::Hearts, Rank::King),
        (Suit::Hearts, Rank::Ace),
        (Suit::Diamonds, Rank::Three),
        (Suit::Spades, Rank::Four),
    ]
}

fn shoe_ids(state: &GameState) -> Vec<u32> {
    let mut ids = Vec::new();
    for player in &state.players {
        ids.extend(player.hand.iter().map(|card| card.id));
    }
    for meld in &state.board {
        ids.extend(meld.iter().map(|card| card.id));
    }
    ids.extend(state.deck.cards.iter().map(|card| card.id));
    ids.extend(state.deck.discard.iter().map(|card| card.id));
    ids.sort_unstable();
    ids
}

fn large_play_table() -> GameState {
    let state = table_from(set_hand(), 4);
    assert_eq!(state.players[0].hand.len(), 14);
    state
}

/// Round 4 lists 750 plays. `false` and `ControlFlow::Break` each stop after one.
/// `true` still walks the whole list, in the same order.
#[test]
fn test_visit_false_and_break_stop_a_large_play_walk() {
    let state = large_play_table();
    let mut full = Vec::new();
    let _ = visit_legal_kind(&state, 0, LegalKind::Play, &mut |action| {
        full.push(action);
        true
    });
    assert_eq!(full.len(), 750);

    let mut stopped = Vec::new();
    let _ = visit_legal_kind(&state, 0, LegalKind::Play, &mut |action| {
        stopped.push(action);
        false
    });
    assert_eq!(stopped.len(), 1);
    assert_eq!(stopped[0], full[0]);

    let mut broken = Vec::new();
    let _ = visit_legal_kind(&state, 0, LegalKind::Play, &mut |action| {
        broken.push(action);
        ControlFlow::<(), ()>::Break(())
    });
    assert_eq!(broken.len(), 1);
    assert_eq!(broken[0], full[0]);

    let mut continued = Vec::new();
    let _ = visit_legal_kind(&state, 0, LegalKind::Play, &mut |action| {
        continued.push(action);
        ControlFlow::<(), ()>::Continue(())
    });
    assert_eq!(continued, full);

    let mut partial = Vec::new();
    let _ = visit_legal_kind(&state, 0, LegalKind::Play, &mut |action| {
        partial.push(action);
        partial.len() < 3
    });
    assert_eq!(partial.len(), 3);
    assert_eq!(partial, full[..3]);
}

/// A hit, a steal, and a discard each stop when the visitor returns false.
#[test]
fn test_visit_false_stops_hit_steal_and_discard_walks() {
    let mut hitting = table_from(heart_run_hand(), 1);
    let six = take(&mut hitting.deck, Suit::Hearts, Rank::Six);
    let seven = take(&mut hitting.deck, Suit::Hearts, Rank::Seven);
    let eight = take(&mut hitting.deck, Suit::Hearts, Rank::Eight);
    hitting.board = vec![vec![six, seven, eight]];
    hitting.players[0].is_on_board = true;

    let mut hits = 0usize;
    let _ = visit_legal_kind(&hitting, 0, LegalKind::Hit, &mut |_| {
        hits += 1;
        true
    });
    assert!(hits > 1, "hit walk listed {hits}");
    let mut stopped_hits = 0usize;
    let _ = visit_legal_kind(&hitting, 0, LegalKind::Hit, &mut |_| {
        stopped_hits += 1;
        false
    });
    assert_eq!(stopped_hits, 1);

    let mut stealing = table_from(
        vec![(Suit::Diamonds, Rank::Eight), (Suit::Clubs, Rank::Eight)],
        1,
    );
    let joker = take(&mut stealing.deck, Suit::None, Rank::Joker);
    let eight_hearts = take(&mut stealing.deck, Suit::Hearts, Rank::Eight);
    let eight_spades = take(&mut stealing.deck, Suit::Spades, Rank::Eight);
    stealing.board = vec![vec![eight_hearts, eight_spades, joker]];
    stealing.players[0].is_on_board = true;
    let mut steals = 0usize;
    let _ = visit_legal_kind(&stealing, 0, LegalKind::Steal, &mut |_| {
        steals += 1;
        true
    });
    assert!(steals > 1, "steal walk listed {steals}");
    let mut stopped_steals = 0usize;
    let _ = visit_legal_kind(&stealing, 0, LegalKind::Steal, &mut |_| {
        stopped_steals += 1;
        false
    });
    assert_eq!(stopped_steals, 1);

    let discarding = large_play_table();
    let mut discards = 0usize;
    let _ = visit_legal_kind(&discarding, 0, LegalKind::Discard, &mut |_| {
        discards += 1;
        true
    });
    assert!(discards > 1, "discard walk listed {discards}");
    let mut stopped_discards = 0usize;
    let _ = visit_legal_kind(&discarding, 0, LegalKind::Discard, &mut |_| {
        stopped_discards += 1;
        false
    });
    assert_eq!(stopped_discards, 1);
}

/// Breaking on the first action leaves the later kinds unlisted.
#[test]
fn test_visit_legal_moves_break_skips_later_kinds() {
    let state = large_play_table();
    let all = generate_legal_moves(&state, 0);
    assert!(all.len() > 1);

    let mut seen = Vec::new();
    visit_legal_moves(&state, 0, &mut |action| {
        seen.push(action);
        false
    });
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0], all[0]);
}

/// Chain: Deck::new → visit stop → play_random_turn on a 70-card hand.
///
/// The capped walk returns. Every card id stays on the table.
#[test]
fn test_deck_new_visit_stop_play_random_turn_seventy_card_hand_finishes() {
    let mut deck = Deck::new();
    let mut hand = Vec::new();
    while hand.len() < 70 {
        hand.push(deck.cards.remove(0));
    }
    let other = deck.cards.remove(0);
    let queen = deck.cards.remove(0);
    deck.discard.push(queen);
    let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
    players[0].hand = hand;
    players[1].hand = vec![other];
    let mut state = GameState::new(players, deck);
    state.round_number = 1;
    assert!(state.players[0].hand.len() >= 70);
    let ids = shoe_ids(&state);

    let started = Instant::now();
    play_random_turn(&mut state, 0, &mut StdRng::seed_from_u64(1));
    let elapsed = started.elapsed();

    assert!(elapsed.as_secs() < 5, "70-card turn took {elapsed:?}");
    assert_eq!(shoe_ids(&state), ids);
    assert_eq!(state.players[0].points, 0);
    assert_eq!(state.players[1].points, 0);
}
